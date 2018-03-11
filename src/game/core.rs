use game::GameBoard;
use player::Player;
use player::Controller;
use entity::card::CardPool;
use game::Zone;
use game::ZoneCollection;
use game::zones::Location;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::thread;
use std::sync::mpsc::{Receiver,RecvError};
use std::time::{Duration, Instant};
use game::{MAX_PLAYER_COUNT,MAX_TURNS};
use utils::timer::Timer;

use ws::{Sender as WsSender,CloseCode};
use game::Action;

// Message from clients to game loop.
pub enum Event {
    Connect(WsSender),
    TakeAction(Action),
    Disconnect(CloseCode),
}

pub fn run(recv: Receiver<Event>) {
    let game_start_time = Instant::now();
    println!("\n\nRunning core game loop. [ press Ctrl-C to exit ]\n");

    if let Ok(Event::Connect(send)) = recv.recv() {
        info!("Core got connection");
        for event in recv.into_iter() {
            match event {
                Event::TakeAction(a) => {
                    info!("PLayer action!")
                }
                Event::Connect(sender) => {
                    info!("PLayer joined")
                }
                Event::Disconnect(code) => {
                    info!("PLayer left")
                }
            }
        }
    } else {
        warn!("First event was not a Connect event as required.");
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