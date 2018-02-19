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

pub trait Zone
{
    fn add_card(&mut self, RefCell<Card>, Location);
    fn add_cards(&mut self, Vec<RefCell<Card>>, Location);
    fn take_card(&mut self, Location) -> Option<RefCell<Card>>;
    fn take_x_cards(&mut self, x: usize, Location) -> Vec<Option<RefCell<Card>>>;
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

#[derive(Debug)]
pub struct ZoneCollection
{
    pub player: u64,
    pub banished: Vec<RefCell<Card>>,
    pub battlefield: Vec<RefCell<Card>>,
    pub deck: Vec<RefCell<Card>>,
    pub limbo: Vec<RefCell<Card>>,
    pub graveyard: Vec<RefCell<Card>>,
    pub hand: Vec<RefCell<Card>>,
}

impl Zone for Vec<RefCell<Card>> {

    fn add_card(&mut self, card: RefCell<Card>, location: Location)
    {
        location.insert(self, card)
    }

    fn add_cards(&mut self, cards: Vec<RefCell<Card>>, location: Location)
    {
        for c in cards
        {
            location.insert(self, c);
        }
    }

    fn take_card(&mut self, location: Location) -> Option<RefCell<Card>>
    {
        location.remove(self)
    }

    fn take_x_cards(&mut self, x: usize, location: Location) -> Vec<Option<RefCell<Card>>>
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