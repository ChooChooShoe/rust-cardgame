use crate::entity::cardpool::CardPool;
use crate::entity::trigger::{Dispatch, Trigger};
use crate::entity::{Card, Effect};
use crate::game::{
    Action, ActionResult, ActiveCardPool, Deck, Player, PlayerId, Zone, ZoneCollection,OkCode
};
use crate::net::{Connection, NetResult, NetworkMode,NetError};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};

pub struct Game {
    pub players: Vec<Player>,
    pub connections: Vec<Connection>,
    pub cards: ActiveCardPool,
    // pub effects: VecDeque<Effect>,
    action_queue: VecDeque<(PlayerId, Action)>,
    //trigger_queue: VecDeque<Trigger>,
    active_player_id: usize,
    network_mode: NetworkMode,
    pub ready_players: HashSet<PlayerId>,
    pub local_player: PlayerId,
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
            cards: ActiveCardPool::new(),
            // effects: VecDeque::new(),
            action_queue: VecDeque::new(),
            network_mode,
            ready_players: HashSet::new(),
            local_player: 0,
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
    pub fn server(&self) -> &Connection {
        &self.connections[0]
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

    pub fn active_card_pool(&mut self) -> &mut ActiveCardPool {
        &mut self.cards
    }

    pub fn queue_action(&mut self, player_id: PlayerId, action: Action) {
        self.action_queue.push_back((player_id, action))
    }
    // pub fn pop_action(&mut self) -> Option<(PlayerId, Action)> {
    //     self.action_queue.pop_front()
    // }
    // pub fn is_action_queue_empty(&self) -> bool {
    //     self.action_queue.is_empty()
    // }

    pub fn min_players(&self) -> usize {
        2
    }
    pub fn max_players(&self) -> usize {
        255
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

    /// Returns true when a state changse is needed.
    /// TODO all actions in queue are performed with this connection
    /// TODO watch for infinit loops.
    pub fn process_actions(&mut self) -> Result<bool,NetError> {
        while let Some(action) = self.action_queue.pop_front() {
            match action.1.perform(self, action.0) {
                Ok(OkCode::ChangeState) => return Ok(true),
                Ok(OkCode::Done) => (),
                Ok(code) => {
                    self.connection(action.0).send(&Action::OnResponceOk(code))?;
                }
                Err(e) => {
                    info!("action err: {:?}", e);
                    self.connection(action.0).send(&Action::OnResponceErr(e))?;
                }
            }
        }
        Ok(false)
    }

    pub fn process_triggers(&mut self) {
        
    }
}
