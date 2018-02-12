use std::collections::HashMap;
use card::Card;
use game::ZoneCollection;
//use tags::*;
use rand::{thread_rng, Rng};
use game::player::*;
use serde::{Serialize,Deserialize};

pub struct GameBoard<P1: Player, P2: Player>
{
    pub uid: u64,
    pub player1: P1,
    pub player2: P2,
    pub p1_zones: ZoneCollection,
    pub p2_zones: ZoneCollection,
}

impl<P1: Player, P2: Player> GameBoard<P1,P2>
{
    pub fn new(uid: u64, player1: P1, player2: P2) -> GameBoard<P1,P2>
    {
        GameBoard {
            uid,
            player1,
            player2,
            p1_zones: ZoneCollection::new(1),
            p2_zones: ZoneCollection::new(2),
        }
    }

    pub fn shuffle_decks(&mut self)
    {
        let mut rng = thread_rng();

        //let mut a = self.p1_zones.deck.as_mut_slice();
        //rng.shuffle(&mut a);

        //let mut b = self.p2_zones.deck.as_mut_slice();
        //rng.shuffle(&mut b);
    }
}
