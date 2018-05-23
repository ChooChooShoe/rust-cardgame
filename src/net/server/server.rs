use entity::CardPool;
use game::GameBoard;
use std::net::ToSocketAddrs;
use ws::{Builder, CloseCode, Error, ErrorKind, Factory, Frame, Handler, Handshake, Message,
         Request, Response, Result};
use ws::Sender as WsSender;
use ws::util::Token;
use net::settings::ServerConfig;
use game::{Action, ActionError, OkCode};
use game::core::{self,Event};
use std::thread;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as TSender;
use net::NetworkMode;
use game::Game;
use game::ClientAction;

pub fn listen<A: ToSocketAddrs>(ip: A, game: Game) {
    let settings = ServerConfig::from_disk().into();
    let (send, recv) = channel();
    let h_game = game.clone();
    let thread_handle = thread::spawn(move || core::run(recv, NetworkMode::Server, h_game));

    let factory = ServerFactory {
        sender: send,
        game,
        next_player_id: 0,
    };
    let ws = Builder::new().with_settings(settings).build(factory).unwrap();

    ws.listen(ip).unwrap();
    info!("Waiting for server game thread to close.");
    thread_handle.join().unwrap();
}
struct ServerFactory {
    sender: TSender<Event>,
    game: Game,
    next_player_id: usize,
}
impl Factory for ServerFactory {
    type Handler = ServerHandle;

    fn connection_made(&mut self, out: WsSender) -> ServerHandle {
        let ret = ServerHandle {
            ws_out: out,
            thread_out: self.sender.clone(),
            pid: self.next_player_id,
        };
        self.next_player_id += 1;
        ret
    }
    fn connection_lost(&mut self, _: ServerHandle) {
        warn!("Connection lost.");
    }
}

/// Represents one player's connection to us (the ServerHandle)
pub struct ServerHandle{
    ws_out: WsSender,
    thread_out: TSender<Event>,
    pid: usize,
}

fn thread_err<E: ::std::error::Error>(e: E, pid: usize) -> Error {
    Error::new(ErrorKind::Internal,
        format!("Unable to communicate between threads for pid {} : {:?}.", pid, e))
}
impl Handler for ServerHandle {
    /// Called when a request to shutdown all connections has been received.
    #[inline]
    fn on_shutdown(&mut self) {
        info!("ServerHandle received WebSocket shutdown request.");
    }

    fn on_open(&mut self, _shake: Handshake) -> Result<()> {
        let event = Event::Connect(::player::Controller::new(self.pid as usize, self.ws_out.clone()));
        self.thread_out.send(event).map_err(|e| thread_err(e,self.pid))
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        info!("Connection closing due to ({:?}) {}", code, reason);

        if let Err(err) = self.thread_out.send(Event::Disconnect(code, self.pid as usize)) {
            error!("Error on conection close (pid:{}): {:?}", self.pid, err)
        }
    }

