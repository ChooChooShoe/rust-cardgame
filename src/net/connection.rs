use bincode::{serialize, ErrorKind};
use crate::game::{Action, PlayerId};
use crate::net::Codec;
use std::error::Error as StdError;
use std::fmt;
use std::result::Result as StdResult;
use std::sync::mpsc::Sender as TSender;
use ws::{CloseCode, Message, Sender as WsSender, Error as WsError};

pub type Result<T> = StdResult<T, Error>;
/// A simple wrapped network error
#[derive(Debug)]
pub enum Error {
    NoConnection,
    Disconnected,
    Encoding(ErrorKind),
    Sending(WsError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(cause) = self.cause() {
            write!(f, "{}: {}", self.description(), cause.description())
        } else {
            write!(f, "{}", self.description())
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            Error::Encoding(_) => "Encoding Error",
            Error::Sending(_) => "Sending Error",
            Error::NoConnection => "No Connection",
            Error::Disconnected => "Disconnected",
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match self {
            Error::Encoding(e) => Some(e),
            Error::Sending(e) => Some(e),
            Error::NoConnection => None,
            Error::Disconnected => None,
        }
    }
}

fn encode_action(action: &Action) -> Result<Message> {
    match action {
        Action::Text(t) => Ok(Message::Text(t.to_string())),
        _ => {
            let data = serialize(action).map_err(|e| Error::Encoding(*e))?;
            Ok(Message::Binary(data))
        },
    }
}

pub enum Connection {
    WsPlayer(PlayerId, WsSender),
    Other(PlayerId),
}
impl Connection {
    pub fn from_network(player_id: PlayerId, sender: WsSender) -> Connection {
        Connection::WsPlayer(player_id, sender)
    }
    pub fn from_empty(player_id: PlayerId) -> Connection {
        Connection::Other(player_id)
    }

    /// Called to encode and send the action.
    pub fn send(&self, action: &Action) -> Result<()> {
        match self {
            Connection::WsPlayer(_, ws) => {
                let message = encode_action(action)?;
                Ok(ws.send(message).map_err(|e| Error::Sending(e))?)
            }
            Connection::Other(_) => Err(Error::NoConnection),
        }
    }
    /// Gets the player id that ownes this connection.
    pub fn player_id(&self) -> PlayerId {
        match self {
            Connection::WsPlayer(player_id, _) => *player_id,
            Connection::Other(player_id) => *player_id,
        }
    }
    /// Called to make a manual disconnect.
    pub fn disconnect(&mut self) {
        match self {
            Connection::WsPlayer(_, ws) =>{
                let _res = ws.close_with_reason(CloseCode::Normal, "Disconnect");
            }
            _ => (),
        }
    }
    /// Called when the server has requested shutdown.
    pub fn shutdown(&mut self) {
        match self {
            Connection::WsPlayer(_, ws) => ws.shutdown().unwrap_or(()),
            _ => (),
        }
    }
    /// Called when this conntion is getting dropped.
    /// or when closed from the other end.
    pub fn destroy(&mut self) {}
}
