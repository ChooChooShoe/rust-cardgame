use std::fmt;
use std::result::Result as StdResult;
use std::error::Error as StdError;
use std::convert::{From, Into};

use ws::Message;

pub type Result<T> = StdResult<T, Error>;

/// The type of an error.
pub enum Error {
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
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Action Responce Error {:?}", self)
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
            &Error::Internal         => "Internal Application Error",
            &Error::Generic          => "Generic Error",
            &Error::InvalidTarget    => "Invalid Target",
            &Error::NoTarget         => "No Target",
            &Error::CantPayCost      => "Can't Pay Cost",
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match self {
            _ => None,
        }
    }
}

pub trait Action<T> : Sized + fmt::Debug + Eq {
    fn perform(&mut self) -> Result<T>;
    fn undo(&mut self) -> Result<T>;
}

#[derive(Eq,PartialEq,Debug)]
pub struct PlayerAction(String);

impl Action<String> for PlayerAction {
    fn perform(&mut self) -> Result<String> {
        Ok(self.0.to_string())
    }
    fn undo(&mut self) -> Result<String> {
        Err(Error::Generic)
    }
}

impl Into<Message> for PlayerAction {
    fn into(self) -> Message {
        Message::Text(self.0)
    }
}
impl From<Message> for PlayerAction {
    fn from(msg: Message) -> Self {
        PlayerAction(msg.into_text().unwrap())
    }
}