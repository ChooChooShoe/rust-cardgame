pub mod client;
pub mod command;
pub mod server;
mod settings;

pub use self::command::Command;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum NetworkMode {
    Client,
    Server,
    Both,
}

const PROTOCOL: &str = "player.rust-cardgame";
const VERSION_HEADER: &str = "rust-cardgame-version";
const PID_HEADER: &str = "rust-cardgame-playerid";

impl NetworkMode {
    #[inline]
    pub fn is_client(&self) -> bool {
        self != &NetworkMode::Server
    }
    #[inline]
    pub fn is_server(&self) -> bool {
        self != &NetworkMode::Client
    }
    #[inline]
    pub fn is_both(&self) -> bool {
        self == &NetworkMode::Both
    }
}
