use crate::game::PlayerId;
use crate::entity::CardKey;

const DEF_BANISHED_SIZE: usize = 0;
const DEF_BATTLEFIELD_SIZE: usize = 5;
const DEF_DECK_SIZE: usize = 30;
const DEF_LIMBO_SIZE: usize = 0;
const DEF_GRAVEYARD_SIZE: usize = 0;
const DEF_HAND_SIZE: usize = 10;

const MAX_BANISHED_SIZE: usize = 1000;
const MAX_BATTLEFIELD_SIZE: usize = 25;
const MAX_DECK_SIZE: usize = 1000;
const MAX_LIMBO_SIZE: usize = 1000;
const MAX_GRAVEYARD_SIZE: usize = 1000;
const MAX_HAND_SIZE: usize = 10;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ZoneName {
    Banished,
    Battlefield,
    Deck,
    Limbo,
    Graveyard,
    Hand,
}
pub trait Zone<T> {
    // Inserts value at this location
    fn insert_at(&mut self, location: Location, element: T) -> &mut T;
    fn insert_all_at(&mut self, location: Location, element: Vec<T>);
    // Removes value at this location and return it.
    fn remove_at(&mut self, location: Location) -> Option<T>;
    fn remove_x_at(&mut self, count: usize, location: Location) -> Vec<Option<T>>;
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Location {
    /// The location is chosen by the zone. 
    Default,
    /// Top of deck, same as Index(len()) or pop()/push()
    Top,
    /// Bottom of deck, same as Index(0)
    Bottom,
    /// Insert at top then suffle. (only for Deck) 
    Shuffle,
    /// Insert at or draw from a random location, no suffle.
    Random,
    /// Indexes from bottom (0) to top (len()). If usize > len() wil not panic and push to the top.
    Index(usize),
}

#[derive(Clone, Debug)]
pub struct ZoneCollection {
    pub player: PlayerId,
    pub banished: Vec<CardKey>,
    pub battlefield: Vec<CardKey>,
    pub deck: Vec<CardKey>,
    pub limbo: Vec<CardKey>,
    pub graveyard: Vec<CardKey>,
    pub hand: Vec<CardKey>,
}

impl Zone<CardKey> for Vec<CardKey> {
    // Inserts value at this location
    fn insert_at(&mut self, location: Location, element: CardKey) -> &mut CardKey {
        match location {
            Location::Default => {
                self.push(element);
                self.last_mut().unwrap()
            }
            Location::Top => {
                self.push(element);
                self.last_mut().unwrap()
            }
            Location::Bottom => {
                self.insert(0, element);
                self.first_mut().unwrap()
            }
            Location::Random => {
                self.push(element);
                self.last_mut().unwrap()
            }
            Location::Shuffle => {
                self.push(element);
                self.last_mut().unwrap()
            }
            Location::Index(index) => {
                let i = Ord::min(index, self.len());
                self.insert(i, element);
                self.get_mut(i).unwrap()
            }
        }
    }
    // Removes value at this location and return it.
    fn remove_at(&mut self, location: Location) -> Option<CardKey> {
        match location {
            Location::Default => self.pop(),
            Location::Top => self.pop(),
            Location::Bottom => {
                if self.len() == 0 {
                    None
                } else {
                    Some(self.remove(0))
                }
            }
            Location::Shuffle => self.pop(),
            Location::Random => self.pop(),
            Location::Index(index) => {
                if index >= self.len() {
                    None
                } else {
                    Some(self.remove(index))
                }
            }
        }
    }

    fn insert_all_at(&mut self, location: Location, cards: Vec<CardKey>) {
        for card in cards {
            self.insert_at(location, card);
        }
    }

    fn remove_x_at(&mut self, count: usize, location: Location) -> Vec<Option<CardKey>> {
        let mut vec = Vec::with_capacity(count);
        for _ in 0..count {
            vec.push(self.remove_at(location));
        }
        vec
    }
}

impl ZoneCollection {
    pub fn new(player: PlayerId) -> ZoneCollection {
        ZoneCollection {
            player,
            banished: Vec::with_capacity(DEF_BANISHED_SIZE),
            battlefield: Vec::with_capacity(DEF_BATTLEFIELD_SIZE),
            deck: Vec::with_capacity(DEF_DECK_SIZE),
            limbo: Vec::with_capacity(DEF_LIMBO_SIZE),
            graveyard: Vec::with_capacity(DEF_GRAVEYARD_SIZE),
            hand: Vec::with_capacity(DEF_HAND_SIZE),
        }
    }
    pub fn get_mut(&mut self, zone: ZoneName) -> &mut Zone<CardKey> {
        match zone {
            ZoneName::Banished => &mut self.banished,
            ZoneName::Battlefield => &mut self.battlefield,
            ZoneName::Deck => &mut self.deck,
            ZoneName::Limbo => &mut self.limbo,
            ZoneName::Graveyard => &mut self.graveyard,
            ZoneName::Hand => &mut self.hand,
        }
    }
    pub fn get(&self, zone: ZoneName) -> &Zone<CardKey> {
        match zone {
            ZoneName::Banished => &self.banished,
            ZoneName::Battlefield => &self.battlefield,
            ZoneName::Deck => &self.deck,
            ZoneName::Limbo => &self.limbo,
            ZoneName::Graveyard => &self.graveyard,
            ZoneName::Hand => &self.hand,
        }
    }
}
