use std::fmt;
use game::action::{Act,Action,Error,Result,OkCode};

/// The Client-side version of Act trait. Implemented for Action.
pub trait ClientAct : Sized + fmt::Debug {
    fn perform(&mut self) -> Result;
    fn undo(&mut self) -> Result;
}


impl ClientAct for Action {
    fn perform(&mut self) -> Result {
        match self {
            &mut Action::SelfEndTurn => Ok(OkCode::EchoAction),
            _ => Ok(OkCode::EchoAction)
        }
    }
    fn undo(&mut self) -> Result {
        Err(Error::NoTarget)
    }
}