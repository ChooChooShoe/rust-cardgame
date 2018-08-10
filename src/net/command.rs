use bincode::{deserialize, serialize, ErrorKind};
use game::Action;
use std::result::Result as StdResult;
use ws::{Error as WsError, ErrorKind as WsErrorKind, Message};

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Text(String),
    Unknown,
    Invalid,
    ChangePlayerId(i32, i32),
    TakeAction(Action),
    UndoAction(Action),
}

impl Command {
    pub fn encode(&self) -> StdResult<Message, WsError> {
        match self {
            Command::Text(t) => Ok(Message::Text(t.to_string())),
            _ => Ok(Message::Binary(try!(serialize(&self).map_err(
                |bincode_err| {
                    warn!("Encoding command '{:?}' to message failed: {:?}", self, bincode_err);
                    match *bincode_err {
                        ErrorKind::Io(e) => WsError::new(WsErrorKind::Io(e), ""),
                        ErrorKind::InvalidUtf8Encoding(e) => {
                            WsError::new(WsErrorKind::Encoding(e), "")
                        }
                        _ => WsError::new(WsErrorKind::Custom(bincode_err), ""),
                    }
                }
            )))),
        }
    }
    pub fn decode(msg: &Message) -> StdResult<Self, WsError> {
        match msg {
            Message::Text(t) => Ok(Command::Text(t.to_string())),
            Message::Binary(b) => deserialize(&b[..]).map_err(|bincode_err| {
                warn!("Decoding command from message failed: {:?}", bincode_err);
                match *bincode_err {
                    ErrorKind::Io(e) => WsError::new(WsErrorKind::Io(e), ""),
                    ErrorKind::InvalidUtf8Encoding(e) => WsError::new(WsErrorKind::Encoding(e), ""),
                    _ => WsError::new(WsErrorKind::Custom(bincode_err), ""),
                }
            }),
        }
    }
}
