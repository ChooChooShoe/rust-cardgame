use bincode::{deserialize, serialize, ErrorKind};
use crate::game::action_result::{Error, OkCode, Result};
use crate::game::{Game, Deck, Phase, Turn, CardId, ClientId, PlayerId};
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

    /// current turn is not this turn.
    SwitchTurn(Turn),
    // Player responses
    EndTurn(PlayerId),

    // Client sent requests.
    RequestStateChange,
}

impl Action {
    pub fn perform(self, game: &mut Game, sender: PlayerId) -> Result {
        if game.network_mode().is_client() {
            self.client_perform(game, sender) // sender is *not* always 0
        } else {
            self.server_perform(game, sender)
        }
    }
    pub fn undo(self, game: &mut Game, sender: PlayerId) -> Result {
        Err(Error::NotSupported)
    }

    fn common_perform(self, game: &mut Game, sender: PlayerId) -> Result {
        warn!("No implementation from {:?} w/ player #{}", self, sender);
        Err(Error::NotSupported)
    }

    fn server_perform(self, game: &mut Game, sender: PlayerId) -> Result {
        match self {
            Action::ChangePlayerId(_from, _to) => Err(Error::NotSupported),
            Action::EndTurn(p) => {
                if sender != p {
                    warn!("Player ended the fro turn")
                }
                game.players[p].draw_x_cards(1);
                //game.queue_action(sender, Action::StartNextTurn());
                Ok(OkCode::ChangeState)
            }
            Action::DrawCardAnon(pid, amount) => {
                game.players[pid].draw_x_cards(amount);
                game.queue_action(sender, Action::EndTurn(sender));
                Ok(OkCode::Continue)
            }
            Action::SetDeck(deck) => {
                if deck.is_valid() {
                    game.player(sender).set_deck(deck);
                    Ok(OkCode::Done)
                } else {
                    Err(Error::InvalidParamaters)
                }
            }
            Action::GameStart() => Err(Error::NotSupported),
            Action::ReadyToPlay() => {
                game.ready_players.insert(sender);
                if game.ready_players.len() == game.players().len() {
                    // Change state when all players are ready.
                    Ok(OkCode::ChangeState)
                } else {
                    Ok(OkCode::Done)
                }
            }
            _ => self.common_perform(game, sender),
        }
    }
    fn server_undo(self, _game: &mut Game) -> Result {
        Err(Error::NotSupported)
    }
    fn client_perform(self, game: &mut Game, sender: PlayerId) -> Result {
        match self {
            Action::ChangePlayerId(_from, to) => {
                game.local_player = to;
                Ok(OkCode::Done)
            }
            Action::GameStart() => {
                game.server().send(&Action::DrawCardAnon(0, 3)).unwrap();
                Ok(OkCode::Done)
            }
            Action::BeginGameSetup() => {
                game.server().send(&Action::SetDeck(Deck::new()))?;
                game.server().send(&Action::ReadyToPlay())?;
                Ok(OkCode::Done)
            }
            Action::SwitchTurn(turn) => {
                // if this is our turn.
                if turn.player() == game.local_player && turn.phase() == Phase::Play{
                    Connection::do_turn(game, turn);
                }
                Ok(OkCode::Done)
            }
            _ => self.common_perform(game, sender),
        }
    }
    fn client_undo(self, _game: &mut Game) -> Result {
        Err(Error::NotSupported)
    }
}
