//pub mod gameclient;
//pub mod gameserver;
//pub mod message;
pub mod ws_server;
pub mod ws_client;
mod settings;

//pub use self::message::Message;
//pub use self::gameclient::GameClient;
//pub use self::gameserver::GameServer;

use std::sync::Mutex;

pub enum NetworkMode {None,ClientOnly,ServerOnly,ClientServer}

pub trait Networked {
    fn netid(&self) -> u64;
}


pub fn create_client_and_server() {

}