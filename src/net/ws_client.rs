use ws::{self,Result,Request,Response,Message,Handshake,CloseCode,Handler,Error,ErrorKind};
use ws::Sender as WsSender;
use ws::util::{Token, Timeout};
use io;
use std::thread;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as TSender;
use bincode::*;

pub fn connect(url: &str) {
    let (send,recv) = channel();
    let thread_handle = thread::spawn(move || {
        for e in recv {
            info!("e: ");
        }
    });

    ws::connect(url, |out: WsSender| {
        out.send("Hello WebSocket").unwrap();
        Client {
            ws_out: out,
            thread_out: send.clone()
        }
    }).unwrap();

    //thread_handle.connect();    
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

// Message from clients to game loop.
pub enum Event {
    Connect(WsSender),
    Disconnect(CloseCode),
}

struct Client {
    ws_out: WsSender,
    thread_out: TSender<Event>
}

impl Client {
    fn new(out: WsSender, thread_out: TSender<Event>) -> Client {
        Client {
            ws_out: out,
            thread_out
        }
    }
}

impl Handler for Client {
    #[inline]
    fn on_shutdown(&mut self) {
        info!("Client received WebSocket shutdown request.");
    }
    
    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        self.thread_out
            .send(Event::Connect(self.ws_out.clone()))
            .map_err(|err| Error::new(
                ErrorKind::Internal, 
                format!("Unable to communicate between threads: {:?}.", err)
            ))
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        info!("Connection closing due to ({:?}) {}", code, reason);

        if let Err(err) = self.thread_out.send(Event::Disconnect(code)) {
            error!("Error on conection close: {:?}", err)
        }
    }


    fn on_message(&mut self, msg: Message) -> Result<()> {
        info!("Received message {:?}", msg);
        Ok(())
    }
}