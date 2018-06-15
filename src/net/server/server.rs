use entity::CardPool;
use game::core::{self, Event};
use game::Game;
use game::GameBoard;
use game::{Action, ActionError, OkCode};
use net::settings::ServerConfig;
use net::NetworkMode;
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

pub fn listen<A: ToSocketAddrs>(ip: A, game: Game) {
    let settings = ServerConfig::from_disk().into();
    let (send, recv) = channel();
    let h_game = game.clone();
    let thread_handle = thread::spawn(move || core::run(recv, NetworkMode::Server, h_game));

    let factory = ServerFactory {
        sender: send,
        game,
        next_player_id: 0,
        max_players: 2,
    };
    let ws = Builder::new()
        .with_settings(settings)
        .build(factory)
        .unwrap();

    ws.listen(ip).unwrap();
    info!("Waiting for server game thread to close.");
    thread_handle.join().unwrap();
}
struct ServerFactory {
    sender: TSender<Event>,
    game: Game,
    next_player_id: usize,
    max_players: usize,
}
impl Factory for ServerFactory {
    type Handler = ServerHandle;

    fn connection_made(&mut self, out: WsSender) -> ServerHandle {
        if self.next_player_id < self.max_players {
            let ret =
                ServerHandle::new(out, self.sender.clone(), self.next_player_id, Role::Player);
            self.next_player_id += 1;
            ret
        } else {
            ServerHandle::new(
                out,
                self.sender.clone(),
                self.next_player_id,
                Role::GameFull,
            )
        }
    }
    fn connection_lost(&mut self, handle: ServerHandle) {
        warn!("Connection lost for pid {}", handle.pid);
        self.sender.send(Event::ConnectionLost(handle.pid)).unwrap();
    }
    fn on_shutdown(&mut self) {
        info!("ServerFactory received WebSocket shutdown request.");
        self.sender.send(Event::OnShutdown()).unwrap_or(());
    }
}

pub enum Role {
    Player,
    Spectator,
    GameFull,
}
/// Represents one player's connection to us (the ServerHandle)
pub struct ServerHandle {
    ws: WsSender,
    core: TSender<Event>,
    pid: usize,
    role: Role,
    expire_timeout: Option<Timeout>,
    mulligin_timeout: Option<Timeout>,
}
impl ServerHandle {
    fn new(ws: WsSender, core: TSender<Event>, pid: usize, role: Role) -> ServerHandle {
        ServerHandle {
            ws,
            core,
            pid,
            role,
            expire_timeout: None,
            mulligin_timeout: None,
        }
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
        let controller = WsNetController::new(self.pid, self.ws.clone());
        let event = Event::Connect(Box::new(controller));
        self.core.send(event).map_err(thread_err)
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        info!(
            "Connection closing due to ({:?}) {} for pid {}",
            code, reason, self.pid
        );
        let event = Event::Disconnect(code, self.pid);
        // Try to send and ignore any error.
        self.core.send(event).unwrap_or(())
    }

    /// Called on incoming messages.
    fn on_message(&mut self, msg: Message) -> Result<()> {
        let action = try!(Action::decode(&msg));
        info!("Received action {:?}", action);
        let event = Event::OnClientAction(action, self.pid);
        self.core.send(event).map_err(thread_err)
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
                let event = Event::OnClientAction(Action::MuliginResult { swap: false }, self.pid);
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
        let mut res = try!(Response::from_request(req));
        if try!(req.protocols())
            .iter()
            .find(|&&proto| proto.contains("player.rust-cardgame"))
            .is_some()
        {
            res.set_protocol("player.rust-cardgame");
            res.headers_mut().push((
                format!("Rust-Cardgame-PlayerId"),
                self.pid.to_string().into_bytes(),
            ));

            if let Some(header) = req
                .headers()
                .iter()
                .find(|ref header| header.0.contains("Rust-Cardgame-Version"))
            {
                match &header.1[..] {
                    b"100" => Ok(res),
                    _ => Err(Error::new(
                        ErrorKind::Protocol,
                        format!("Version Missmatch: Expected 1.0.0 got {:?}", header.1),
                    )),
                }
            } else {
                Err(Error::new(
                    ErrorKind::Protocol,
                    "No Rust-Cardgame-Version given but is required.",
                ))
            }
        } else {
            Err(Error::new(
                ErrorKind::Protocol,
                "Protocol player.rust-cardgame is required.",
            ))
        }
    }

    #[inline]
    fn on_response(&mut self, res: &Response) -> Result<()> {
        info!("ServerHandle received response. This should not happen!");
        Ok(())
    }

    /// A method for wrapping a client TcpStream with Ssl Authentication machinery
    ///
    /// Override this method to customize how the connection is encrypted. By default
    /// this will use the ServerHandle Name Indication extension in conformance with RFC6455.
    #[inline]
    #[cfg(feature = "ssl")]
    fn upgrade_ssl_client(
        &mut self,
        stream: TcpStream,
        url: &url::Url,
    ) -> Result<SslStream<TcpStream>> {
        let domain = try!(url.domain().ok_or(Error::new(
            Kind::Protocol,
            format!("Unable to parse domain from {}. Needed for SSL.", url)
        )));
        let connector = try!(
            SslConnectorBuilder::new(SslMethod::tls()).map_err(|e| {
                Error::new(
                    Kind::Internal,
                    format!("Failed to upgrade client to SSL: {}", e),
                )
            })
        ).build();
        connector.connect(domain, stream).map_err(Error::from)
    }

    /// A method for wrapping a ServerHandle TcpStream with Ssl Authentication machinery
    ///
    /// Override this method to customize how the connection is encrypted. By default
    /// this method is not implemented.
    #[inline]
    #[cfg(feature = "ssl")]
    fn upgrade_ssl_server(&mut self, _: TcpStream) -> Result<SslStream<TcpStream>> {
        unimplemented!()
    }
}
