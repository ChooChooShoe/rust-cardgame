use crate::entity::cardpool::CardPool;
use crate::entity::trigger::{Dispatch, Trigger};
use crate::entity::{Card, Effect};
use crate::game::{Action, ActionResult, Deck, Player, PlayerId, Zone, ZoneCollection};
use crate::net::{Connection, NetResult, NetworkMode};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};

pub struct Game {
    pub players: Vec<Player>,
    pub connections: Vec<Connection>,
    pub battlefield: Vec<Card>,
    pub effects: VecDeque<Effect>,
    pub action_queue: VecDeque<Action>,
    active_player_id: usize,
    network_mode: NetworkMode,
    pub ready_players: HashSet<PlayerId>,
}

impl Game {
    pub fn new(player_count: usize, network_mode: NetworkMode) -> Game {
        let mut players = Vec::with_capacity(player_count);
        let mut connections = Vec::with_capacity(player_count);
        for x in 0..player_count {
            players.push(Player::new(x, format!("Player #{}", x + 1)));
            connections.push(Connection::from_empty(x));
        }
        Game {
            players,
            connections,
            active_player_id: 0,
            battlefield: Vec::new(),
            effects: VecDeque::new(),
            action_queue: VecDeque::new(),
            network_mode,
            ready_players: HashSet::new(),
        }
    }
    /// Gets which of Server, Client, or Both that this game is running as.
    pub fn network_mode(&self) -> NetworkMode {
        self.network_mode
    }

    pub fn connections(&mut self) -> &mut [Connection] {
        &mut self.connections
    }
    /// Gets the server's connection for player with id.
    pub fn connection(&mut self, id: usize) -> &mut Connection {
        &mut self.connections[id]
    }
    /// Gets connection to server for the client.
    pub fn server(&mut self) -> &mut Connection {
        &mut self.connections[0]
    }
    pub fn players(&mut self) -> &mut [Player] {
        &mut self.players
    }
    pub fn player(&mut self, id: usize) -> &mut Player {
        &mut self.players[id]
    }
    pub fn set_active_player(&mut self, player_id: usize) {
        self.active_player_id = player_id
    }
    pub fn active_player_id(&self) -> usize {
        self.active_player_id
    }
    pub fn active_player(&mut self) -> &mut Player {
        &mut self.players[self.active_player_id]
    }

    pub fn battlefield(&mut self) -> &mut [Card] {
        &mut self.battlefield
    }

    pub fn queue_action(&mut self, action: Action) {
        self.action_queue.push_back(action)
    }
    pub fn pop_action(&mut self) -> Option<Action> {
        self.action_queue.pop_front()
    }
    pub fn is_action_queue_empty(&self) -> bool {
        self.action_queue.is_empty()
    }

    pub fn shuffle_decks(&mut self) {
        let mut rng = thread_rng();
        for p in self.players() {
            p.set_deck(crate::game::Deck::new());
            let mut deck = p.zones.deck.as_mut_slice();
            rng.shuffle(&mut deck);
        }
    }

    pub fn run_mulligan(&mut self) {
        for p in self.players() {
            p.draw_x_cards(5);
        }
    }

    pub fn send_all_action(&mut self, action: &Action) {
        for conn in self.connections() {
            match conn.send(action) {
                Ok(_) => (),
                Err(_) => break,
            }
        }
    }
    // Sends a game action to the player over their connection.
    pub fn send_action(&mut self, client_id: usize, action: &Action) -> NetResult {
        self.connection(client_id).send(action)
    }
}
