use game::GameBoard;
use player::Player;
use player::Controller;
use entity::CardPool;
use game::Zone;
use game::ZoneCollection;
use game::zones::Location;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::thread;
use std::sync::mpsc::{Receiver,RecvError,RecvTimeoutError};
use std::time::{Duration, Instant};
use game::{MAX_PLAYER_COUNT,MAX_TURNS};
use utils::timer::Timer;

use ws::{Sender as WsSender,CloseCode, Error as WsError};
use game::action::{Act,Action,ClientAction};
use game::Game;
use net::NetworkMode;
use vecmap::VecMap;

// Message from clients to game loop.
pub enum Event {
    Connect(WsSender, u8),
    /// When we as a server got a Action from thte client.
    OnClientAction(ClientAction, u8),
    TakeAction(Action, u8),
    Disconnect(CloseCode, u8),
    WsError(WsError, u8),
}

pub enum Connection {

}
pub fn run(recv: Receiver<Event>, mode: NetworkMode, game: Game) {
    let game_start_time = Instant::now();
    info!("\n\nRunning core game loop. [ press Ctrl-C to exit ]\n");
   
    info!("Waiting for connections");
    let mut connections = VecMap::new();

    loop {
        // try every 500ms to get a connection. break when we can start game.
        match recv.recv_timeout(Duration::from_millis(500)) {
            Ok(Event::Connect(send,pid)) => {
                info!("Core got connection");
                
                match mode {
                    NetworkMode::Client => { // client makes one coonection to host/server
                        assert_eq!(pid,0); //host is always pid 0
                        connections.push(send);
                        break
                    }
                    NetworkMode::Server => {
                        connections.insert(pid as usize, send);
                    }
                    _ => {}
                }
            }
            Ok(_) => unreachable!(),
            Err(RecvTimeoutError::Timeout) => { warn!("Could not connect: Channel timeout"); return }
            Err(RecvTimeoutError::Disconnected) => { warn!("Could not connect: Channel dropped"); return }
        }
    }
    main_loop(recv, connections, mode, game)
}
fn main_loop(recv: Receiver<Event>, connections: VecMap<WsSender>, mode: NetworkMode, game: Game) {
    info!("Game Started");

    let mut active_player = 0;
    let mut turn_count = 0;

    for event in recv.into_iter() {
        match event {
            Event::TakeAction(mut a, pid) => {
                info!("PLayer action!: action = {:?}, pid = {}", a, pid);
                match a.perform(&game) {
                    Ok(code) => info!("action: {:?}", code),
                    Err(e) => info!("action: {:?}", e),
                }
            }
            Event::OnClientAction(action, pid) => {
                info!("client action!: action = {:?}, pid = {}", action, pid);
                let _con = connections.get(pid as usize).unwrap();

            }
            Event::Connect(sender, pid) => {
                info!("server joined: sender = {:?}, pid = {}", sender.token(), pid)
            }
            Event::Disconnect(code, pid) => {
                if mode == NetworkMode::Client {
                    info!("server lost: code = {:?}, pid = {}", code, pid);
                    break
                }
                info!("connection lost: code = {:?}, pid = {}", code, pid)
            }
            Event::WsError(err, pid) => {
                break
            }
        }
    }
    //setup_decks(&pool, &mut board);

    //let mut up_channels = Vec::with_capacity(MAX_PLAYER_COUNT as usize);

    //board.shuffle_decks();
    //board.run_mulligan();

    //let (response_sender, response_recv) = mpsc::channel();

    //let step_loop = StepLoop::new();
//
    //for mut step in step_loop {
    //    let timer = Timer::from_duration(step.get_duration());
    //    //info!("Start step {:?} at {} secs", step, (start_time - game_start_time).as_secs());
//
    //    match step {
    //        Step::GameStart => run_game_start(&mut up_channels, &request_recv, timer),
    //        Step::PlayerTurn(i,t) => run_turn_step(&mut up_channels[i], &request_recv, t, timer),
    //        Step::EndGame => run_game_end(timer),
    //    }
//
    //    //info!("End step: {:?} at {:+.6} ms\n", step, fmt_time_delta(end_time,Instant::now()));
//
    //    //let event_queue: VecDeque<Event> = VecDeque::new();
    //    //for event in event_queue {
    //    //    //board.controller1.do_turn(&mut board.player1, turn_count);
    //    //    //board.controller2.do_turn(&mut board.player2, turn_count);
    //    //}
    //}
}



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    GameStart,
    PlayerTurn(u8,u32),
    EndGame,
}

impl Step {
    pub fn get_duration(&self) -> Duration {
        match self {
            &Step::GameStart => Duration::new(3, 0),
            &Step::PlayerTurn(_pidx,_turn) => Duration::new(2, 0),
            &Step::EndGame => Duration::new(10, 0),
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
        StepLoop {
            next: Some(step),
        }
    }
}

impl Iterator for StepLoop {
    type Item = Step;

    fn next(&mut self) -> Option<Step> {
        let curr = self.next.clone();
        self.next = match curr {
            Some(Step::GameStart) => Some(Step::PlayerTurn(0,0)),
            Some(Step::PlayerTurn(i,t)) => {
                let next_pidx = (i + 1) % MAX_PLAYER_COUNT;
                let mut turn_count = t;
                if next_pidx == 0 {
                    turn_count += 1;
                }
                if turn_count >= MAX_TURNS {
                    Some(Step::EndGame)
                } else {
                    Some(Step::PlayerTurn(next_pidx,turn_count))
                }
            }
            Some(Step::EndGame) => None,
            None => None,
        };
        curr
    }
}