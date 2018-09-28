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
    ChangePlayerId(PlayerId, PlayerId),
    Empty,
    Invalid,
    Error,
    Ok,
    DrawCardKnown(usize, usize),
    DrawCardAnon(usize, usize),

    // Player stated actions
    SelfEndTurn,
    PlayCard(u64),
    DirectAttack(u64, u64),
    DeclareAttack(u64, u64),

    // Player responses
    EndTurn(PlayerId),

    // Sent from core
    GameStart(),
    MuliginStart(),
    MuliginEnd(),
    // from server/client
    MuliginResult {
        swap: bool,
    },

    /// Sent from server when all players are connected and game is being setup.
    BeginGameSetup(),
    /// Sent from client to tell server what deck to use.
    SetDeck(Deck),
    /// Sent from client when setup is done.
    ReadyToPlay(),

    // Action from other actions
    StartNextTurn(),

    // Client sent requests.
    RequestStateChange,
}

impl Action {
    pub fn perform(self, game: &mut Game, player_id: PlayerId) -> Result {
        if game.network_mode().is_client() {
            self.client_perform(game) // player_id is 0 for client
        } else {
            self.server_perform(game, player_id)
        }
    }
    pub fn undo(self, game: &mut Game, player_id: PlayerId) -> Result {
        Err(Error::NotSupported)
    }

    fn common_perform(self, game: &mut Game, player_id: PlayerId) -> Result {
        Err(Error::NotSupported)
    }

    fn server_perform(self, game: &mut Game, player_id: PlayerId) -> Result {
        match self {
            Action::EndTurn(p) => {
                game.players[p].draw_x_cards(1);
                game.queue_action(Action::StartNextTurn());
                Ok(OkCode::ChangeState)
            }
            Action::DrawCardAnon(pid, amount) => {
                game.players[pid].draw_x_cards(amount);
                game.queue_action(Action::EndTurn(0));
                Ok(OkCode::Continue)
            }
            Action::SetDeck(deck) => {
                if deck.is_valid() {
                    game.player(player_id).set_deck(deck);
                    Ok(OkCode::Done)
                } else {
                    Err(Error::InvalidParamaters)
                }
            }
            Action::StartNextTurn() => {
                info!("Starting Turn");
                //TODO not loop forever.
                //game.queue_action(Action::EndTurn(0));
                Ok(OkCode::Done)
            }
            Action::GameStart() => Err(Error::NotSupported),
            Action::ReadyToPlay() => {
                game.ready_players.insert(player_id);
                if game.ready_players.len() == game.players().len() {
                    // Change state when all players are ready.
                    Ok(OkCode::ChangeState) 
                } else {
                    Ok(OkCode::Done)
                }
            }
            _ => self.common_perform(game, player_id),
        }
    }
    fn server_undo(self, _game: &mut Game) -> Result {
        Err(Error::NotSupported)
    }
    fn client_perform(self, game: &mut Game) -> Result {
        match self {
            Action::GameStart() => {
                game.server().send(&Action::DrawCardAnon(0, 3)).unwrap();
                Ok(OkCode::Done)
            }
            Action::BeginGameSetup() => {
                game.server().send(&Action::SetDeck(Deck::new()))?;
                game.server().send(&Action::ReadyToPlay())?;
                Ok(OkCode::Done)
            }
            _ => self.common_perform(game, 0),
        }
    }
    fn client_undo(self, _game: &mut Game) -> Result {
        Err(Error::NotSupported)
    }
}
