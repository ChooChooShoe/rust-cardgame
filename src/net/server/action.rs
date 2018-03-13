use std::fmt;
use game::action::{Act,Action,Error,Result,OkCode};
use net::server::{ServerHandle};
use game::Game;

/// The Server-side version of Act trait. Implemented for Action.
pub trait ServerAct {
    fn verify(&mut self, game: &mut Game) -> Result;
}

impl ServerAct for Action {
    fn verify(&mut self, game: &mut Game) -> Result {
        match self {
            &mut Action::SelfEndTurn => Ok(OkCode::Nothing),
            _ => Ok(OkCode::Nothing),
        }
    }
}