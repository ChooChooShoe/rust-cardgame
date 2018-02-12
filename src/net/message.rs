use std::error;
use std::fmt;

pub trait Receiver {
    fn on_message_receive(&self, Message) -> Result<(),MsgError>;

}

#[derive(Debug)]
pub enum Message {
    Simple(i32)
}


#[derive(Debug, Clone)]
pub enum MsgError {
}

impl error::Error for MsgError {
    fn description(&self) -> &str {
        "unknown message result error"
    }
    fn cause(&self) -> Option<&error::Error> {
        None
    }
}
impl fmt::Display for MsgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unknown message result error")
    }
}