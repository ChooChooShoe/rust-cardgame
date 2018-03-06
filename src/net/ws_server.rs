use std::net::ToSocketAddrs;
use ws::Handler;
use ws::*;

use ws::Message;
use ws::Frame;
use ws::CloseCode;
//use ws::handshake::{Handshake, Request, Response};
//use ws::result::{Result, Error, Kind};
use ws::util::{Token, Timeout};
use net::settings::ServerConfig;

pub fn listen<A: ToSocketAddrs>(ip: A) {
    let settings: Settings = ServerConfig::from_disk().into();

    let ws = Builder::new().with_settings(settings).build(ServerFactory).unwrap();
    
    ws.listen(ip).unwrap();
}

struct ServerFactory;
impl Factory for ServerFactory
{
    type Handler = ServerHandler;

    fn connection_made(&mut self, out: Sender) -> ServerHandler {
        ServerHandler(out)
    }
    fn connection_lost(&mut self, _: Self::Handler) {
        warn!("Connection lost.");
    }

}
struct ServerHandler(Sender);

impl ServerHandler {
}

impl Handler for ServerHandler {
    /// Called when a request to shutdown all connections has been received.
    #[inline]
    fn on_shutdown(&mut self) {
        info!("ServerHandler received WebSocket shutdown request.");
    }
    
    /// Called when the WebSocket handshake is successful and the connection is open for sending
    /// and receiving messages.
    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        if let Some(addr) = try!(shake.remote_addr()) {
            info!("Connection with {} now open", addr);
        }
        Ok(())
    }

    /// Called on incoming messages.
    fn on_message(&mut self, msg: Message) -> Result<()> {
        info!("Received message {:?}", msg);
        let t = msg.into_text()?;
        if t == "quit" {
            return self.0.close(CloseCode::Normal)
        }
        return self.0.send(format!("You say '{:?}' wow!",t))
    }

    /// Called any time this endpoint receives a close control frame.
    /// This may be because the other endpoint is initiating a closing handshake,
    /// or it may be the other endpoint confirming the handshake initiated by this endpoint.
    fn on_close(&mut self, code: CloseCode, reason: &str) {
        info!("Connection closing due to ({:?}) {}", code, reason);
    }

    /// Called when an error occurs on the WebSocket.
    //fn on_error(&mut self, err: Error);

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
        debug!("ServerHandler received request:\n{}", req);
        Response::from_request(req)
    }

    /// A method for handling the low-level workings of the response portion of the WebSocket
    /// handshake.
    ///
    /// Implementors can inspect the Response and choose to fail the connection by
    /// returning an error. This method will not be called when the handler represents a server
    /// endpoint. The response should indicate which WebSocket protocol and extensions the server
    /// has agreed to if any.
    #[inline]
    fn on_response(&mut self, res: &Response) -> Result<()> {
        debug!("ServerHandler received response:\n{}", res);
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
        info!("ServerHandler received timeout token: {:?}", event);
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
    #[inline]
    fn on_new_timeout(&mut self, _: Token, _: Timeout) -> Result<()> {
        // default implementation discards the timeout handle
        Ok(())
    }

    /// A method for wrapping a client TcpStream with Ssl Authentication machinery
    ///
    /// Override this method to customize how the connection is encrypted. By default
    /// this will use the Server Name Indication extension in conformance with RFC6455.
    #[inline]
    #[cfg(feature="ssl")]
    fn upgrade_ssl_client(&mut self, stream: TcpStream, url: &url::Url) -> Result<SslStream<TcpStream>>
    {
        let domain = try!(url.domain().ok_or(Error::new(
            Kind::Protocol,
            format!("Unable to parse domain from {}. Needed for SSL.", url))));
        let connector = try!(SslConnectorBuilder::new(SslMethod::tls()).map_err(|e| {
            Error::new(Kind::Internal, format!("Failed to upgrade client to SSL: {}", e))
        })).build();
        connector.connect(domain, stream).map_err(Error::from)
    }

    /// A method for wrapping a server TcpStream with Ssl Authentication machinery
    ///
    /// Override this method to customize how the connection is encrypted. By default
    /// this method is not implemented.
    #[inline]
    #[cfg(feature="ssl")]
    fn upgrade_ssl_server(&mut self, _: TcpStream) -> Result<SslStream<TcpStream>>
    {
        unimplemented!()
    }
}