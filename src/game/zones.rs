use std::collections::HashMap;
use card::Card;
use std::rc::Rc;
use std::fmt;

const DEF_BANISHED_SIZE : usize = 0;
const MAX_CARDS_IN_BATTLEFIELD : usize = 5;
const DEF_DECK_SIZE : usize = 30;
const DEF_LIMBO_SIZE : usize = 0;
const DEF_GRAVEYARD_SIZE : usize = 0;
const MAX_CARDS_IN_HAND : usize = 10;

pub trait Zone
{
    fn draw_top_1(&self) -> Option<Rc<Card>>;
    fn draw_bottom_1(&self) -> Option<Rc<Card>>;
    fn draw_random_1(&self) -> Option<Rc<Card>>;
    
    //Draw up to x cards from the top of the stack.
    fn draw_top_x(&self, x: usize) -> Vec<Rc<Card>>;
    //Draw up to x cards from the bottom of the stack.
    fn draw_bottom_x(&self, x: usize) -> Vec<Rc<Card>>;
    //Draw up to x cards at random from the stack.
    fn draw_random_x(&self, x: usize) -> Vec<Rc<Card>>;
}
#[derive(Debug)]
pub struct ZoneCollection
{
    pub player: u64,
    pub banished: SimpleZone,
    pub battlefield: SimpleZone,
    pub deck: SimpleZone,
    pub limbo: SimpleZone,
    pub graveyard: SimpleZone,
    pub hand: SimpleZone,
}

#[derive(Debug)]
pub struct SimpleZone(pub Vec<Rc<Card>>);

impl SimpleZone
{
    fn new() -> SimpleZone
    {
        SimpleZone(Vec::new())
    }
    fn with_capacity(size: usize) -> SimpleZone
    {
        SimpleZone(Vec::with_capacity(size))
    }
    pub fn exile_all() {}
}
impl Zone for SimpleZone {
    
    fn draw_top_1(&self) -> Option<Rc<Card>>
    {
        None
    }
    fn draw_bottom_1(&self) -> Option<Rc<Card>>
    {
        None
    }
    fn draw_random_1(&self) -> Option<Rc<Card>>{
        None
    }
    
    //Draw up to x cards from the top of the stack.
    fn draw_top_x(&self, x: usize) -> Vec<Rc<Card>>{
        Vec::new()
    }
    //Draw up to x cards from the bottom of the stack.
    fn draw_bottom_x(&self, x: usize) -> Vec<Rc<Card>>{
        Vec::new()
    }
    //Draw up to x cards at random from the stack.
    fn draw_random_x(&self, x: usize) -> Vec<Rc<Card>>{
        Vec::new()
    }
}

impl ZoneCollection
{
    pub fn new(player : u64) -> ZoneCollection {
        ZoneCollection {
            player,
            banished: SimpleZone::new(),
            battlefield: SimpleZone::new(),
            deck: SimpleZone::with_capacity(DEF_DECK_SIZE),
            limbo: SimpleZone::with_capacity(DEF_LIMBO_SIZE),
            graveyard: SimpleZone::with_capacity(DEF_GRAVEYARD_SIZE),
            hand: SimpleZone::new(),
        }
    }
}