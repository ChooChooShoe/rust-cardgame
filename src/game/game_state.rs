use crate::utils::timer::Timer;
use crate::entity::{Card, Effect};
use crate::game::action::Actor;
use crate::game::{
    Action, ActionResult, ActiveCardPool, Deck, OkCode, Player, PlayerId, Zone, ZoneCollection,
};
use crate::net::{Connection, NetError, NetResult, NetworkMode};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};

pub struct GameSettings {
    local_player_id: usize,
    max_players: usize,
    network_mode: NetworkMode,
}
impl GameSettings {
    pub fn new(local_player_id: usize, max_players: usize, network_mode: NetworkMode) -> GameSettings {
        GameSettings {
            local_player_id, 
            max_players, 
            network_mode
        }
    }
}

pub struct Game {
    pub players: Vec<Player>,
    pub connections: Vec<Connection>,
    active_player_id: PlayerId,
    pub local_player_id: usize,
    pub cards: ActiveCardPool,
    // action_queue: VecDeque<(PlayerId, Action)>,
    pub stack: VecDeque<Effect>,
    network_mode: NetworkMode,
    pub ready_players: HashSet<PlayerId>,
    pub timer: Timer,
}

impl Game {
    pub fn new(settings: &GameSettings) -> Game {
        let mut players = Vec::with_capacity(settings.max_players);
        let mut connections;

        match settings.network_mode {
            NetworkMode::Client => {
                connections = vec![Connection::from_empty(0)];
                for x in 0..settings.max_players {
                    if settings.local_player_id == x {
                        players.push(Player::new(x, format!("Local Player #{}", x + 1)));
                    } else {
                        players.push(Player::new(x, format!("Remote Player #{}", x + 1)));
                    }
                }
            }
            NetworkMode::Server => {
                connections = Vec::with_capacity(settings.max_players);
                for x in 0..settings.max_players {
                    players.push(Player::new(x, format!("Player #{}", x + 1)));
                    connections.push(Connection::from_empty(x));
                }
            }
        }

        Game {
            players,
            connections,
            active_player_id: 0,
            local_player_id: settings.local_player_id,
            cards: ActiveCardPool::new(),
            stack: VecDeque::new(),
            // action_queue: VecDeque::new(),
            network_mode: settings.network_mode,
            ready_players: HashSet::new(),
            timer: Timer::default(),
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
        // self.action_queue.push_back((player_id, action))
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

    pub fn send_all_action(&mut self, action: &Action) -> NetResult<()> {
        for conn in self.connections() {
            match conn.send(action) {
                Ok(_) => (),
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
    // Sends a game action to the player over their connection.
    pub fn send_action(&mut self, client_id: usize, action: &Action) -> NetResult<()> {
        self.connection(client_id).send(action)
    }

    /// Returns true when a state changse is needed.
    /// TODO all actions in queue are performed with this connection
    /// TODO watch for infinit loops.
    pub fn process_actions(&mut self) -> Result<bool, NetError> {
        // while let Some(action) = self.action_queue.pop_front() {
        //     match action.1.perform(self, Actor::User(action.0)) {
        //         Ok(OkCode::ChangeState) => return Ok(true),
        //         Ok(OkCode::Done) => (),
        //         Ok(code) => {
        //             self.connection(action.0)
        //                 .send(&Action::OnResponceOk(code))?;
        //         }
        //         Err(e) => {
        //             info!("action err: {:?}", e);
        //             self.connection(action.0).send(&Action::OnResponceErr(e))?;
        //         }
        //     }
        // }
        Ok(false)
    }

    pub fn process_triggers(&mut self) {}
}