    /// Called on incoming messages.
    fn on_message(&mut self, msg: Message) -> Result<()> {
        let action = ClientAction::decode(msg);
        info!("Received action {:?}", action);
        self.thread_out.send(Event::OnClientAction(action, self.pid)).map_err(|e| thread_err(e,self.pid))
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

    // handshake events

    /// A method for handling the low-level workings of the request portion of the WebSocket
    /// handshake.
    ///
    /// Implementors should select a WebSocket protocol and extensions where they are supported.
    ///
    /// Implementors can inspect the Request and must return a Response or an error
    /// indicating that the handshake failed. The default implementation provides conformance with
    /// the WebSocket protocol, and implementors should use the `Response::from_request` method and
    /// then modify the resulting response as necessary in order to maintain conformance.
    ///
    /// This method will not be called when the handler represents a client endpoint. Use
    /// `build_request` to provide an initial handshake request.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut res = try!(Response::from_request(req));
    /// if try!(req.extensions()).iter().find(|&&ext| ext.contains("myextension-name")).is_some() {
    ///     res.add_extension("myextension-name")
    /// }
    /// Ok(res)
    /// ```
    #[inline]
    fn on_request(&mut self, req: &Request) -> Result<Response> {
        info!("Server received request.\n----\n{:?}\n----\n", req);
        let mut res = try!(Response::from_request(req));
        if try!(req.protocols()).iter().find(|&&proto| proto.contains("player.rust-cardgame")).is_some() {
            res.set_protocol("player.rust-cardgame");
            res.headers_mut().push((format!("Rust-Cardgame-PlayerId"), self.pid.to_string().into_bytes()));
            
            if let Some(header) = req.headers().iter().find(|ref header| header.0.contains("Rust-Cardgame-Version")) {
                match &header.1[..] {
                    b"100" => {
                        info!("sending: {:?}", res);
                        Ok(res)
                    },
                    _ => Err(Error::new(ErrorKind::Protocol, format!("Version Missmatch: Expected 1.0.0 got {:?}", header.1)))
                }
            } else {
                Err(Error::new(ErrorKind::Protocol, "No Rust-Cardgame-Version given but is required."))
            }
        } else {
            Err(Error::new(ErrorKind::Protocol, "Protocol player.rust-cardgame is required."))
        }
    }

    /// A method for handling the low-level workings of the response portion of the WebSocket
    /// handshake.
    ///
    /// Implementors can inspect the Response and choose to fail the connection by
    /// returning an error. This method will not be called when the handler represents a Server
    /// endpoint. The response should indicate which WebSocket protocol and extensions the Server
    /// has agreed to if any.
    #[inline]
    fn on_response(&mut self, res: &Response) -> Result<()> {
        info!("ServerHandle received response??");
        Ok(())
    }

    // timeout events

    /// Called when a timeout is triggered.
    ///
    /// This method will be called when the eventloop encounters a timeout on the specified
    /// token. To schedule a timeout with your specific token use the `Sender::timeout` method.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// const GRATI: Token = Token(1);
    ///
    /// ... Handler
    ///
    /// fn on_open(&mut self, _: Handshake) -> Result<()> {
    ///     // schedule a timeout to send a gratuitous pong every 5 seconds
    ///     self.ws.timeout(5_000, GRATI)
    /// }
    ///
    /// fn on_timeout(&mut self, event: Token) -> Result<()> {
    ///     if event == GRATI {
    ///         // send gratuitous pong
    ///         try!(self.ws.pong(vec![]))
    ///         // reschedule the timeout
    ///         self.ws.timeout(5_000, GRATI)
    ///     } else {
    ///         Err(Error::new(ErrorKind::Internal, "Invalid timeout token encountered!"))
    ///     }
    /// }
    /// ```
    #[inline]
    fn on_timeout(&mut self, event: Token) -> Result<()> {
        info!("ServerHandle received timeout token: {:?}", event);
        Ok(())
    }

    /// Called when a timeout has been scheduled on the eventloop.
    ///
    /// This method is the hook for obtaining a Timeout object that may be used to cancel a
    /// timeout. This is a noop by default.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// const PING: Token = Token(1);
    /// const EXPIRE: Token = Token(2);
    ///
    /// ... Handler
    ///
    /// fn on_open(&mut self, _: Handshake) -> Result<()> {
    ///     // schedule a timeout to send a ping every 5 seconds
    ///     try!(self.ws.timeout(5_000, PING));
    ///     // schedule a timeout to close the connection if there is no activity for 30 seconds
    ///     self.ws.timeout(30_000, EXPIRE)
    /// }
    ///
    /// fn on_timeout(&mut self, event: Token) -> Result<()> {
    ///     match event {
    ///         PING => {
    ///             self.ws.ping(vec![]);
    ///             self.ws.timeout(5_000, PING)
    ///         }
    ///         EXPIRE => self.ws.close(CloseCode::Away),
    ///         _ => Err(Error::new(ErrorKind::Internal, "Invalid timeout token encountered!")),
    ///     }
    /// }
    ///
    /// fn on_new_timeout(&mut self, event: Token, timeout: Timeout) -> Result<()> {
    ///     if event == EXPIRE {
    ///         if let Some(t) = self.timeout.take() {
    ///             try!(self.ws.cancel(t))
    ///         }
    ///         self.timeout = Some(timeout)
    ///     }
    ///     Ok(())
    /// }
    ///
    /// fn on_frame(&mut self, frame: Frame) -> Result<Option<Frame>> {
    ///     // some activity has occurred, let's reset the expiration
    ///     try!(self.ws.timeout(30_000, EXPIRE));
    ///     Ok(Some(frame))
    /// }
    /// ```
    //#[inline]
    //fn on_new_timeout(&mut self, _: Token, _: Timeout) -> Result<()> {
    //    // default implementation discards the timeout handle
    //    Ok(())
    //}

    /// A method for wrapping a client TcpStream with Ssl Authentication machinery
    ///
    /// Override this method to customize how the connection is encrypted. By default
    /// this will use the ServerHandle Name Indication extension in conformance with RFC6455.
    #[inline]
    #[cfg(feature = "ssl")]
    fn upgrade_ssl_client(&mut self, stream: TcpStream, url: &url::Url) -> Result<SslStream<TcpStream>> {
        let domain = try!(url.domain().ok_or(Error::new(
            Kind::Protocol,
            format!("Unable to parse domain from {}. Needed for SSL.", url)
        )));
        let connector = try!(
            SslConnectorBuilder::new(SslMethod::tls()).map_err(|e| Error::new(
                Kind::Internal,
                format!("Failed to upgrade client to SSL: {}", e)
            ))
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
