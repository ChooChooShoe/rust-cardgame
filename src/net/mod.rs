pub mod server;
pub mod client;
mod settings;

//pub use self::message::Message;
//pub use self::gameclient::GameClient;
//pub use self::gameserver::GameServer;

use std::sync::Mutex;

#[derive(Eq,PartialEq,Clone,Debug)]
pub enum NetworkMode {Client,Server,Both}

impl NetworkMode {
    #[inline]
    pub fn is_client(&self) -> bool {
        self != &NetworkMode::Server
    }
    #[inline]
    pub fn is_server(&self) -> bool {
        self != &NetworkMode::Client
    }
}
pub trait Networked {
    fn netid(&self) -> u64;
}

