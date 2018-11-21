use bincode::*;
use crate::entity::card::CardKey;
use crate::game::{Deck, Game, PlayerId};
use crate::net::NetError;
use std::convert::{From, Into};
use std::error::Error as StdError;
use std::fmt;
use std::result::Result as StdResult;
use ws::Message;

pub type Result = StdResult<OkCode, Error>;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum OkCode {
    /// Default value, the action has been done.
    Done,
    /// Multi-step action: step is done, next step will continue.
    Continue,
    /// Multi-step action: all steps are complete.
    Complete,
    /// This action has been skipped or did not need to be completed.
    Skip,
    /// This action will trigger a change state for core.
    ChangeState,
}

/// The type of an error.
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum Error {
    /// When an action perfroms when it is not supported.
    NotSupported,
    /// Indicates an internal prossesing error.
    Internal,
    /// Indicates an unknown error or error that was expected to happen.
    Generic,
    /// When an invalid target was given.
    InvalidTarget,
    /// When a target was needed and none were given or avalable.
    NoTarget,
    /// When the cost is too high.
    CantPayCost,
    /// When a player does an action for a diffrent player but was not allowed to.
    NotFromRightPlayer,
    /// The action had invalid parameters.
    InvalidParamaters,
    /// The action needed more parameters.
    MissingParamaters,
}

impl Error {}

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
            Error::Internal => "Internal Application Error",
            Error::Generic => "Generic Error",
            Error::InvalidTarget => "Invalid Target",
            Error::NoTarget => "No Target",
            Error::CantPayCost => "Can't Pay Cost",
            Error::NotSupported => "Not Supported",
            _ => "Unknown Error",
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match self {
            _ => None,
        }
    }
}
impl From<NetError> for Error {
    fn from(_: NetError) -> Error {
        Error::Internal
    }
}
