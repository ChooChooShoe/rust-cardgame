use std::result;
use std::fmt;
use ws::Message;
pub type Result = result::Result<(),()>;

pub trait Action : Sized + fmt::Debug + Eq {
    fn preform() -> Result;
    fn undo() -> Result;
}

#[derive(Eq,PartialEq,Debug)]
pub struct PlayerAction(String);

impl Action for PlayerAction{
    fn preform() -> Result {
        Ok(())
    }
    fn undo() -> Result {
        Err(())
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