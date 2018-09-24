use crate::entity::card::CardId;
use crate::entity::Card;
use crate::entity::CardPool;
use crate::game::zones::ZoneName;
use crate::game::Player;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Deck {
    name: String,
    cards: Vec<Entry>,
}
impl Deck {
    pub fn new() -> Deck {
        Deck {
            name: String::from("Example"),
            cards: vec![
                Entry::new("auto_gen_card_009", 1),
                Entry::new("auto_gen_card_008", 2),
                Entry::new("auto_gen_card_007", 3),
                Entry::new("auto_gen_card_006", 4),
                Entry::new("auto_gen_card_005", 5),
                Entry::new("auto_gen_card_004", 6),
                Entry::new("auto_gen_card_003", 7),
                Entry::new("auto_gen_card_002", 8),
                Entry::new("auto_gen_card_001", 9),
            ],
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
    pub fn is_valid(&self) -> bool {
        self.cards.len() > 0
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Entry {
    card: String,
    count: usize,
}
impl Entry {
    pub fn new(card: &str, count: usize) -> Entry {
        Entry {
            card: String::from(card),
            count,
        }
    }
    pub fn card(&self) -> &str {
        &self.card
    }
    pub fn count(&self) -> usize {
        self.count
    }
}
