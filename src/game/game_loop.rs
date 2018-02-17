use game::GameBoard;
use player::Player;
use player::Controller;
use card::CardPool;
use game::Zone;
use game::ZoneCollection;
use game::zones::Location;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::thread;
use std::sync::mpsc;
use std::time::{Duration, Instant};

fn run_send_recv_player(sender: mpsc::Sender<Request>, recv: mpsc::Receiver<Response>) {
    loop {
        match sender.send(Request::PlayCard("1".to_string())) {
            Ok(()) => {} // everything good
            Err(e) => {
                println!("Err {}", e);
                return;
            } // we have been released, don't panic
        }
        thread::sleep(Duration::new(1, 0));
        match sender.send(Request::EndTurn()) {
            Ok(()) => {} // everything good
            Err(e) => {
                println!("Err 2 {}", e);
                return;
            } // we have been released, don't panic
        }
        //drop(sender);
        //break;
    }
}

#[derive(Debug)]
pub struct Channel
{
    response_send: mpsc::Sender<Response>,
    request_recv: mpsc::Receiver<Request>,
    handle: thread::JoinHandle<()>,
}

pub fn fmt_time(duration: Duration) -> f64
{
    duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9
}

pub fn run(mut pool: CardPool, mut board: GameBoard) {
    let game_start_time = Instant::now();
    println!("\n\nRunning core game loop. [ press Ctrl-C to exit ]\n");

    setup_decks(&pool, &mut board);

    let mut up_channels = Vec::with_capacity(board.player_count());
    for _ in 0..board.player_count() {
        let (request_send, request_recv) = mpsc::channel();
        let (response_send, response_recv) = mpsc::channel();
        let handle = thread::spawn(move || run_send_recv_player(request_send, response_recv));
        up_channels.push(Channel{response_send,request_recv,handle});
    }
    board.shuffle_decks();
    board.run_mulligan();

    //let (response_sender, response_recv) = mpsc::channel();

    let step_loop = StepLoop::new(board.player_count(), 5);



    for mut step in step_loop {
        let start_time = Instant::now();
        let end_time = start_time + step.get_duration();
        println!("Start step {:?} at {:.*}", step, 2, fmt_time(start_time - game_start_time));

        match step {
            Step::GameStart() => {

            },
            Step::PlayerTurn(x) => {

            },
            Step::EndGame() => {

            },
        }

        let (sender, receiver) = mpsc::channel();
        //let local_receiver = response_recv.clone();


        loop {
            let now = Instant::now();
            if now > end_time {
                info!("Turn End: Overtime");
                match receiver.try_recv() {
                    Ok(s) => info!("Got in Overtime: {:?}", s),
                    Err(mpsc::TryRecvError::Empty) => {
                        info!("Overtime: Empty");
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        info!("Overtime: Disconnected");
                    }
                }
                break;
            }
            match receiver.recv_timeout(end_time - now) {
                Ok(s) => {
                    info!("Got: {:?}", s);
                    match s {
                        Request::EndTurn() => {}
                        Request::PlayCard(_) => {}
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
        //
        //thread::sleep(Duration::new(2, 0));
        //
        //match receiver.try_recv() {
        //    Ok(x) => println!("recived: {}",x), // we have a connection
        //    Err(mpsc::TryRecvError::Empty) => {
        //        drop(receiver);
        //        drop(t);
        //        // connecting took more than 5 seconds
        //        ()
        //    }
        //    Err(mpsc::TryRecvError::Disconnected) => unreachable!(),
        //}
        step.execute().expect("Step Error:");

        println!("End step: {:?}", step);

        //let event_queue: VecDeque<Event> = VecDeque::new();
        //for event in event_queue {
        //    prosses_events();
        //
        //    board.do_step_before_turn();
        //    board.do_step_draw();
        //
        //    //board.do_step_draw();
        //    prosses_events();
        //    //board.do_step_turn();
        //    prosses_events();
        //    //board.do_step_endof_turn();
        //    prosses_events();
        //
        //    //board.do_step_between_turns();
        //    prosses_events();
        //    //board.controller1.do_turn(&mut board.player1, turn_count);
        //    //board.controller2.do_turn(&mut board.player2, turn_count);
        //}
    }
}

// Message from out comeing in.
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Request {
    EndTurn(),
    PlayCard(String),
}

// Message going out.
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Response {
    Pong(),
    TurnIsOver(),
    CreateCard(),
    DrawCard(),
}

pub enum StepResult {
    Ok(),
    Empty(),
    GameEnd(usize, String),
}
fn prosses_events() -> () {}
use std::rc::Rc;

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

        board.player1.zones.deck.add_card(RefCell::new(c.clone()), Location::Top);
        board.player2.zones.deck.add_card(RefCell::new(c.clone()), Location::Top);
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    GameStart(),
    PlayerTurn(usize),
    EndGame(),
}

impl Step {
    fn execute(&mut self) -> Result<(), Error> {
        Ok(())
    }

    pub fn get_duration(&self) -> Duration {
        match self {
            &Step::GameStart() => Duration::new(1, 0),
            &Step::PlayerTurn(_) => Duration::new(2, 0),
            &Step::EndGame() => Duration::new(10, 0),
        }
    }
}

#[derive(Debug)]
enum Error {
    Generic(),
    GameEnd(),
}

struct StepLoop {
    next: Option<Step>,
    turn_count: u32,
    max_turns: u32,
    max_players: usize,
}

impl StepLoop {
    fn new(max_players: usize, max_turns: u32) -> StepLoop {
        StepLoop {
            next: Some(Step::GameStart()),
            turn_count: 0,
            max_players,
            max_turns,
        }
    }
}

impl Iterator for StepLoop {
    type Item = Step;

    fn next(&mut self) -> Option<Step> {
        let curr = self.next.clone();
        self.next = match curr {
            Some(ref x) => match x {
                &Step::GameStart() => Some(Step::PlayerTurn(0)),
                &Step::PlayerTurn(n) => {
                    let next_pidx = (n + 1) % self.max_players;
                    if next_pidx == 0 {
                        self.turn_count += 1;
                    }
                    if self.turn_count >= self.max_turns {
                        Some(Step::EndGame())
                    } else {
                        Some(Step::PlayerTurn(next_pidx))
                    }
                }
                &Step::EndGame() => None,
            },
            None => None,
        };
        curr
    }
}

#[derive(Debug)]
enum Event {
    PlayCard(),
    GameEnd(),
}

impl Event {
    fn prosses(&self) {
        println!("Event {:?}", self)
    }
}

struct EventQueue {
    events: Vec<Event>,
}
impl EventQueue {
    fn new() -> EventQueue {
        EventQueue { events: Vec::new() }
    }
}

impl Iterator for EventQueue {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        //let new_next = self.curr + self.next;

        /////self.curr = self.next;
        //self.next = new_next;

        // Since there's no endpoint to a Fibonacci sequence, the `Iterator`
        // will never return `None`, and `Some` is always returned.
        //Some(self.curr)
        None
    }
}
