use game::core::{self, Event};
use game::Action;
use game::Game;
use net::{Codec, Connection, NetworkMode};
use net::{PID_HEADER, PROTOCOL, VERSION_HEADER};
use std::borrow::Borrow;
use std::error::Error as StdError;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as TSender;
use std::thread;
use url;
use ws::util::Token;
use ws::Sender as WsSender;
use ws::{
    self, CloseCode, Error, ErrorKind, Handler, Handshake, Message, Request, Response, Result,
};

pub fn connect<U: Borrow<str>>(url: U) {
    let (send, recv) = channel();
    let client_core_handle =
        thread::spawn(move || core::run_client(recv, NetworkMode::Client, Game::new(2)));

    ws::connect(url, |out: WsSender| Client::new(out, send.clone()))
        .expect("Couldn't begin connection to remote server and/or create a local client");

    info!("Waiting for 'Game Core Thread' to close.");
    client_core_handle
        .join()
        .expect("Couldn't join on 'Game Core Thread'");
}

pub struct Client {
    ws_out: WsSender,
    thread_out: TSender<Event>,
    player_id: i32,
}

impl Client {
    fn new(out: WsSender, thread_out: TSender<Event>) -> Client {
        Client {
            ws_out: out,
            thread_out,
            player_id: -1,
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

        match action {
            Action::ChangePlayerId(_from, to) => {
                self.player_id = to as i32;
                Ok(())
            }
            Action::Text(t) => {
                info!("Received chat: {}", t);
                Ok(())
            }
            _ => {// Any other action is sent to core thread.
                info!("Client #{} received general action {:?}", self.player_id, action);
                let ev = Event::OnPlayerAction(self.player_id as usize, action);
                self.thread_out.send(ev).map_err(thread_err)
            }
        }
    }

    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        if let Some(addr) = try!(shake.remote_addr()) {
            debug!("Connection with {} now open", addr);
        }
        let connection = Connection::from_network(self.player_id as usize, self.ws_out.clone());
        let ev = Event::OpenConnection(self.player_id as usize, connection);

        self.thread_out.send(ev).map_err(thread_err)
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        info!("Connection closing due to ({:?}) {}", code, reason);

        let ev = Event::CloseConnection(self.player_id as usize);
        if let Err(err) = self.thread_out.send(ev) {
            error!(
                "Error: Thread channel dropped on conection close: {:?}",
                err
            )
        }
    }
    #[inline]
    fn on_shutdown(&mut self) {
        info!("Client received WebSocket shutdown request.");
    }

    #[inline]
    fn on_request(&mut self, req: &Request) -> Result<Response> {
        warn!("Client received request. This should not happen!");
        Response::from_request(req)
    }
    #[inline]
    fn on_response(&mut self, res: &Response) -> Result<()> {
        info!("Client received response.");
        // res.header() is private? why?
        if let Some(header_entry) = res
            .headers()
            .iter()
            .find(|&(ref key, _)| key.to_lowercase() == PID_HEADER)
        {
            match String::from_utf8(header_entry.1.clone()) {
                Ok(pid_string) => if let Ok(pid) = pid_string.parse::<i32>() {
                    info!("Client is now player id {}.", pid);
                    self.player_id = pid;
                    Ok(())
                } else {
                    Err(Error::new(
                        ErrorKind::Protocol,
                        format!("Server gave us a non-number player id '{}'.", pid_string),
                    ))
                },
                Err(x) => Err(Error::new(
                    ErrorKind::Encoding(x.utf8_error()),
                    "Server gave us an invalid UTF-8 string for player id.",
                )),
            }
        } else {
            Err(Error::new(
                ErrorKind::Protocol,
                "Server never gave us a player id.",
            ))
        }
    }
    #[inline]
    fn build_request(&mut self, url: &url::Url) -> Result<Request> {
        let mut req = try!(Request::from_url(url));
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
                warn!("Connection reset: {:?}", err);
                return;
            }
        }
        error!("Client Error: {:?}", err);
        if self
            .thread_out
            .send(Event::CloseConnection(self.player_id as usize))
            .is_err()
        {
            warn!("Thread channel dropped on ws error")
        }
    }
}
