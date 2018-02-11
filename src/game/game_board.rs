use std::collections::HashMap;
use card::Card;
//use tags::*;
use rand::{thread_rng, Rng};
use game::player::*;
use serde::{Serialize,Deserialize};

const DEF_BANISHED_SIZE : usize = 0;
const MAX_CARDS_IN_BATTLEFIELD : usize = 5;
const DEF_DECK_SIZE : usize = 30;
const DEF_LIMBO_SIZE : usize = 0;
const DEF_GRAVEYARD_SIZE : usize = 0;
const MAX_CARDS_IN_HAND : usize = 10;

pub struct GameBoard<P1: Player, P2: Player>
{
    pub uid: u64,
    pub player1: P1,
    pub player2: P2,
    pub p1_zones: Zones,
    pub p2_zones: Zones,
}

impl<P1: Player, P2: Player> GameBoard<P1,P2>
{
    pub fn new(uid: u64, player1: P1, player2: P2) -> GameBoard<P1,P2>
    {
        GameBoard {
            uid,
            player1,
            player2,
            p1_zones: Zones::new(1),
            p2_zones: Zones::new(2),
        }
    }

    pub fn shuffle_decks(&mut self)
    {
        let mut rng = thread_rng();

        let mut a = self.p1_zones.deck.as_mut_slice();
        rng.shuffle(&mut a);

        let mut b = self.p2_zones.deck.as_mut_slice();
        rng.shuffle(&mut b);
    }
}



//#[derive(Debug,Serialize,Deserialize)]
pub struct Zones
{
    pub player: u64,
    pub banished: Vec<Box<Card>>,
    pub battlefield: [Option<Box<Card>>; MAX_CARDS_IN_BATTLEFIELD],
    pub deck: Vec<Box<Card>>,
    pub limbo: Vec<Box<Card>>,
    pub graveyard: Vec<Box<Card>>,
    pub hand: [Option<Box<Card>>; MAX_CARDS_IN_HAND],
}

impl Zones
{
    fn new(player : u64) -> Zones {
        Zones {
            player,
            banished: Vec::with_capacity(DEF_BANISHED_SIZE),
            battlefield: Default::default(),
            deck: Vec::with_capacity(DEF_DECK_SIZE),
            limbo: Vec::with_capacity(DEF_LIMBO_SIZE),
            graveyard: Vec::with_capacity(DEF_GRAVEYARD_SIZE),
            hand: Default::default(),
        }
    }
}