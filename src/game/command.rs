use std::fmt;
use ws::Message;
use bincode::{serialize, deserialize,ErrorKind};
use std::result::Result as StdResult;
use std::error::Error as StdError;

pub type Result = StdResult<OkCode,Error>;

pub enum OkCode {
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
        write!(f, "Command Error: {}", self.description())
    }
}
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Command Error: {:?}", self)
    }
}

#[derive(Debug,Serialize,Deserialize,PartialEq,Clone)]
pub enum Command {
    Text(String),
    Empty,
    Invalid,
    DrawCard(u64),
    PlayCard(u64)
}

impl Command {
    fn perform(&mut self) -> Result {
        Ok(OkCode::Nothing)
    }
    fn undo(&mut self) -> Result {
        Err(Error::NoTarget)
    }

    pub fn encode(self) -> Message {
        match self {
            Command::Text(t) => Message::Text(t),
            Command::Invalid => {
                warn!("Atempting to encode Invalid Command");
                Message::Binary(serialize(&self).unwrap())
            }
            Command::Empty => {
                warn!("Atempting to encode Empty Command");
                Message::Binary(serialize(&self).unwrap())
            }
            _ => Message::Binary(serialize(&self).unwrap())
        }
    }

    pub fn decode(msg: Message) -> Self {
        match msg {
            Message::Text(t) => Command::Text(t),
            Message::Binary(b) => match deserialize(&b[..]) {
                Ok(cmd) => cmd,
                Err(boxed) => {
                    warn!("Decoded invalid message: {:?}", boxed.as_ref());
                    match *boxed {
                        ErrorKind::Io(_e) => Command::Invalid, //Error
                        ErrorKind::InvalidUtf8Encoding(_e) => Command::Invalid, // Utf8Error
                        ErrorKind::InvalidBoolEncoding(_d) => Command::Invalid,
                        ErrorKind::InvalidCharEncoding => Command::Invalid,
                        ErrorKind::InvalidTagEncoding(_d) => Command::Invalid,
                        ErrorKind::DeserializeAnyNotSupported => Command::Invalid,
                        ErrorKind::SizeLimit => Command::Invalid,
                        ErrorKind::SequenceMustHaveLength => Command::Invalid,
                        ErrorKind::Custom(_s) => Command::Invalid
                    }
                }
            }
        }
    }
}