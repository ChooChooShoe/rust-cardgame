use player::PlayerId;
use std::fmt;
use std::result::Result as StdResult;
use std::error::Error as StdError;
use std::convert::{From, Into};
use bincode::{serialize,deserialize,ErrorKind};
use game::action_result::{Result,Error,OkCode};
use game::Game;
use game::Deck;
use ws::Message;

/// Actions created by the client and send to the server.
#[derive(Debug,Serialize,Deserialize)]
pub enum ClientAction {
    Invalid,
    ChatMessage(),
    ReadyForGameStart,
    MuligunResults(),
    PlayCardFromHand(),
    DeclareAttack(),
    ChoiceResults(),
    EndTurn,
}
impl ClientAction { 
    pub fn encode(self) -> Message {
        Message::Binary(serialize(&self).unwrap())
    }

    pub fn decode(msg: Message) -> Self {
        match deserialize(&msg.into_data()[..]) {
            Ok(action) => action,
            Err(boxed) => {
                warn!("Decoded invalid message: {:?}", boxed.as_ref());
                match *boxed {
                    ErrorKind::Io(_e) => ClientAction::Invalid, //Error
                    ErrorKind::InvalidUtf8Encoding(_e) => ClientAction::Invalid, // Utf8Error
                    ErrorKind::InvalidBoolEncoding(_d) => ClientAction::Invalid,
                    ErrorKind::InvalidCharEncoding => ClientAction::Invalid,
                    ErrorKind::InvalidTagEncoding(_d) => ClientAction::Invalid,
                    ErrorKind::DeserializeAnyNotSupported => ClientAction::Invalid,
                    ErrorKind::SizeLimit => ClientAction::Invalid,
                    ErrorKind::SequenceMustHaveLength => ClientAction::Invalid,
                    ErrorKind::Custom(_s) => ClientAction::Invalid
                }
            }
        }
    }
}
/// The common version of act impleted for all actions.
pub trait Act : Sized + fmt::Debug {
    fn perform(self, game: &Game) -> Result;
    fn undo(self, game: &Game) -> Result;
}

#[derive(Debug,Serialize,Deserialize)]
pub enum Action {
    Text(String),
    Empty,
    Invalid,
    Error,
    Ok,
    DrawCardKnown(usize,usize),
    DrawCardAnon(usize,usize),
    SetDeck(PlayerId,Deck),

    // Player stated actions
    SelfEndTurn,
    PlayCard(u64),
    DirectAttack(u64,u64),
    DeclareAttack(u64,u64),

    // Player responses
    EndTurn(usize),

}

impl Act for Action {
    fn perform(self, game: &Game) -> Result {
        match self {
            Action::EndTurn(p) => {
                game.board_lock().player_mut(p).draw_x_cards(1)
            }
            Action::DrawCardAnon(pid,amount) => {
                game.board_lock().player_mut(pid).draw_x_cards(amount)
            }
            Action::SetDeck(pid, deck) =>  {
                //game.board_lock().player_mut(pid).set_deck(deck);
                Ok(OkCode::Nothing)
            }
            _ => Err(Error::NotSupported)
        }
    }
    fn undo(self, game: &Game) -> Result {
        Err(Error::NotSupported)
    }
}

impl Action { 
    pub fn encode(self) -> Message {
        match self {
            Action::Text(t) => Message::Text(t),
            Action::Invalid => {
                warn!("Atempting to encode Invalid Action");
                Message::Binary(serialize(&self).unwrap())
            }
            Action::Empty => {
                warn!("Atempting to encode Empty Action");
                Message::Binary(serialize(&self).unwrap())
            }
            _ => Message::Binary(serialize(&self).unwrap())
        }
    }

    pub fn decode(msg: Message) -> Self {
        match msg {
            Message::Text(t) => Action::Text(t),
            Message::Binary(b) => match deserialize(&b[..]) {
                Ok(action) => action,
                Err(boxed) => {
                    warn!("Decoded invalid message: {:?}", boxed.as_ref());
                    match *boxed {
                        ErrorKind::Io(_e) => Action::Invalid, //Error
                        ErrorKind::InvalidUtf8Encoding(_e) => Action::Invalid, // Utf8Error
                        ErrorKind::InvalidBoolEncoding(_d) => Action::Invalid,
                        ErrorKind::InvalidCharEncoding => Action::Invalid,
                        ErrorKind::InvalidTagEncoding(_d) => Action::Invalid,
                        ErrorKind::DeserializeAnyNotSupported => Action::Invalid,
                        ErrorKind::SizeLimit => Action::Invalid,
                        ErrorKind::SequenceMustHaveLength => Action::Invalid,
                        ErrorKind::Custom(_s) => Action::Invalid
                    }
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_vec_and_back() {
        assert_eq!(Action::DrawCard(500), Action::decode(Action::DrawCard(500).encode()));
    }
}