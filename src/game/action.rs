use std::fmt;
use ws::Message;
//use byteorder::{NetworkEndian,ByteOrder};
use bincode::{serialize, deserialize,ErrorKind};
use std::result::Result as StdResult;
use std::error::Error as StdError;

pub type Result = StdResult<OkCode,Error>;

pub enum OkCode {
    EchoAction,
    Nothing,
}
pub enum Error {
    NoTarget,
    CantPayCost,
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            &Error::CantPayCost => "Can't Pay Cost",
            &Error::NoTarget => "No Target",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Action Error: {}", self.description())
    }
}
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Action Error: {:?}", self)
    }
}

pub trait Act : Sized + fmt::Debug {
    fn perform(&mut self) -> Result;
    fn undo(&mut self) -> Result;
}

#[derive(Debug,Serialize,Deserialize,PartialEq,Clone)]
pub enum Action {
    Text(String),
    Empty,
    Invalid,
    DrawCard(u64),
    PlayCard(u64)
}

impl Act for Action {
    fn perform(&mut self) -> Result {
        Ok(OkCode::EchoAction)
    }
    fn undo(&mut self) -> Result {
        Err(Error::NoTarget)
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