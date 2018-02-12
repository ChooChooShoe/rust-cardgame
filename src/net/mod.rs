pub mod client;
pub mod server;
pub mod message;

pub use self::message::Message;
pub use self::client::Client;
pub use self::server::Server;

use std::sync::Mutex;

pub enum NetworkMode {None,ClientOnly,ServerOnly,ClientServer}

pub fn create_local_client() -> Client 
{
    Client{}
}

use std::thread;
use std::sync::mpsc::channel;

pub fn create_local_clientserver() -> (Client,Server)
{
    let (tx, rx) = channel();
    for i in 0..10 {
        let tx = tx.clone();
        thread::spawn(move|| {
            tx.send(Message::Simple(i)).unwrap();
        });
    }

    for _ in 0..10 {
        let j = rx.recv().unwrap();
        println!("Recive {:?}", j);
    }
    (Client{},Server::new())
}

pub fn create_local_server() -> Server
{
    Server::new()
}