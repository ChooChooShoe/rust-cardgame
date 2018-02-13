use std::collections::HashMap;
use card::Card;
use game::ZoneCollection;
//use tags::*;
use rand::{thread_rng, Rng};
use game::player::*;
use serde::{Serialize,Deserialize};
use game::Zone;

pub struct GameBoard
{
    pub uid: u64,
    pub player1: Player,
    pub player2: Player,
}

impl GameBoard
{
    pub fn new(uid: u64, player1: Player, player2: Player) -> GameBoard
    {
        GameBoard {
            uid,
            player1,
            player2,
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

    pub fn run_mulligan(&mut self)
    {
        self.player1.draw_x_cards(5);
        self.player2.draw_x_cards(5);
    }
}
