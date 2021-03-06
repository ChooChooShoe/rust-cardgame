use crate::game::stage::{NetRelay, Stage};
use crate::game::{Action, Game, GameSettings, PlayerId};
use crate::net::{Codec, Connection, NetworkMode, PROTOCOL, VERSION_HEADER};
use std::borrow::Borrow;
use std::error::Error as StdError;
use std::sync::mpsc::{channel, Sender as TSender};
use std::thread;
use url;
use ws::util::Token;
use ws::Sender as WsSender;
use ws::{
    self, CloseCode, Error, ErrorKind, Handler, Handshake, Message, Request, Response, Result,
};

pub fn connect<U: Borrow<str>>(url: U, id: usize, max_players: usize) {
    let game_settings = GameSettings::new(id, max_players, NetworkMode::Client);
    let (send, stage) = Stage::build(game_settings);
    let builder = thread::Builder::new().name(format!("client_{}", id));
    let thread_handle = builder.spawn(move || stage.run());

    ws::connect(url, |out: WsSender| Client::new(out, send.clone()))
        .expect("Couldn't begin connection to remote server and/or create a local client");

    info!("Waiting for client core to close.");
    thread_handle.unwrap().join().unwrap();
    info!("Client Done!");
}

pub struct Client {
    ws_out: WsSender,
    core: TSender<NetRelay>,
    player_id: PlayerId,
}

impl Client {
    fn new(out: WsSender, core: TSender<NetRelay>) -> Client {
        Client {
            ws_out: out,
            core,
            player_id: 0,
        }
    }
}

fn thread_err<E: StdError>(e: E) -> Error {
    Error::new(
        ErrorKind::Internal,
        format!("Unable to communicate between threads: {:?}.", e),
    )
}
impl Handler for Client {
    fn on_message(&mut self, msg: Message) -> Result<()> {
        let action = Action::decode(&msg)?;
        debug!("Client #{} got {:?}", self.player_id, action);

        match action {
            Action::ChangePlayerId(_from, to) => {
                let a = Action::ChangePlayerId(self.player_id, to);
                self.player_id = to;
                // still passed to the core.
                let ev = NetRelay::Act(self.player_id, a);
                self.core.send(ev).map_err(thread_err)
            }
            Action::Text(t) => {
                info!("Received chat: {}", t);
                Ok(())
            }
            _ => {
                // Any other action is sent to core thread.
                let ev = NetRelay::Act(self.player_id, action);
                self.core.send(ev).map_err(thread_err)
            }
        }
    }

    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        if let Some(addr) = shake.remote_addr()? {
            debug!("Connection with {} now open", addr);
        }
        let connection = Connection::from_network(self.player_id, self.ws_out.clone());
        let ev = NetRelay::Open(0, connection);

        self.core.send(ev).map_err(thread_err)
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        info!(
            "Client #{} closing do to ({:?}) '{}'",
            self.player_id, code, reason
        );
        let ev = NetRelay::Close(0);
        // Try to send and ignore any error.
        self.core.send(ev).unwrap_or(());
        // if self.core.send(Event::Shutdown()).is_err() {
        //     warn!("Unable to communicate between threads on close")
        // }
    }
    #[inline]
    fn on_shutdown(&mut self) {
        info!(
            "Client #{} received WebSocket shutdown request.",
            self.player_id
        );
        if let Err(e) = self.core.send(NetRelay::Shutdown(self.player_id)) {
            warn!("{}", e)
        }
    }

    #[inline]
    fn on_request(&mut self, req: &Request) -> Result<Response> {
        warn!("Client received request. This should not happen!");
        Response::from_request(req)
    }
    #[inline]
    fn on_response(&mut self, _res: &Response) -> Result<()> {
        debug!("Client received response.");
        Ok(())
        // res.header() is private? why?
        // let mut headers = res.headers().iter();
        // let search = headers.find(|&(ref key, _)| key.to_lowercase() == PID_HEADER);
        // if let Some(header_entry) = search {
        //     match String::from_utf8(header_entry.1.clone()) {
        //         Ok(pid_string) => if let Ok(pid) = pid_string.parse::<usize>() {
        //             info!("Client is now player id {}.", pid);
        //             self.player_id = pid;
        //             Ok(())
        //         } else {
        //             Err(Error::new(
        //                 ErrorKind::Protocol,
        //                 format!("Server gave us an invalid player id '{}'.", pid_string),
        //             ))
        //         },
        //         Err(x) => Err(Error::new(
        //             ErrorKind::Encoding(x.utf8_error()),
        //             "Server gave us an invalid UTF-8 string for player id.",
        //         )),
        //     }
        // } else {
        //     Err(Error::new(
        //         ErrorKind::Protocol,
        //         "Server never gave us a player id.",
        //     ))
        // }
    }
    #[inline]
    fn build_request(&mut self, url: &url::Url) -> Result<Request> {
        let mut req = Request::from_url(url)?;
        req.add_protocol(PROTOCOL);
        req.headers_mut()
            .push((VERSION_HEADER.into(), "0.0.1".into()));
        Ok(req)
    }

    fn on_error(&mut self, err: Error) {
        // Ignore connection reset errors by default, but allow library clients to see them by
        // overriding this method if they want
        if let ErrorKind::Io(ref err) = err.kind {
            if let Some(104) = err.raw_os_error() {
                warn!("Client #{} connection reset: {:?}", self.player_id, err);
                return;
            }
        }
        error!("Client #{} error: {:?}", self.player_id, err);
    }
}
