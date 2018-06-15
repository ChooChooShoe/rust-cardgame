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
use player::controller::WsNetController;
use std::error::Error as StdError;
use url;

pub fn connect<U: Borrow<str>>(url: U, game: Game) {
    let (send,recv) = channel();
    let h_game = game.clone();
    let thread_handle = thread::spawn(move || core::run_client(recv, NetworkMode::Client, h_game));

    ws::connect(url, |out: WsSender| {
        Client::new(out, game.clone(), send.clone())
    }).expect("Couldn't begin connection to remote server and/or create a local client");

    info!("Waiting for the game client thread to close.");
    thread_handle.join().expect("Couldn't join on the game client thread");
}

pub struct Client {
    ws_out: WsSender,
    game: Game,
    thread_out: TSender<Event>,
    player_index: Option<usize>
}

impl Client {
    fn new(out: WsSender, game: Game, thread_out: TSender<Event>) -> Client {
        Client {
            ws_out: out,
            game,
            thread_out,
            player_index: None,
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
        if let Some(addr) = try!(shake.remote_addr()) {
            debug!("Connection with {} now open", addr);
        }
        let controller = WsNetController::new(self.player_index.unwrap_or_default(), self.ws_out.clone());
        let ev = Event::Connect(Box::new(controller));
        
        self.thread_out.send(ev).map_err(thread_err)
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        info!("Connection closing due to ({:?}) {}", code, reason);

        if let Err(err) = self.thread_out.send(Event::Disconnect(code, self.player_index.unwrap_or_default())) {
            error!("Error: Thread channel dropped on conection close: {:?}", err)
        }
    }


    fn on_message(&mut self, msg: Message) -> Result<()> {
        let action = try!(Action::decode(&msg));
        info!("Received action {:?}", action);
        let ev = Event::TakeAction(action, self.player_index.unwrap_or_default());
        self.thread_out.send(ev).map_err(thread_err)
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
        let s = "rust-cardgame-playerid".to_lowercase();
        if let Some(header_entry) = res.headers().iter().find(|&&(ref key, _)| key.to_lowercase() == s){
            match String::from_utf8(header_entry.1.clone()) {
                Ok(pid_string) =>
                    if let Ok(pid) = pid_string.parse::<usize>() {
                        info!("Client is now player id {}.", pid);
                        self.player_index = Some(pid);
                        Ok(())
                    } else {
                        Err(Error::new(ErrorKind::Protocol, format!("Server gave us a non-number player id '{}'.", pid_string)))
                    }
                Err(x) => {
                    Err(Error::new(ErrorKind::Encoding(x.utf8_error()), "Server gave us an invalid UTF-8 string for player id."))
                }
            }
        } else {
            Err(Error::new(ErrorKind::Protocol, "Server never gave us a player id."))
        }
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
        if self.thread_out.send(Event::WsError(err, self.player_index.unwrap_or_default())).is_err() {
            warn!("Thread channel dropped on ws error")
        }
    }
}