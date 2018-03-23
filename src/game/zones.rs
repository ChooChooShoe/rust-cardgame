use std::collections::HashMap;
use entity::Card;
use std::rc::Rc;
use std::fmt;
use std::cell::RefCell;

const DEF_BANISHED_SIZE : usize = 0;
const MAX_CARDS_IN_BATTLEFIELD : usize = 5;
const DEF_DECK_SIZE : usize = 30;
const DEF_LIMBO_SIZE : usize = 0;
const DEF_GRAVEYARD_SIZE : usize = 0;
const MAX_CARDS_IN_HAND : usize = 10;

#[derive(Debug,Clone,Deserialize,Serialize)]
pub enum ZoneName {
    Banished,
    Battlefield,
    Deck,
    Limbo,
    Graveyard,
    Hand,
}
impl ZoneName {
    pub fn match_zone<'a>(&self, zones: &'a mut ZoneCollection) -> &'a mut Zone {
        match self {
            &ZoneName::Banished => &mut zones.banished,
            &ZoneName::Battlefield => &mut zones.battlefield,
            &ZoneName::Deck => &mut zones.deck,
            &ZoneName::Limbo => &mut zones.limbo,
            &ZoneName::Graveyard => &mut zones.graveyard,
            &ZoneName::Hand => &mut zones.hand,
        }
    }
}
pub trait Zone
{
    fn push(&mut self, Card);
    fn add_card(&mut self, Card, Location);
    fn add_cards(&mut self, Vec<Card>, Location);
    fn take_card(&mut self, Location) -> Option<Card>;
    fn take_x_cards(&mut self, x: usize, Location) -> Vec<Option<Card>>;
}

pub enum Location {Top,Bottom,Random}

impl Location
{
    // Insert value at this location
    pub fn insert<T>(&self, vec: &mut Vec<T>, val: T) 
    {
        match self {
            &Location::Top => vec.push(val),
            &Location::Bottom => vec.insert(0,val),
            &Location::Random => vec.push(val),
        }
    }
    // Remove value at this location
    pub fn remove<T>(&self, vec: &mut Vec<T>) -> Option<T>
    {
        match self {
            &Location::Top => vec.pop(),
            &Location::Bottom => Some(vec.remove(0)),
            &Location::Random => vec.pop(),
        }
    }
}

#[derive(Debug,Clone)]
pub struct ZoneCollection
{
    pub player: u64,
    pub banished: Vec<Card>,
    pub battlefield: Vec<Card>,
    pub deck: Vec<Card>,
    pub limbo: Vec<Card>,
    pub graveyard: Vec<Card>,
    pub hand: Vec<Card>,
}

impl Zone for Vec<Card> {

    fn push(&mut self, card: Card) {
        Vec::push(self, card)
    }
    fn add_card(&mut self, card: Card, location: Location)
    {
        location.insert(self, card)
    }

    fn add_cards(&mut self, cards: Vec<Card>, location: Location)
    {
        for c in cards
        {
            location.insert(self, c);
        }
    }

    fn take_card(&mut self, location: Location) -> Option<Card>
    {
        location.remove(self)
    }

    fn take_x_cards(&mut self, x: usize, location: Location) -> Vec<Option<Card>>
    {
        let mut vec = Vec::with_capacity(x);
        for _ in 0..x {
            vec.push(location.remove(self));
        }
        vec
    }
}


impl ZoneCollection
{
    pub fn new(player : u64) -> ZoneCollection {
        ZoneCollection {
            player,
            banished: Vec::new(),
            battlefield: Vec::new(),
            deck: Vec::with_capacity(DEF_DECK_SIZE),
            limbo: Vec::with_capacity(DEF_LIMBO_SIZE),
            graveyard: Vec::with_capacity(DEF_GRAVEYARD_SIZE),
            hand: Vec::new(),
        }
    }
}