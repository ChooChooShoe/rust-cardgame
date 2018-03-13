use std::borrow::Borrow;
use ws::{self,Result,Request,Response,Message,Handshake,CloseCode,Handler,Error,ErrorKind};
use ws::Sender as WsSender;
use ws::util::{Token};
use io;
use std::thread;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as TSender;
use bincode::*;
use game::core::{self,Event};
use game::Action;
use net::NetworkMode;
use game::Game;
use std::error::Error as StdError;
use url;

pub fn connect<U: Borrow<str>>(url: U, game: Game) {
    let (send,recv) = channel();
    let h_game = game.clone();
    let thread_handle = thread::spawn(move || core::run(recv, NetworkMode::Client, h_game));

    ws::connect(url, |out: WsSender| {
        out.send(Action::DirectAttack(10,20).encode()).unwrap();
        out.send(Action::EndTurn(2).encode()).unwrap();
        out.send(Action::DrawCard(0,5).encode()).unwrap();
        Client::new(out, game.clone(), send.clone())
    }).unwrap();

    info!("Waiting for game thread to close.");
    thread_handle.join().unwrap();   
}
//fn def() {
//    let mut input = String::new();
//    match io::stdin().read_line(&mut input) {
//        Ok(num_bytes) => {
//            let c = input.trim().to_string();
//            if c == "bytes" {
//                self.ws_out.send(Message::Binary(vec!(num_bytes as u8)))?;
//            }
//            self.ws_out.send(Message::Text(c))
//        }
//        Err(_e) => Ok(())
//    }
//}

pub struct Client {
    ws_out: WsSender,
    game: Game,
    thread_out: TSender<Event>,
    pid: u8
}

impl Client {
    fn new(out: WsSender, game: Game, thread_out: TSender<Event>) -> Client {
        Client {
            ws_out: out,
            game,
            thread_out,
            pid: 0xFE,
        }
    }
}

fn thread_err<E: StdError>(e: E) -> Error {
    Error::new(ErrorKind::Internal,
        format!("Unable to communicate between threads: {:?}.", e))
}
impl Handler for Client {
    #[inline]
    fn on_shutdown(&mut self) {
        info!("Client received WebSocket shutdown request.");
    }
    
    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        self.thread_out
            .send(Event::Connect(self.ws_out.clone(), self.pid))
            .map_err(thread_err)
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        info!("Connection closing due to ({:?}) {}", code, reason);

        if let Err(err) = self.thread_out.send(Event::Disconnect(code, self.pid)) {
            error!("Error: Thread channel dropped on conection close: {:?}", err)
        }
    }


    fn on_message(&mut self, msg: Message) -> Result<()> {
        let action = Action::decode(msg);
        info!("Received action {:?}", action);
        self.thread_out.send(Event::TakeAction(action, self.pid)).map_err(|e| Error::new(
            ErrorKind::Internal, format!("Thread channel dropped wehn sending action: {:?}", e)
        ))
    }
    #[inline]
    fn on_request(&mut self, req: &Request) -> Result<Response> {
        info!("Client received request??");
        Response::from_request(req)
    }
    #[inline]
    fn on_response(&mut self, res: &Response) -> Result<()> {
        info!("Client received response.");
        // res.header() is private? why?
        let s = "rust-cardgame-playerid".to_lowercase();
        if let Some(playerid) = res.headers().iter().find(|&&(ref key, _)| key.to_lowercase() == s){
            if let Ok(s) = String::from_utf8(playerid.1.clone()){
                if let Ok(i) = s.parse::<u8>() {
                    info!("Server never gave us pid {}.", i);
                    self.pid = i;
                } else {
                    info!("Server never gave us a invalid pid '{}'.",s);
                    self.pid = 0xFB;
                }
            } else {
                info!("Server never gave us a bad string for pid.");
                self.pid = 0xFC;
            }
        } else {
            warn!("Server never gave us a player id.");
            self.pid = 0xFF;
        }
        Ok(())
    }
    #[inline]
    fn build_request(&mut self, url: &url::Url) -> Result<Request> {
        let mut req = try!(Request::from_url(url));
        req.add_protocol("player.rust-cardgame");
        req.headers_mut().push(("Rust-Cardgame-Version".into(), b"100".to_vec()));
        Ok(req)
    }
    
    fn on_error(&mut self, err: Error) {
        // Ignore connection reset errors by default, but allow library clients to see them by
        // overriding this method if they want
        if let ErrorKind::Io(ref err) = err.kind {
            if let Some(104) = err.raw_os_error() {
                warn!("Connection reset: {:?}", err);
                return
            }
        }
        error!("Client Error: {:?}", err);
        if self.thread_out.send(Event::WsError(err, self.pid)).is_err() {
            warn!("Thread channel dropped on ws error")
        }
    }
}