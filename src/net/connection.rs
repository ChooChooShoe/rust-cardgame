use crate::game::{Action, Game, Phase, Player, Turn, Zone};
use crate::net::Codec;
use std::error::Error as StdError;
use std::fmt;
use std::io;
use std::result::Result as StdResult;
use std::time::Instant;
use ws::{CloseCode, Sender as WsSender};

pub type Result = StdResult<(), Error>;
/// A simple wrapped network error
#[derive(Debug)]
pub enum Error {
    Internal,
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
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match self {
            _ => None,
        }
    }
}

pub struct Connection {
    player_id: usize,
    inner: Inner,
}
impl Connection {
    pub fn from_network(player_id: usize, sender: WsSender) -> Connection {
        Connection {
            player_id,
            inner: Inner::WebSocetPlayer(sender),
        }
    }
    pub fn from_empty(player_id: usize) -> Connection {
        Connection {
            player_id,
            inner: Inner::EmptyPlayer(),
        }
    }
    pub fn from_name(name: &str) -> Connection {
        Connection {
            player_id: 0,
            inner: Inner::LocalPlayer(String::from(name)),
        }
    }

    pub fn player_id(&self) -> usize {
        self.player_id
    }
    pub fn set_player_id(&mut self, player_id: usize) {
        self.player_id = player_id
    }

    pub fn send(&self, action: &Action) -> Result {
        self.inner.send(action)
    }

    pub fn close(&self) {
        match &self.inner {
            Inner::WebSocetPlayer(ws) => ws.close(CloseCode::Normal).unwrap_or(()),
            _ => (),
        }
    }
    pub fn shutdown(&self) {
        match &self.inner {
            Inner::WebSocetPlayer(ws) => ws.shutdown().unwrap_or(()),
            _ => (),
        }
    }
    pub fn on_close_connection(&mut self) {
        self.inner = Inner::EmptyPlayer();
    }
}

enum Inner {
    LocalPlayer(String),
    WebSocetPlayer(WsSender),
    EmptyPlayer(),
}

impl Inner {
    pub fn send(&self, action: &Action) -> Result {
        // TODO errors are not boxed.
        match self {
            Inner::WebSocetPlayer(sender) => {
                let message = action.encode().map_err(|e| Error::Encoding(0))?;
                Ok(sender.send(message).map_err(|e| Error::Sending(1))?)
            }
            Inner::EmptyPlayer() =>  {
                warn!("EmptyPlayer says: {:?}", action);
                Ok(())
            }
            Inner::LocalPlayer(s) => {
                warn!("Local Player '{}' says: {:?}", s, action);
                Ok(())
            }
        }
    }
}
