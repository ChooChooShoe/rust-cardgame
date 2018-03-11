use std::fmt;
use game::action::{Act,Action,Error,Result,OkCode};
use net::server::{ServerHandle,Stage};

/// The Server-side version of Act trait. Implemented for Action.
pub trait ServerAct {
    fn perform(&mut self, stage: &mut Stage) -> Result;
    fn undo(&mut self, stage: &mut Stage) -> Result;
}

impl ServerAct for Action {
    fn perform(&mut self, stage: &mut Stage) -> Result {
        match self {
            &mut Action::SelfEndTurn => Ok(OkCode::Nothing),
            _ => Ok(OkCode::Nothing),
        }
    }
    fn undo(&mut self, stage: &mut Stage) -> Result {
        Ok(OkCode::Nothing)
    }
}