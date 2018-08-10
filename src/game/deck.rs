use game::Player;
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
    pub fn new() -> Deck {
        Deck {
            name: String::from("Example"),
            cards: vec!(
                Entry::new("auto_gen_card_005",3),
                Entry::new("auto_gen_card_006",3),
                Entry::new("auto_gen_card_003",2),
            ),
            valid: true,
        }
    }
    pub fn cards_for_zone(&self, zone: ZoneName) -> &[Entry] {
        match zone {
            ZoneName::Deck => &self.cards[..],
            _ => &[],
        }
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

#[derive(Debug,Clone,Deserialize,Serialize)]
pub struct Entry {
    card: String,
    count: usize,
}
impl Entry {
    pub fn new(card: &str, count: usize) -> Entry {
        Entry { card: String::from(card), count }
    }
    pub fn card(&self) -> &str {
        &self.card
    }
    pub fn count(&self) -> usize {
        self.count
    }
}