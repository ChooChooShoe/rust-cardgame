use net::{VERSION_HEADER,PID_HEADER,PROTOCOL};
use entity::CardPool;
use game::core::{self, Event};
use game::Game;
use game::{Action, ActionError, OkCode};
use net::server::server::Role;
use net::settings::ServerConfig;
use net::{Command, NetworkMode};
use player::controller::WsNetController;
use std::error::Error as StdError;
use std::net::ToSocketAddrs;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as TSender;
use std::thread;
use ws::util::Timeout;
use ws::util::Token;
use ws::Sender as WsSender;
use ws::{
    Builder, CloseCode, Error, ErrorKind, Factory, Frame, Handler, Handshake, Message, Request,
    Response, Result,
};
/// Represents one player's connection to us (the ServerHandle)
pub struct ServerHandle {
    ws: WsSender,
    core: TSender<Event>,
    player_id: i32,
    role: Role,
    expire_timeout: Option<Timeout>,
    mulligin_timeout: Option<Timeout>,
}
impl ServerHandle {
    pub fn new(ws: WsSender, core: TSender<Event>, player_id: i32, role: Role) -> ServerHandle {
        ServerHandle {
            ws,
            core,
            player_id,
            role,
            expire_timeout: None,
            mulligin_timeout: None,
        }
    }
    pub fn player_id(&self) -> i32 {
        self.player_id
    }
}

fn thread_err<E: StdError>(_e: E) -> Error {
    Error::new(
        ErrorKind::Internal,
        format!("Unable to communicate between threads: Core dropped early!"),
    )
}
const PING: Token = Token(1);
const EXPIRE: Token = Token(2);
const MULIGIN: Token = Token(3);

impl Handler for ServerHandle {
    /// Called when a request to shutdown all connections has been received.
    #[inline]
    fn on_shutdown(&mut self) {
        info!("ServerHandle received WebSocket shutdown request.");
    }

    fn on_open(&mut self, _shake: Handshake) -> Result<()> {
        // schedule a timeout to send a ping every 5 seconds.
        try!(self.ws.timeout(5_000, PING));
        // schedule a timeout to close the connection if there is no activity for 30 seconds.
        try!(self.ws.timeout(30_000, EXPIRE));
        // create a controller and send to thread.
        let controller = WsNetController::new(self.player_id as usize, self.ws.clone());
        let event = Event::Connect(controller.into());
        self.core.send(event).map_err(thread_err)
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        info!(
            "Connection closing due to ({:?}) {} for player_id {}",
            code, reason, self.player_id
        );
        let event = Event::Disconnect(code, self.player_id as usize);
        // Try to send and ignore any error.
        self.core.send(event).unwrap_or(())
    }

    /// Called on incoming messages.
    fn on_message(&mut self, msg: Message) -> Result<()> {
        let command = try!(Command::decode(&msg));
        info!("Received command {:?}", command);

        match command {
            Command::ChangePlayerId(from, to) => {
                self.player_id = to;
                Ok(())
            }
            Command::Text(t) => {
                info!("Received chat: {}", t);
                Ok(())
            }
            Command::TakeAction(action) => {
                info!("Received action {:?}", action);
                let ev = Event::TakeAction(action, self.player_id as usize);
                self.core.send(ev).map_err(thread_err)
            }
            _ => {
                warn!("Unsupported command recived. Ignoring.");
                Ok(())
            }
        }
    }

    /// Called when an error occurs on the WebSocket.
    fn on_error(&mut self, err: Error) {
        // Ignore connection reset errors by default, but allow library clients to see them by
        // overriding this method if they want
        if let ErrorKind::Io(ref err) = err.kind {
            if let Some(104) = err.raw_os_error() {
                return;
            }
        }

        error!("{:?}", err);
    }

    #[inline]
    fn on_timeout(&mut self, event: Token) -> Result<()> {
        match event {
            PING => {
                try!(self.ws.ping(vec![]));
                self.ws.timeout(5_000, PING)
            }
            EXPIRE => self.ws.close(CloseCode::Away),
            MULIGIN => {
                let event =
                    Event::TakeAction(Action::MuliginResult { swap: false }, self.player_id as usize);
                self.core.send(event).map_err(thread_err)
            }
            _ => Err(Error::new(
                ErrorKind::Internal,
                "Invalid timeout token encountered!",
            )),
        }
    }

    #[inline]
    fn on_new_timeout(&mut self, event: Token, timeout: Timeout) -> Result<()> {
        match event {
            EXPIRE => {
                // Cancel expire if one was scheduled and store the new one.
                if let Some(t) = self.expire_timeout.take() {
                    try!(self.ws.cancel(t))
                }
                self.expire_timeout = Some(timeout);
            }
            MULIGIN => {
                // same as expire.
                if let Some(t) = self.mulligin_timeout.take() {
                    try!(self.ws.cancel(t));
                }
                self.mulligin_timeout = Some(timeout);
            }
            _ => (),
        }
        Ok(())
    }

    #[inline]
    fn on_frame(&mut self, frame: Frame) -> Result<Option<Frame>> {
        // some activity has occurred, let's reset the expiration
        try!(self.ws.timeout(30_000, EXPIRE));
        Ok(Some(frame))
    }

    #[inline]
    fn on_request(&mut self, req: &Request) -> Result<Response> {
        info!("Server received request.");
        let mut res = Response::from_request(req)?;

        if req.protocols()?.iter().any(|&s| s.contains(PROTOCOL)) {
            res.set_protocol(PROTOCOL);
            res.headers_mut().push((
                PID_HEADER.into(),
                self.player_id.to_string().into_bytes(),
            ));
            res.headers_mut().push((
                VERSION_HEADER.into(),
                "0.0.1".into(),
            ));
            Ok(res)
        } else {
            Err(Error::new(
                ErrorKind::Protocol,
                "Protocol player.rust-cardgame was not given but is required.",
            ))
        }
    }

    #[inline]
    fn on_response(&mut self, _res: &Response) -> Result<()> {
        info!("ServerHandle received response. This should not happen!");
        Ok(())
    }

    //TODO ssl implementation.
}
