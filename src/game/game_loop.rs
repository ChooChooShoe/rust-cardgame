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
use std::sync::mpsc;
use std::time::{Duration, Instant};
use game::{MAX_PLAYER_COUNT,MAX_TURNS};
use utils::timer::Timer;

fn run_send_recv_player(pidx: usize, sender: mpsc::Sender<Request>, recv: mpsc::Receiver<Response>) {
    
    loop {
        match recv.try_recv(){
            Ok(response) => {
                info!("Got Responce: {:?}",response);
                sender.send(Request::ReadyCheck());
            },
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => return
        }
        
        //match sender.send(Request::PlayCard("1".to_string())) {
        //    Ok(()) => {} // everything good
        //    Err(e) => {
        //        println!("Err {}", e);
        //        return;
        //    } // we have been released, don't panic
        //}
        thread::sleep(Duration::new(0, 50000));
    }
}

#[derive(Debug)]
pub struct Channel {
    response_send: mpsc::Sender<Response>,
    request_recv: mpsc::Receiver<Request>,
    handle: thread::JoinHandle<()>,
    pidx: usize,
}

pub fn time_to_secs(duration: Duration) -> f64 {
    duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9
}
pub fn time_to_mills(duration: Duration) -> f64 {
    info!("duration: {:?}", duration);
    duration.as_secs() as f64 * 1e3 + duration.subsec_nanos() as f64 * 1e-6
}

// Takes two Instants and gives the time diffrance in milliseconds
pub fn fmt_time_delta(expected: Instant, actual: Instant) -> f64 {
    if actual > expected {
        time_to_mills(actual - expected)//.subsec_nanos() as f64 * 1e-6
    } else {
        time_to_mills(expected - actual)//.subsec_nanos() as f64 * -1e-6
    }
}

pub fn run(mut pool: CardPool, mut board: GameBoard) {
    let game_start_time = Instant::now();
    println!("\n\nRunning core game loop. [ press Ctrl-C to exit ]\n");

    setup_decks(&pool, &mut board);

    let mut up_channels = Vec::with_capacity(MAX_PLAYER_COUNT);
    for pidx in 0..MAX_PLAYER_COUNT {
        let (request_send, request_recv) = mpsc::channel();
        let (response_send, response_recv) = mpsc::channel();
        let handle = thread::spawn(move || run_send_recv_player(pidx, request_send, response_recv));
        up_channels.push(Channel {
            response_send,
            request_recv,
            handle,
            pidx,
        });
    }
    board.shuffle_decks();
    board.run_mulligan();

    //let (response_sender, response_recv) = mpsc::channel();

    let step_loop = StepLoop::new();

    for mut step in step_loop {
        let timer = Timer::from_duration(step.get_duration());
        //info!("Start step {:?} at {} secs", step, (start_time - game_start_time).as_secs());

        match step {
            Step::GameStart() => run_game_start(&mut up_channels, timer),
            Step::PlayerTurn(i,t) => run_turn_step(&mut up_channels[i], t, timer),
            Step::EndGame() => run_game_end(timer),
        }

        //info!("End step: {:?} at {:+.6} ms\n", step, fmt_time_delta(end_time,Instant::now()));

        //let event_queue: VecDeque<Event> = VecDeque::new();
        //for event in event_queue {
        //    //board.controller1.do_turn(&mut board.player1, turn_count);
        //    //board.controller2.do_turn(&mut board.player2, turn_count);
        //}
    }
}

fn run_game_start(channels: &mut Vec<Channel>, timer: Timer){
    info!("Game Start!");
    for chan in channels.into_iter() {
        chan.response_send.send(Response::GameStart()).expect("ReadyCheck: send channel closed");
    }
    for chan in channels.into_iter() {
        match chan.request_recv.recv_timeout(timer.time_left()) {
            Ok(req) => {
                match req {
                    Request::ReadyCheck() => {
                        info!("ReadyCheck: Player #{} is ready", chan.pidx);
                    }
                    _ => {warn!("ReadyCheck: Unexpected recv, got '{:?}' needed 'ReadyCheck'", req)}
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                warn!("ReadyCheck: Timeout")
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                warn!("ReadyCheck: Disconnected")
            }
        }
    }
}
fn run_game_end(timer: Timer){
    info!("Game End!");
    timer.wait();
}

fn run_turn_step(chan: &mut Channel, turn_count: u32, timer: Timer) {
    info!("Turn {} for Player #{}",turn_count, chan.pidx);
    loop {
        if timer.is_out_of_time() {
            info!("Turn End: Overtime");
            match chan.request_recv.try_recv() {
                Ok(s) => info!("Got in Overtime: {:?}", s),
                Err(mpsc::TryRecvError::Empty) => {
                    info!("Overtime: Empty");
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    info!("Overtime: Disconnected");
                    //unreachable!()
                }
            }
            break;
        }
        match chan.request_recv.recv_timeout(timer.time_left()) {
            Ok(s) => {
                info!("Got: {:?}", s);
                match s {
                    Request::EndTurn() => {}
                    Request::PlayCard(_) => {}
                    _ => {}
                };
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                info!("Turn End: Timeout");
                break;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                info!("Turn End: Disconnected");
                break;
            }
        }
    }
}
// Message from out comeing in.
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Request {
    EndTurn(),
    PlayCard(String),
    ReadyCheck(),
}

// Message going out.
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Response {
    GameStart(),
    TurnIsOver(),
    CreateCard(),
    DrawCard(),
}

fn setup_decks(pool: &CardPool, board: &mut GameBoard) {
    let cards_to_add = vec![
        "auto_gen_card_000",
        "auto_gen_card_001",
        "auto_gen_card_002",
        "auto_gen_card_003",
        "auto_gen_card_004",
        "auto_gen_card_005",
        "auto_gen_card_006",
        "auto_gen_card_007",
        "auto_gen_card_008",
        "auto_gen_card_009",
    ];

    for add in cards_to_add {
        let c = pool.all_cards.get(add).unwrap(); //TODO remove unwrap.

        board
            .player1
            .zones
            .deck
            .add_card(RefCell::new(c.clone()), Location::Top);
        board
            .player2
            .zones
            .deck
            .add_card(RefCell::new(c.clone()), Location::Top);
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    GameStart(),
    PlayerTurn(usize,u32),
    EndGame(),
}

impl Step {
    pub fn get_duration(&self) -> Duration {
        match self {
            &Step::GameStart() => Duration::new(3, 0),
            &Step::PlayerTurn(_pidx,_turn) => Duration::new(2, 0),
            &Step::EndGame() => Duration::new(10, 0),
        }
    }
}

struct StepLoop {
    next: Option<Step>,
}

impl StepLoop {
    fn new() -> StepLoop {
        StepLoop {
            next: Some(Step::GameStart()),
        }
    }
}

impl Iterator for StepLoop {
    type Item = Step;

    fn next(&mut self) -> Option<Step> {
        let curr = self.next.clone();
        self.next = match curr {
            Some(ref x) => match x {
                &Step::GameStart() => Some(Step::PlayerTurn(0,0)),
                &Step::PlayerTurn(i,t) => {
                    let next_pidx = (i + 1) % MAX_PLAYER_COUNT;
                    let mut turn_count = t;
                    if next_pidx == 0 {
                        turn_count += 1;
                    }
                    if turn_count >= MAX_TURNS {
                        Some(Step::EndGame())
                    } else {
                        Some(Step::PlayerTurn(next_pidx,turn_count))
                    }
                }
                &Step::EndGame() => None,
            },
            None => None,
        };
        curr
    }
}