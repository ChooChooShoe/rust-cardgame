use entity::CardPool;
use game::core::{self, Event};
use game::{Action, ActionError, Game, OkCode};
use net::server::server::Role;
use net::settings::ServerConfig;
use net::{Codec, Connection, NetworkMode};
use net::{PID_HEADER, PROTOCOL, VERSION_HEADER};
use std::error::Error as StdError;
use std::net::ToSocketAddrs;
use std::sync::mpsc::{channel, Sender as TSender};
use std::thread;
use ws::util::Timeout;
use ws::util::Token;
use ws::{
    Builder, CloseCode, Error, ErrorKind, Factory, Frame, Handler, Handshake, Message, Request,
    Response, Result, Sender as WsSender,
};
/// Represents one player's connection to us (the ServerHandle)
pub struct ServerHandle {
    ws: WsSender,
    core: TSender<Event>,
    player_id: usize,
    role: Role,
    expire_timeout: Option<Timeout>,
    mulligin_timeout: Option<Timeout>,
}
impl ServerHandle {
    pub fn new(ws: WsSender, core: TSender<Event>, player_id: usize, role: Role) -> ServerHandle {
        ServerHandle {
            ws,
            core,
            player_id,
            role,
            expire_timeout: None,
            mulligin_timeout: None,
        }
    }
    pub fn player_id(&self) -> usize {
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

const GAMESTART: Token = Token(12);

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

        match self.role {
            Role::Player(is_final) => {
                if is_final {
                    // timeout for 200 ms to start the game.
                    try!(self.ws.timeout(0_200, GAMESTART));
                }
                // create a controller and send to thread.
                let conn = Connection::from_network(self.player_id, self.ws.clone());
                let event = Event::OpenConnection(self.player_id, conn);
                self.core.send(event).map_err(thread_err)
            }
            Role::GameFull => self.ws.close(CloseCode::Normal),
            Role::Spectator => Err(Error::new(
                ErrorKind::Internal,
                "Spectator is not implemented",
            )),
        }
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        info!(
            "Connection closing due to ({:?}) {} for player_id {}",
            code, reason, self.player_id
        );
        let event = Event::CloseConnection(self.player_id as usize);
        // Try to send and ignore any error.
        self.core.send(event).unwrap_or(())
    }

    /// Called on incoming messages.
    fn on_message(&mut self, msg: Message) -> Result<()> {
        let action = Action::decode(&msg)?;
        info!("Received command {:?}", action);

        match action {
            Action::ChangePlayerId(_from, _to) => {
                warn!("Command::ChangePlayerId is not implemented by the server.");
                //self.player_id = to;
                Ok(())
            }
            Action::Text(t) => {
                info!("Server recived chat from player #{}: {}", self.player_id, t);
                self.ws
                    .send(Action::Text(String::from("You know im a computer, right?")))
            }
            _ => {
                // Any other action is sent to core thread.
                info!(
                    "Server received general action {:?} from player #{}",
                    action, self.player_id
                );
                let ev = Event::OnPlayerAction(self.player_id as usize, action);
                self.core.send(ev).map_err(thread_err)
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
                let ev = Event::OnPlayerAction(
                    self.player_id as usize,
                    Action::MuliginResult { swap: false },
                );
                self.core.send(ev).map_err(thread_err)
            }
            GAMESTART => {
                self.core.send(Event::AllPlayersConnected()).map_err(thread_err)
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
            res.headers_mut()
                .push((PID_HEADER.into(), self.player_id.to_string().into_bytes()));
            res.headers_mut()
                .push((VERSION_HEADER.into(), "0.0.1".into()));
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
