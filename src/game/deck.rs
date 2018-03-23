use player::player::Player;
use net::IntoMessage;
use game::zones::ZoneName;
use entity::card::CardId;
use std::convert::{From, Into};
use bincode::*;
use ws::Message;
use entity::Card;
use entity::CardPool;
use game::ZoneCollection;

#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct Deck {
    name: String,
    cards: Vec<Entry>,
    valid: bool,
}
impl Deck {
    pub fn cards(&self) -> &[Entry] {
        &self.cards[..]
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn valid(&self) -> bool {
        self.valid
    }
    pub fn recheck_is_valid(&self) -> bool {
        true
    }
}
impl IntoMessage for Deck {
    fn try_encode(&self) -> Result<Message> {
        Ok(Message::Binary(serialize(&self)?))
    }
    fn try_decode(msg: Message) -> Result<Self> {
        let data = msg.into_data();
        Ok(deserialize(&data[..])?)
    }
}

#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct Entry {
    pub zone: ZoneName,
    pub card: CardId,
    pub count: usize,
}
impl Entry {
    pub fn new(zone: ZoneName, card: CardId, count: usize) -> Entry {
        Entry { zone, card, count }
    }
    

}