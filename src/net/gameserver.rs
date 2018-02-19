use std::net::{TcpStream, TcpListener};
use std::io::{Read, Write, Error};
use std::sync::mpsc;
use std::thread;
use std::thread::JoinHandle;

use tarpc;
use tokio_core::reactor;
use tarpc::util::{FirstSocketAddr, Never};

//use tarpc::sync::{client, server};
//use tarpc::sync::client::ClientExt;

use futures::Future;
use tarpc::future::{client, server};
use tarpc::future::client::ClientExt;

service! {
    rpc hello(name: String) -> String;
    rpc version() -> String;
}

#[derive(Clone)]
pub struct GameServer;
impl FutureService for GameServer {
    
    type HelloFut = Result<String, Never>;
    fn hello(&self, name: String) -> Result<String, Never> {
        if name.len() > 6{
            Ok(format!("Hello, {}!", name.split_at(6).0.to_string()))
        }else{
            Ok(format!("Hello, {}!", name))
        }
    }

    type VersionFut = Result<String, Never>;
    fn version(&self) -> Self::VersionFut {
        Ok(String::from("0.1.1"))
    }
}

pub fn create_sync()
{
    //let (tx, rx) = mpsc::channel();
    //let h = thread::spawn(move || {
    //    let mut handle = GameServer.listen("localhost:36650", server::Options::default())
    //        .unwrap();
    //    tx.send(handle.addr()).unwrap();
    //    handle.run();
    //});
    //let x = rx.recv().unwrap();
    //let client = SyncClient::connect(x, client::Options::default()).unwrap();
    //println!("{}", client.hello("Mom".to_string()).unwrap());
    //h.join().unwrap();
}

pub fn create_server() {
    info!("Create Server");
    //let handle = GameServer.listen("localhost:36650", server::Options::default()).unwrap();
    //handle.addr();
    //handle.run();

    let mut reactor = reactor::Core::new().unwrap();
    let (handle, server) = GameServer.listen(
        "localhost:12512".first_socket_addr(),
        &reactor.handle(),
        server::Options::default())
        .unwrap();

    //reactor.handle().spawn(server);
    reactor.run(server).unwrap();
    
}
pub fn create_client() {
    info!("Create Client");
    //let x = "localhost:36650";
    //let client = SyncClient::connect(x, client::Options::default()).unwrap();
    //let client2 = SyncClient::connect(x, client::Options::default()).unwrap();
    //println!("{}", client.hello("Mom".to_string()).unwrap());
    //println!("{}", client2.hello("Dada".to_string()).unwrap());
    
    let mut reactor = reactor::Core::new().unwrap();
    let options = client::Options::default().handle(reactor.handle());
    reactor.run(FutureClient::connect("localhost:12512".first_socket_addr(), options)
        .map_err(tarpc::Error::from)
        .and_then(|client| client.hello("Mom".to_string()))
        .map(|resp| println!("{}", resp)))
        .unwrap();
}