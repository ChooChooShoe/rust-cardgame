use crate::game::Action;
use crate::net::Codec;
use std::error::Error as StdError;
use std::fmt;
use std::ops::Deref;
use std::result::Result as StdResult;
use std::sync::mpsc::Sender as TSender;
use ws::{CloseCode, Sender as WsSender};

pub type Result = StdResult<(), Error>;
/// A simple wrapped network error
#[derive(Debug)]
pub enum Error {
    Internal,
    NoConnection,
    Encoding(usize),
    Sending(usize),
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
            Error::Internal => "Internal Application Error",
            Error::Encoding(_) => "Encoding Error",
            Error::Sending(_) => "Sending Error",
            Error::NoConnection => "No Connection",
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match self {
            _ => None,
        }
    }
}

pub struct BoxedConnection {
    inner: Box<dyn Connection>,
}

impl Deref for BoxedConnection {
    type Target = Box<dyn Connection>;

    fn deref(&self) -> &Box<dyn Connection> {
        &self.inner
    }
}
impl BoxedConnection {
    pub fn from_network(player_id: usize, sender: WsSender) -> BoxedConnection {
        BoxedConnection {
            inner: Box::new((player_id, sender)),
        }
    }
    pub fn from_empty(player_id: usize) -> BoxedConnection {
        BoxedConnection {
            inner: Box::new(player_id),
        }
    }
    pub fn from_name(name: &str) -> BoxedConnection {
        BoxedConnection { inner: Box::new(0) }
    }
}
pub trait Connection: Sync + Send {
    /// Called to encode and send the action.
    fn send(&self, action: &Action) -> Result;
    /// Gets the player id that ownes this connection.
    fn player_id(&self) -> usize;
    /// Called to make a manual disconnect.
    fn disconnect(&self) {}
    /// Called when the server has requested shutdown.
    fn shutdown(&self) {}
    /// Called when this conntion is getting dropped.
    /// or when closed from the other end.
    fn destroy(&self) {}
}

impl Connection for usize {
    fn send(&self, action: &Action) -> Result {
        Err(Error::NoConnection)
    }
    fn player_id(&self) -> usize {
        *self
    }
}
impl Connection for (usize, WsSender) {
    fn send(&self, action: &Action) -> Result {
        let message = action.encode().map_err(|e| Error::Encoding(0))?;
        Ok(self.1.send(message).map_err(|e| Error::Sending(1))?)
    }
    fn player_id(&self) -> usize {
        self.0
    }
    fn disconnect(&self) {
        self.1.close(CloseCode::Normal).unwrap_or(())
    }
    fn shutdown(&self) {
        self.1.shutdown().unwrap_or(())
    }
}
