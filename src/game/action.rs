use std::fmt;
use std::result::Result as StdResult;
use std::error::Error as StdError;
use std::convert::{From, Into};
use bincode::*;

use ws::Message;

pub type Result = StdResult<OkCode, Error>;

pub enum OkCode {
    Nothing,
    EchoAction,
}
/// The type of an error.
#[derive(Serialize,Deserialize)]
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
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Action Responce Error {}", self)
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
            _ => "No desciption"
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match self {
            _ => None,
        }
    }
}

/// The common version of act impleted for all actions.
pub trait Act : Sized + fmt::Debug {
    fn perform(&mut self) -> Result;
    fn undo(&mut self) -> Result;
}

#[derive(Debug,Serialize,Deserialize)]
pub enum Action {
    Text(String),
    Empty,
    Invalid,
    Error(Error),
    Ok,
    DrawCard(u64),

    // Player stated actions
    SelfEndTurn,
    PlayCard(u64),
    DirectAttack(u64,u64),
    DeclareAttack(u64,u64),

    // Player responses
    OnEndTurn(u8),

}

impl Act for Action {
    fn perform(&mut self) -> Result {
        Err(Error::NotSupported)
    }
    fn undo(&mut self) -> Result {
        Err(Error::NotSupported)
    }
}

impl Action { 
    pub fn encode(self) -> Message {
        match self {
            Action::Text(t) => Message::Text(t),
            Action::Invalid => {
                warn!("Atempting to encode Invalid Action");
                Message::Binary(serialize(&self).unwrap())
            }
            Action::Empty => {
                warn!("Atempting to encode Empty Action");
                Message::Binary(serialize(&self).unwrap())
            }
            _ => Message::Binary(serialize(&self).unwrap())
        }
    }

    pub fn decode(msg: Message) -> Self {
        match msg {
            Message::Text(t) => Action::Text(t),
            Message::Binary(b) => match deserialize(&b[..]) {
                Ok(action) => action,
                Err(boxed) => {
                    warn!("Decoded invalid message: {:?}", boxed.as_ref());
                    match *boxed {
                        ErrorKind::Io(_e) => Action::Invalid, //Error
                        ErrorKind::InvalidUtf8Encoding(_e) => Action::Invalid, // Utf8Error
                        ErrorKind::InvalidBoolEncoding(_d) => Action::Invalid,
                        ErrorKind::InvalidCharEncoding => Action::Invalid,
                        ErrorKind::InvalidTagEncoding(_d) => Action::Invalid,
                        ErrorKind::DeserializeAnyNotSupported => Action::Invalid,
                        ErrorKind::SizeLimit => Action::Invalid,
                        ErrorKind::SequenceMustHaveLength => Action::Invalid,
                        ErrorKind::Custom(_s) => Action::Invalid
                    }
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_vec_and_back() {
        assert_eq!(Action::DrawCard(500), Action::decode(Action::DrawCard(500).encode()));
    }
}