use bincode::{deserialize, serialize, ErrorKind};
use game::action_result::{Error, OkCode, Result};
use game::Deck;
use game::Game;
use std::convert::{From, Into};
use std::error::Error as StdError;
use std::fmt;
use std::result::Result as StdResult;
use ws::{Error as WsError, ErrorKind as WsErrorKind, Message};

/// The common version of act impleted for all actions.
pub trait Act: Sized + fmt::Debug {
    fn perform(self, game: &Game) -> Result;
    fn undo(self, game: &Game) -> Result;
}

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
    MuliginStart(),
    MuliginEnd(),
    // from server/client
    MuliginResult{swap: bool},
}

impl Act for Action {
    fn perform(self, game: &Game) -> Result {
        match self {
            Action::EndTurn(p) => {
                game.board_lock().player_mut(p).draw_x_cards(1);
                Ok(OkCode::Nothing)
            }
            Action::DrawCardAnon(pid, amount) => {
                game.board_lock().player_mut(pid).draw_x_cards(amount);
                Ok(OkCode::Nothing)
            }
            Action::SetDeck(pid, deck) => {
                //game.board_lock().player_mut(pid).set_deck(deck);
                Ok(OkCode::Nothing)
            }
            _ => Err(Error::NotSupported),
        }
    }
    fn undo(self, game: &Game) -> Result {
        Err(Error::NotSupported)
    }
}

impl Action {
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
#[cfg(test)]
mod tests {
    use super::*;

}
