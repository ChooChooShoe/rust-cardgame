use bincode::*;
use std::convert::{From, Into};
use std::error::Error as StdError;
use std::fmt;
use std::result::Result as StdResult;

use game::Deck;
use game::Game;
use ws::Message;

pub type Result = StdResult<OkCode, Error>;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum OkCode {
    Nothing,
    EchoAction,
}
/// The type of an error.
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum Error {
    /// When an action perfroms when it is not supported.
    NotSupported,
    /// Indicates an internal prossesing error.
    Internal(String),
    /// Indicates an unknown error or error that was expected to happen.
    Generic,
    /// When an invalid target was given.
    InvalidTarget,
    /// When a target was needed and none were given or avalable.
    NoTarget,
    /// When the cost is too high.
    CantPayCost,
}

impl Error {
    pub fn from<T: StdError>(e: T) -> Error {
        Error::Internal(e.description().to_string())
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Action Error {}", self)
    }
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
            Error::Internal(s) => "Internal Application Error",
            Error::Generic => "Generic Error",
            Error::InvalidTarget => "Invalid Target",
            Error::NoTarget => "No Target",
            Error::CantPayCost => "Can't Pay Cost",
            Error::NotSupported => "Not Supported",
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match self {
            _ => None,
        }
    }
}
