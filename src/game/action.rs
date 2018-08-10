use bincode::{deserialize, serialize, ErrorKind};
use game::action_result::{Error, OkCode, Result};
use game::Deck;
use game::Game;
use net::Connection;
use std::convert::{From, Into};
use std::error::Error as StdError;
use std::fmt;
use std::result::Result as StdResult;
use std::time::Instant;
use ws::{Error as WsError, ErrorKind as WsErrorKind, Message};

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    Text(String),
    Empty,
    Invalid,
    Error,
    Ok,
    DrawCardKnown(usize, usize),
    DrawCardAnon(usize, usize),
    SetDeck(usize, Deck),

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
    StartNextTurn()
}
pub trait ServerAction {
    fn perform(self, game: &mut Game, client: &mut Connection) -> Result;
    fn undo(self, game: &mut Game) -> Result;
}
pub trait ClientAction {
    fn perform(self, game: &mut Game, server: &mut Connection) -> Result;
    fn undo(self, game: &mut Game) -> Result;
}
// Code for the server when a client want to do an action
impl ServerAction for Action {
    fn perform(self, game: &mut Game, _client: &mut Connection) -> Result {
        match self {
            Action::EndTurn(p) => {
                game.player_mut(p).draw_x_cards(1);
                game.queue_action(Action::StartNextTurn());
                Ok(OkCode::Nothing)
            }
            Action::DrawCardAnon(pid, amount) => {
                game.player_mut(pid).draw_x_cards(amount);
                game.queue_action(Action::EndTurn(0));
                Ok(OkCode::Nothing)
            }
            Action::SetDeck(_pid, _deck) => {
                //game.board_lock().player_mut(pid).set_deck(deck);
                Ok(OkCode::Nothing)
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
    fn perform(self, _game: &mut Game, server: &mut Connection) -> Result {
        match self {
            Action::GameStart() => {
                server.send(Action::DrawCardAnon(0,3)).unwrap();
                Ok(OkCode::Nothing)
            }
            _ => Ok(OkCode::Nothing),
        }
    }
    fn undo(self, _game: &mut Game) -> Result {
        Err(Error::NotSupported)
    }
}
impl Action {
    #[deprecated]
    pub fn encode(&self) -> StdResult<Message, WsError> {
        match self {
            Action::Text(t) => Ok(Message::Text(t.to_string())),
            _ => Ok(Message::Binary(try!(serialize(&self).map_err(
                |bincode_err| {
                    warn!("Encoded action failed: {:?}", bincode_err);
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

    #[deprecated]
    pub fn decode(msg: &Message) -> StdResult<Self, WsError> {
        match msg {
            Message::Text(t) => Ok(Action::Text(t.to_string())),
            Message::Binary(b) => deserialize(&b[..]).map_err(|bincode_err| {
                warn!("Decoded invalid message: {:?}", bincode_err);
                match *bincode_err {
                    ErrorKind::Io(e) => WsError::new(WsErrorKind::Io(e), ""),
                    ErrorKind::InvalidUtf8Encoding(e) => WsError::new(WsErrorKind::Encoding(e), ""),
                    _ => WsError::new(WsErrorKind::Custom(bincode_err), ""),
                }
            }),
        }
    }
}