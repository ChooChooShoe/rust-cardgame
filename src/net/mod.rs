pub mod server;
pub mod client;
mod settings;

//pub use self::message::Message;
//pub use self::gameclient::GameClient;
//pub use self::gameserver::GameServer;

use std::sync::Mutex;

pub enum NetworkMode {None,ClientOnly,ServerOnly,ClientServer}

pub trait Networked {
    fn netid(&self) -> u64;
}

