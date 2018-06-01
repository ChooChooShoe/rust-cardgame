use entity::trigger::Dispatch;
use entity::trigger::Trigger;
use entity::Card;
use game::ZoneCollection;
use player::Controller;
use std::collections::HashMap;
//use tags::*;
use game::Zone;
use net::Networked;
use game::Player;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

pub struct GameBoard {
    pub players: Vec<Player>,
    active_player_pidx: usize,
    trigger_callbacks: Vec<Box<Fn(&mut GameBoard, &mut Trigger) + Send>>,
}

impl GameBoard {
    pub fn new(_uid: u64, player1: Player, player2: Player) -> GameBoard {
        GameBoard {
            players: vec![player1, player2],
            active_player_pidx: 0,
            trigger_callbacks: Vec::new(),
        }
    }
    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    pub fn players(&self) -> &[Player] {
        &self.players[..]
    }

    pub fn players_mut(&mut self) -> &mut [Player] {
        &mut self.players[..]
    }

    pub fn player(&self, index: usize) -> &Player {
        &self.players[index]
    }

    pub fn player_mut(&mut self, index: usize) -> &mut Player {
        &mut self.players[index]
    }

    pub fn set_active_player(&mut self, pidx: usize) {
        self.active_player_pidx = pidx
    }

    pub fn active_player_index(&self) -> usize {
        self.active_player_pidx
    }

    pub fn active_player(&self) -> &Player {
        &self.players[self.active_player_pidx]
    }

    pub fn active_player_mut(&mut self) -> &mut Player {
        &mut self.players[self.active_player_pidx]
    }

    pub fn shuffle_decks(&mut self) {
        //let mut rng = thread_rng();

        //let mut a = self.p1_zones.deck.as_mut_slice();
        //rng.shuffle(&mut a);

        //let mut b = self.p2_zones.deck.as_mut_slice();
        //rng.shuffle(&mut b);
    }

    pub fn run_mulligan(&mut self) {
        self.player_mut(0).draw_x_cards(5);
        self.player_mut(1).draw_x_cards(5);
    }
}
