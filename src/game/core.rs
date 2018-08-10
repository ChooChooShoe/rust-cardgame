use entity::CardPool;
use game::zones::Location;
use game::Player;
use game::Zone;
use game::ZoneCollection;
use game::{MAX_PLAYER_COUNT, MAX_TURNS};
use net::{Connection,ConnectionVec};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, RecvError, RecvTimeoutError, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};
use utils::timer::Timer;

use entity::{Dispatch, Trigger};
use game::action::{Action, ClientAction, ServerAction};
use game::Game;
use net::NetworkMode;
use vecmap::VecMap;
use ws::{CloseCode, Error as WsError, Sender as WsSender};

// Message from clients to game loop.
pub enum Event {
    OpenConnection(usize, Connection),
    CloseConnection(usize),
    StartGame(),
    StopAndExit(),
    OnPlayerAction(usize, Action),
}

pub fn run(recv: Receiver<Event>, mode: NetworkMode, mut game: Game) {
    //assert_eq!(mode, NetworkMode::Server);

    info!("\n\nRunning core game loop. [ press Ctrl-C to exit ]\n");

    let mut connections = Vec::new();
    let mut active_player = 0;
    let mut turn_count = 0;
    let mut current_step = Step::PlayersConnecting;

    info!("Waiting for connections");
    loop {
        // Loops waiting for 0 or more Event::AddPlayers(_,_) until 1 Event::StartGame().
        match recv.recv() {
            Ok(Event::OpenConnection(id, connection)) => {
                info!("Core got connection for player #{}", id);
                connections.add_player(id, connection);
            }
            Ok(Event::CloseConnection(id)) => connections.remove_player(id),
            Ok(Event::StartGame()) => break,
            Ok(Event::StopAndExit()) => return,
            Ok(_) => warn!("All players have not connected yet! Event can not be handled."),
            Err(RecvError) => return,
        }
    }
    //connections.sort_by(|a, b| a.index().cmp(&b.index()));

    info!("Game Started");
    let _game_start_time = Instant::now();

    connections.send_all(Action::GameStart());

    game.shuffle_decks();

    loop {
        match recv.recv() {
            Ok(Event::OpenConnection(id, connection)) => {
                info!("Core got connection for player #{}", id);
                connections.add_player(id, connection);
            }
            Ok(Event::CloseConnection(id)) => connections.remove_player(id),
            Ok(Event::StartGame()) => break,
            Ok(Event::StopAndExit()) => return,
            Err(RecvError) => return,

            Ok(Event::OnPlayerAction(player_id, action)) => {
                info!("Server got Player action: {:?}, id = {}", action, player_id);
                game.action_queue.push_back(action);
                let connection = &mut connections[player_id];
                //TODO all actions in queue are performed with this connection
                //TODO watch for infinit loops.
                while let Some(action) = game.action_queue.pop_front() {
                    match ServerAction::perform(action, &mut game, connection) {
                        Ok(code) => info!("action: {:?}", code),
                        Err(e) => info!("action: {:?}", e),
                    }
                }
            }
        }
    }
}

pub fn run_client(recv: Receiver<Event>, mode: NetworkMode, mut game: Game) {
    assert_eq!(mode, NetworkMode::Client);
    let mut connection = Connection::from_empty(0);
    let mut client_id = 0;
    let mut active_player = 0;
    let mut turn_count = 0;
    let mut current_step = Step::PlayersConnecting;
    loop {
        match recv.recv() {
            Ok(Event::OpenConnection(id, new_connection)) => {
                client_id = id;
                connection = new_connection;
            }
            Ok(Event::CloseConnection(id)) => {
                current_step = Step::NoConnection;
                connection.on_connection_lost();
            },
            Ok(Event::StartGame()) => current_step = Step::GameStart,
            Ok(Event::StopAndExit()) => {
                current_step = Step::EndGame;
                return;
            }
            Err(RecvError) => return,

            Ok(Event::OnPlayerAction(player_id, action)) => {
                info!(
                    "Client got Player action: {:?}, pid = {}",
                    action, player_id
                );
                game.queue_action(action);
                //TODO all actions in queue are performed with this connection
                //TODO watch for infinit loops.
                while let Some(action) = game.action_queue.pop_front() {
                    match ClientAction::perform(action, &mut game, &mut connection) {
                        Ok(code) => info!("action: {:?}", code),
                        Err(e) => info!("action: {:?}", e),
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    PlayersConnecting,
    GameStart,
    MuliginStart,
    MuliginEnd,
    PlayerTurn(usize, u32, Phase),
    EndGame,
    NoConnection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Start,
    Draw,
    Play,
    Combat,
    End,
    Cleanup,
}

impl Step {
    pub fn get_duration(&self) -> Duration {
        match self {
            Step::PlayersConnecting => Duration::from_secs(1),
            Step::GameStart => Duration::from_millis(500),
            Step::MuliginStart => Duration::from_secs(4),
            Step::MuliginEnd => Duration::from_millis(500),
            Step::PlayerTurn(_pidx, _turn, _phase) => Duration::from_secs(2),
            Step::EndGame => Duration::from_millis(100),
            Step::NoConnection => Duration::from_secs(30),
        }
    }
}

struct StepLoop {
    next: Option<Step>,
}

impl StepLoop {
    fn new() -> StepLoop {
        StepLoop {
            next: Some(Step::GameStart),
        }
    }
    fn from(step: Step) -> StepLoop {
        StepLoop { next: Some(step) }
    }
}

// impl Iterator for StepLoop {
//     type Item = Step;

//     fn next(&mut self) -> Option<Step> {
//         let curr = self.next.clone();
//         self.next = match curr {
//             Some(Step::GameStart) => Some(Step::PlayerTurn(0, 0)),
//             Some(Step::PlayerTurn(i, t)) => {
//                 let next_pidx = (i + 1) % MAX_PLAYER_COUNT;
//                 let mut turn_count = t;
//                 if next_pidx == 0 {
//                     turn_count += 1;
//                 }
//                 if turn_count >= MAX_TURNS {
//                     Some(Step::EndGame)
//                 } else {
//                     Some(Step::PlayerTurn(next_pidx, turn_count))
//                 }
//             }
//             Some(Step::EndGame) => None,
//             None => None,
//         };
//         curr
//     }
// }
