use game::action::Action;
use entity::cardpool::CardPool;
use entity::trigger::Dispatch;
use entity::trigger::Trigger;
use entity::Card;
use game::ZoneCollection;
use player::Controller;
use std::collections::HashMap;
//use tags::*;
use game::Player;
use game::Zone;
use net::Networked;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub struct Game {
    pub players: Vec<Player>,
    active_player_pidx: usize,
    battlefield: Vec<Card>,
    pub action_queue: VecDeque<Action>,
}

impl Game {
    pub fn new(player_count: usize) -> Game {
        let mut players = Vec::with_capacity(player_count);
        for x in 0..player_count {
            players.push(Player::new(x, format!("Player #{}", x + 1)));
        }
        Game {
            players,
            active_player_pidx: 0,
            battlefield: Vec::new(),
            action_queue: VecDeque::new(),
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
    pub fn max_players(&self) -> usize {
        self.players.len()
    }

    pub fn shuffle_decks(&mut self) {
        //let mut rng = thread_rng();
        for p in self.players_mut() {
            p.set_deck(::game::Deck::new());
        }

        //let mut a = self.p1_zones.deck.as_mut_slice();
        //rng.shuffle(&mut a);

        //let mut b = self.p2_zones.deck.as_mut_slice();
        //rng.shuffle(&mut b);
    }

    pub fn battlefield(&self) -> &[Card] {
        &self.battlefield
    }
    pub fn battlefield_mut(&mut self) -> &mut [Card] {
        &mut self.battlefield
    }

    pub fn run_mulligan(&mut self) {
        self.player_mut(0).draw_x_cards(5);
        self.player_mut(1).draw_x_cards(5);
    }

    pub fn queue_action(&mut self, action: Action) {
        self.action_queue.push_back(action)
    }

    pub fn is_action_queue_empty(&self) -> bool {
        self.action_queue.is_empty()
    }
}
