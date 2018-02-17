use std::collections::HashMap;
use card::Card;
use player::Controller;
use game::ZoneCollection;
//use tags::*;
use rand::{thread_rng, Rng};
use player::Player;
use serde::{Serialize,Deserialize};
use game::Zone;
use net::Networked;

pub struct GameBoard
{
    pub uid: u64,
    pub player1: Player,
    pub player2: Player,
    active_player_pidx: usize,
}

impl GameBoard
{
    pub fn new(uid: u64, player1: Player, player2: Player) -> GameBoard
    {
        GameBoard {
            uid,
            player1,
            player2,
            active_player_pidx: 0,
        }
    }
    pub fn player_count(&self) -> usize {2}

    pub fn player(&self, idx: usize) -> &Player {
        match idx {
            1 => &self.player1,
            _ => &self.player2,
        }
    }

    pub fn player_mut(&mut self, idx: usize) -> &mut Player {
        match idx {
            1 => &mut self.player1,
            _ => &mut self.player2,
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

    pub fn set_active_player(&mut self, pidx: usize)
    {
        self.active_player_pidx = pidx
    }
    pub fn active_player(&mut self) -> usize
    {
        self.active_player_pidx
    }
    pub fn active_player_mut(&mut self) -> &mut Player{
        match self.active_player_pidx {
            1 => &mut self.player1,
            _ => &mut self.player2,
        }
    }
}
