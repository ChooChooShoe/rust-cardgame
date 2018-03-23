pub mod server;
pub mod client;
mod settings;

//pub use self::message::Message;
//pub use self::gameclient::GameClient;
//pub use self::gameserver::GameServer;

use std::sync::Mutex;
use std::convert::{From, Into};
use bincode::Result;
use ws::Message;

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

pub trait IntoMessage: Sized {
    fn try_encode(&self) -> Result<Message>;
    fn try_decode(msg: Message) -> Result<Self>;

    fn encode(&self) -> Message {
        self.try_encode().expect("Message encoding failed")
    }
    fn decode(msg: Message) -> Self {
        Self::try_decode(msg).expect("Message decoding failed")
    }
}
