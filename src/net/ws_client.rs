use ws::{CloseCode,Handler,Sender};
use ws::*;
use ws::util::{Token, Timeout};
use url::Url;
use io;

pub fn connect(url: &str) -> Result<()> {
    let mut ws = try!(WebSocket::new(ClientFactory));
    let parsed = try!(
        Url::parse(url)
            .map_err(|err| Error::new(
                ErrorKind::Internal,
                format!("Unable to parse {} as url due to {:?}", url, err))));
    try!(ws.connect(parsed));
    try!(ws.run());
    Ok(())
}

struct ClientFactory;
impl Factory for ClientFactory
{
    type Handler = ClientHandler;

    fn connection_made(&mut self, out: Sender) -> ClientHandler {
        out.send("Hello WebSocket").unwrap();
        ClientHandler(out)
    }
    fn connection_lost(&mut self, _: Self::Handler) {
        warn!("Connection lost.");
    }

}
struct ClientHandler(Sender);
impl ClientHandler {
}
impl Handler for ClientHandler {
    #[inline]
    fn on_shutdown(&mut self) {
        info!("ClientHandler received WebSocket shutdown request.");
    }
    
    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        if let Some(addr) = try!(shake.remote_addr()) {
            info!("Connection with {} now open", addr);
        }
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        info!("Connection closing due to ({:?}) {}", code, reason);
    }

    #[inline]
    fn on_request(&mut self, req: &Request) -> Result<Response> {
        debug!("ClientHandler received request:\n{}", req);
        Response::from_request(req)
    }

    #[inline]
    fn on_response(&mut self, res: &Response) -> Result<()> {
        debug!("ClientHandler received response:\n{}", res);
        Ok(())
    }

    #[inline]
    fn on_timeout(&mut self, event: Token) -> Result<()> {
        debug!("ClientHandler received timeout token: {:?}", event);
        Ok(())
    }

    #[inline]
    fn on_new_timeout(&mut self, _: Token, _: Timeout) -> Result<()> {
        // default implementation discards the timeout handle
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        info!("Received message {:?}", msg);

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(num_bytes) => {
                let c = input.trim().to_string();
                if c == "bytes" {
                    self.0.send(Message::Binary(vec!(num_bytes as u8)))?;
                }
                self.0.send(Message::Text(c))
            }
            Err(_e) => Ok(())
        }
    }
}