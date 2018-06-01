use entity::CardPool;
use game::zones::Location;
use game::GameBoard;
use game::Zone;
use game::ZoneCollection;
use game::{MAX_PLAYER_COUNT, MAX_TURNS};
use player::Controller;
use game::Player;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, RecvError, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};
use utils::timer::Timer;

use entity::{Trigger,Dispatch};
use game::action::{Act, Action, ClientAction};
use game::Game;
use net::NetworkMode;
use vecmap::VecMap;
use ws::{CloseCode, Error as WsError, Sender as WsSender};

// Message from clients to game loop.
pub enum Event {
    Connect(Box<Controller>),
    /// When we as a server got a Action from thte client.
    OnClientAction(ClientAction, usize),
    TakeAction(Action, usize),
    Disconnect(CloseCode, usize),
    WsError(WsError, usize),
}

pub fn run(recv: Receiver<Event>, mode: NetworkMode, game: Game) {
    assert_eq!(mode, NetworkMode::Server);

    let game_start_time = Instant::now();
    info!("\n\nRunning core game loop. [ press Ctrl-C to exit ]\n");

    info!("Waiting for connections");
    let mut controllers = Vec::new();

    loop {
        // Wait up to 500ms to get the first connection.
        match recv.recv_timeout(Duration::from_millis(500)) {
            Ok(Event::Connect(controller)) => {
                info!("Core got connection");

                match mode {
                    NetworkMode::Server => {
                        controllers.push(controller);

                        if controllers.len() == 2 {
                            break;
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Ok(_) => warn!("No players have connected yet! Event can not be handled."),
            Err(RecvTimeoutError::Timeout) => {
                warn!("No contollers connected before timeout.");
                return;
            }
            Err(RecvTimeoutError::Disconnected) => {
                warn!("No controllers connected before channel dropped.");
                return;
            }
        }
    }
    //connections.sort_by(|a, b| a.index().cmp(&b.index()));

    info!("Game Started");

    {
        let mut b = game.board_lock();
        b.shuffle_decks();
    }

    for c in &controllers {
        c.on_muligin_start().unwrap();
    }
    for c in &controllers {
        c.on_muligin_end().unwrap();
    }

    let mut dispatch = Dispatch::new();
    let mut trigger_queue = VecDeque::new();
    let mut active_player = 0;
    let mut turn_count = 0;

    loop {
        for event in recv.try_iter() {
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
                    let _con = &controllers[pid];
                    trigger_queue.push_back(Trigger::OnCardDrawn(6));
                }
                Event::Connect(connection) => {
                    //info!("server joined: sender = {:?}, pid = {}", sender.token(), pid)
                }
                Event::Disconnect(code, pid) => {
                    if mode == NetworkMode::Client {
                        info!("server lost: code = {:?}, pid = {}", code, pid);
                        break;
                    }
                    info!("connection lost: code = {:?}, pid = {}", code, pid)
                }
                Event::WsError(err, pid) => break,
            }
        }
        for (count, trigger) in trigger_queue.iter_mut().enumerate() {
            if count > 99 {
                warn!("100 Triggers in a row detected, stoping.");
                break;
            }
            match trigger {
                Trigger::OnCardDrawn(u) => {},
                _ => {}
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
    PlayerTurn(usize, u32),
    EndGame,
}

impl Step {
    pub fn get_duration(&self) -> Duration {
        match self {
            &Step::GameStart => Duration::new(3, 0),
            &Step::PlayerTurn(_pidx, _turn) => Duration::new(2, 0),
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
        StepLoop { next: Some(step) }
    }
}

impl Iterator for StepLoop {
    type Item = Step;

    fn next(&mut self) -> Option<Step> {
        let curr = self.next.clone();
        self.next = match curr {
            Some(Step::GameStart) => Some(Step::PlayerTurn(0, 0)),
            Some(Step::PlayerTurn(i, t)) => {
                let next_pidx = (i + 1) % MAX_PLAYER_COUNT;
                let mut turn_count = t;
                if next_pidx == 0 {
                    turn_count += 1;
                }
                if turn_count >= MAX_TURNS {
                    Some(Step::EndGame)
                } else {
                    Some(Step::PlayerTurn(next_pidx, turn_count))
                }
            }
            Some(Step::EndGame) => None,
            None => None,
        };
        curr
    }
}
