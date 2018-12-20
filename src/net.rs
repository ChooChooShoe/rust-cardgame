pub mod connection;
pub use self::connection::Connection;
pub use self::connection::Error as NetError;
pub use self::connection::Result as NetResult;
pub use ws::{Error as WsError, ErrorKind as WsErrorKind, Message as WsMessage};

use bincode::{deserialize, serialize, ErrorKind as BincodeError};
use crate::game::action::Action;
use crate::game::PlayerId;
use std::convert::From;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NetworkMode {
    Client,
    Server,
}

pub const PROTOCOL: &str = "player.rust-cardgame";
pub const VERSION_HEADER: &str = "rust-cardgame-version";
pub const PID_HEADER: &str = "rust-cardgame-playerid";

impl NetworkMode {
    #[inline]
    pub fn is_client(&self) -> bool {
        &NetworkMode::Client == self
    }
    #[inline]
    pub fn is_server(&self) -> bool {
        &NetworkMode::Server == self
    }
}
pub trait Codec: Sized {
    fn encode(&self) -> Result<WsMessage, WsError>;
    fn decode(msg: &WsMessage) -> Result<Self, WsError>;
}

fn bincode_ws(error: Box<BincodeError>) -> WsError {
    match *error {
        BincodeError::Io(e) => WsError::new(WsErrorKind::Io(e), ""),
        BincodeError::InvalidUtf8Encoding(e) => WsError::new(WsErrorKind::Encoding(e), ""),
        _ => WsError::new(WsErrorKind::Custom(error), ""),
    }
}
impl From<Action> for WsMessage {
    fn from(action: Action) -> WsMessage {
        match action {
            Action::Text(t) => WsMessage::Text(t),
            _ => WsMessage::Binary(serialize(&action).unwrap()),
        }
    }
}
impl Codec for Action {
    fn encode(&self) -> Result<WsMessage, WsError> {
        match self {
            Action::Text(t) => Ok(WsMessage::Text(t.to_string())),
            _ => Ok(WsMessage::Binary(serialize(&self).map_err(bincode_ws)?)),
        }
    }
    fn decode(msg: &WsMessage) -> Result<Self, WsError> {
        match msg {
            WsMessage::Text(t) => Ok(Action::Text(t.to_string())),
            WsMessage::Binary(b) => deserialize(&b[..]).map_err(bincode_ws),
        }
    }
}
