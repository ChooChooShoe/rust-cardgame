use bincode::{deserialize, serialize, ErrorKind};
use crate::game::action_result::{Error, OkCode, Result};
use crate::game::Deck;
use crate::game::Game;
use crate::game::{CardId, ClientId, PlayerId};
use crate::net::Connection;
use std::convert::{From, Into};
use std::error::Error as StdError;
use std::fmt;
use std::result::Result as StdResult;
use std::time::Instant;
use ws::{Error as WsError, ErrorKind as WsErrorKind, Message};

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    Text(String),
    ChangePlayerId(usize, usize),
    Empty,
    Invalid,
    Error,
    Ok,
    DrawCardKnown(usize, usize),
    DrawCardAnon(usize, usize),
    SetDeck(PlayerId, Deck),

    // Player stated actions
    SelfEndTurn,
    PlayCard(u64),
    DirectAttack(u64, u64),
    DeclareAttack(u64, u64),

    // Player responses
    EndTurn(usize),

    // Sent from core
    GameStart(),
    MuliginStart(),
    MuliginEnd(),
    // from server/client
    MuliginResult { swap: bool },

    // Action from other actions
    StartNextTurn(),

    // Client sent requests.
    RequestStateChange,
}
pub trait ServerAction {
    fn perform(self, game: &mut Game, client_id: usize) -> Result;
    fn undo(self, game: &mut Game) -> Result;
}
pub trait ClientAction {
    fn perform(self, game: &mut Game) -> Result;
    fn undo(self, game: &mut Game) -> Result;
}
// Code for the server when a client want to do an action
impl ServerAction for Action {
    fn perform(self, game: &mut Game, _client_id: usize) -> Result {
        match self {
            Action::EndTurn(p) => {
                game.players[p].draw_x_cards(1);
                game.queue_action(Action::StartNextTurn());
                Ok(OkCode::Nothing)
            }
            Action::DrawCardAnon(pid, amount) => {
                game.players[pid].draw_x_cards(amount);
                game.queue_action(Action::EndTurn(0));
                Ok(OkCode::Nothing)
            }
            Action::SetDeck(player_id, deck) => {
                if deck.is_valid() {
                    game.player(player_id).set_deck(deck);
                    Ok(OkCode::Done)
                } else {
                    Err(Error::Generic)
                }
            }
            Action::StartNextTurn() => {
                info!("Starting Turn");
                //TODO not loop forever.
                //game.queue_action(Action::EndTurn(0));
                Ok(OkCode::Nothing)
            }
            Action::GameStart() => Err(Error::NotSupported),
            _ => Err(Error::NotSupported),
        }
    }
    fn undo(self, _game: &mut Game) -> Result {
        Err(Error::NotSupported)
    }
}
// Code for the client when the server wants us to act.
impl ClientAction for Action {
    fn perform(self, game: &mut Game) -> Result {
        match self {
            Action::GameStart() => {
                game.server().send(&Action::DrawCardAnon(0, 3)).unwrap();
                Ok(OkCode::Nothing)
            }
            _ => Ok(OkCode::Nothing),
        }
    }
    fn undo(self, _game: &mut Game) -> Result {
        Err(Error::NotSupported)
    }
}
