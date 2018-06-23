use entity::CardPool;
use game::zones::Location;
use game::Player;
use game::Zone;
use game::ZoneCollection;
use game::{MAX_PLAYER_COUNT, MAX_TURNS};
use player::controller::{Controller,ControllerCollection};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, RecvError, RecvTimeoutError, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};
use utils::timer::Timer;

use entity::{Dispatch, Trigger};
use game::action::{Action,ClientAction,ServerAction};
use game::Game;
use net::NetworkMode;
use vecmap::VecMap;
use ws::{CloseCode, Error as WsError, Sender as WsSender};

// Message from clients to game loop.
pub enum Event {
    Connect(Controller),
    TakeAction(Action, usize),
    Disconnect(CloseCode, usize),
    WsError(WsError, usize),
    OnShutdown(),
    ConnectionLost(usize),
}

pub fn run(recv: Receiver<Event>, mode: NetworkMode, mut game: Game) {
    assert_eq!(mode, NetworkMode::Server);

    info!("\n\nRunning core game loop. [ press Ctrl-C to exit ]\n");

    let mut controllers = Vec::new();
    let mut active_player = 0;
    let mut turn_count = 0;
    let mut current_step = Step::PlayersConnecting;

    info!("Waiting for connections");
    loop {
        // Wait for all connections.
        match recv.recv() {
            Ok(Event::Connect(controller)) => {
                info!("Core got connection");
                controllers.push(controller);
                if controllers.len() == game.max_players() {
                    break;
                }
            }
            Ok(_) => warn!("All players have not connected yet! Event can not be handled."),
            Err(RecvError) => return,
        }
    }
    //connections.sort_by(|a, b| a.index().cmp(&b.index()));

    info!("Game Started");
    let _game_start_time = Instant::now();

    controllers.send_all(&Action::GameStart());

    game.shuffle_decks();

    loop {
        match recv.recv() {
            Ok(Event::WsError(_err, _pid)) => break,
            Ok(Event::OnShutdown()) => break,
            //Err(TryRecvError::Empty) => continue,
            //Err(TryRecvError::Disconnected) => break,
            Err(RecvError) => break,

            Ok(Event::TakeAction(action, pid)) => {
                info!("Srever got Player action: {:?}, pid = {}", action, pid);
                let controller = &mut controllers[pid];
                match ServerAction::perform(action, &mut game, controller) {
                    Ok(code) => info!("action: {:?}", code),
                    Err(e) => info!("action: {:?}", e),
                }
            }
            Ok(Event::Connect(connection)) => {
                //info!("server joined: sender = {:?}, pid = {}", sender.token(), pid)
            }
            Ok(Event::ConnectionLost(_pid)) => break,
            Ok(Event::Disconnect(code, pid)) => {
                if mode == NetworkMode::Client {
                    info!("server lost: code = {:?}, pid = {}", code, pid);
                    break;
                }
                info!("connection lost: code = {:?}, pid = {}", code, pid)
            }
        }
    }
}

pub fn run_client(recv: Receiver<Event>, mode: NetworkMode, mut game: Game) {
    assert_eq!(mode, NetworkMode::Client);
    let mut controller = match recv.recv() {
        Ok(Event::Connect(conn)) => conn,
        _ => {
            warn!("Client did not connect to controller");
            return;
        }
    };
    loop {
        match recv.recv() {
            Err(RecvError) => break,
            Ok(Event::WsError(_err, _pid)) => break,
            Ok(Event::OnShutdown()) => break,

            Ok(Event::TakeAction(action, pid)) => {
                info!("Client got Player action: {:?}, pid = {}", action, pid);
                match ClientAction::perform(action, &mut game, &mut controller) {
                    Ok(code) => info!("action: {:?}", code),
                    Err(e) => info!("action: {:?}", e),
                }
            }
            Ok(Event::Connect(_new_conn)) => break,
            Ok(Event::ConnectionLost(_pid)) => break,
            Ok(Event::Disconnect(code, pid)) => {
                info!(
                    "Client lost connection to server: code = {:?}, pid = {}",
                    code, pid
                );
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
