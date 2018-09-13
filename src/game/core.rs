use game::MAX_TURNS;
use net::Connection;
use std::sync::mpsc::{Receiver, RecvError, RecvTimeoutError, TryRecvError};
use std::time::{Duration, Instant};
use utils::timer::Timer;

use game::action::{Action, ClientAction, ServerAction};
use game::Game;
use net::NetworkMode;

// Message from clients to game loop.
pub enum Event {
    OpenConnection(usize, Connection),
    CloseConnection(usize),
    AllPlayersConnected(),
    StopAndExit(),
    OnPlayerAction(usize, Action),
}

pub fn run(recv: Receiver<Event>, mode: NetworkMode, mut game: Game) {
    assert_eq!(mode, NetworkMode::Server);
    info!("\n\nRunning core game loop. [ press Ctrl-C to exit ]\n");

    info!("Waiting for connections");
    let mut state = State::PlayersConnecting;
    loop {
        // Loops waiting for 0 or more Event::AddPlayers(_,_) until 1 Event::AllPlayersConnected().
        match recv.recv() {
            Ok(Event::OpenConnection(id, connection)) => {
                info!("Core got connection for player #{}", id);
                *game.player_conn(id) = connection;
            }
            Ok(Event::CloseConnection(id)) => game.player_conn(id).on_close_connection(),
            Ok(Event::AllPlayersConnected()) => break,
            Ok(Event::StopAndExit()) => return,
            Ok(_) => warn!("All players have not connected yet! Event can not be handled."),
            Err(RecvError) => return,
        }
    }

    // When any one of the server handles sends StartGame() we begin.
    info!("Game Started");
    let _game_start_time = Instant::now();

    game.send_all_action(&Action::GameStart());

    game.shuffle_decks();

    game.run_mulligan();

    while state != State::Done {
        state = state.next();
        let state_start = Instant::now();
        let deadline = state_start + state.get_duration();
        loop {
            match recv.recv_deadline(deadline) {
                Ok(Event::OpenConnection(id, connection)) => {
                    info!("Core got connection for player #{}", id);
                    *game.player_conn(id) = connection;
                }
                Ok(Event::CloseConnection(id)) => game.player_conn(id).on_close_connection(),
                Ok(Event::AllPlayersConnected()) => (), // Ignore repeated AllPlayersConnected().
                Ok(Event::StopAndExit()) => state = State::EndGame(GameResults::StopAndExit),
                Err(RecvTimeoutError::Disconnected) => return,
                Err(RecvTimeoutError::Timeout) => {
                    warn!("State {:?} timed out.", state);
                    break;
                }
                Ok(Event::OnPlayerAction(player_id, action)) => {
                    game.queue_action(action);
                    //TODO all actions in queue are performed with this connection
                    //TODO watch for infinit loops.
                    while let Some(action) = game.pop_action() {
                        match ServerAction::perform(action, &mut game, player_id) {
                            Ok(code) => info!("action: {:?}", code),
                            Err(e) => info!("action: {:?}", e),
                        }
                    }
                }
            }
        }
    }

    for conn in game.connections() {
        conn.close();
    }
    game.conn_to_server().shutdown();
}

pub fn run_client(recv: Receiver<Event>, mode: NetworkMode, mut game: Game) {
    assert_eq!(mode, NetworkMode::Client);
    let mut connection = Connection::from_empty(0);
    let mut client_id = 0;
    let mut active_player = 0;
    let mut turn_count = 0;
    let mut current_state = State::PlayersConnecting;
    loop {
        match recv.recv() {
            Ok(Event::OpenConnection(id, new_connection)) => {
                client_id = id;
                connection = new_connection;
            }
            Ok(Event::CloseConnection(_id)) => {
                current_state = State::PlayersConnecting;
                connection.on_close_connection();
            }
            Ok(Event::AllPlayersConnected()) => current_state = State::GameStart,
            Ok(Event::StopAndExit()) => {
                current_state = State::EndGame(GameResults::StopAndExit);
                return;
            }
            Err(RecvError) => return,

            Ok(Event::OnPlayerAction(_player_id, action)) => {
                game.queue_action(action);
                //TODO all actions in queue are performed with this connection
                //TODO watch for infinit loops.
                while let Some(action) = game.pop_action() {
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
enum State {
    PlayersConnecting,
    GameStart,
    MuliginStart,
    MuliginEnd,
    PlayerTurn(usize, u32, Phase),
    EndGame(GameResults),
    Done,
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
impl Phase {
    pub fn next(self) -> Phase {
        match self {
            Phase::Start => Phase::Draw,
            Phase::Draw => Phase::Play,
            Phase::Play => Phase::Combat,
            Phase::Combat => Phase::End,
            Phase::End => Phase::Cleanup,
            Phase::Cleanup => Phase::Start,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameResults {
    PlayerWin(usize),
    OutOfTurns,
    StopAndExit,
}

impl State {
    pub fn get_duration(&self) -> Duration {
        match self {
            State::PlayersConnecting => Duration::from_secs(1),
            State::GameStart => Duration::from_millis(500),
            State::MuliginStart => Duration::from_secs(0),
            State::MuliginEnd => Duration::from_secs(0),
            State::PlayerTurn(_pidx, _turn, _phase) => Duration::from_secs(1),
            State::EndGame(_) => Duration::from_millis(100),
            State::Done => Duration::from_secs(0),
        }
    }

    // All State transitions are done here. States only have one possible next state.
    fn next(self) -> State {
        match self {
            State::PlayersConnecting => {
                info!("Game Starting!");
                State::MuliginStart
            }
            State::MuliginStart => {
                info!("Muligin begin!");
                //TODO muligin.
                State::MuliginEnd
            },
            State::MuliginEnd => {
                info!("Sending muligin results");
                //TODO Send muligin results.
                State::GameStart
            }
            State::GameStart => State::PlayerTurn(0, 1, Phase::Start),
            State::PlayerTurn(id, turn, Phase::Cleanup) => {
                let next_player_id = (id + 1) % 2; //change 2 to MAX_PLAYER_COUNT;
                let mut turn_count = turn;
                if next_player_id == 0 {
                    turn_count += 1;
                }
                if turn_count >= MAX_TURNS {
                    State::EndGame(GameResults::OutOfTurns)
                } else {
                    State::PlayerTurn(next_player_id, turn_count, Phase::Start)
                }
            }
            State::PlayerTurn(id, turn, phase) => State::PlayerTurn(id, turn, phase.next()),
            State::EndGame(_) => State::Done,
            State::Done => panic!("State::Done can not have a next() state."),
        }
    }
}

struct StepLoop {
    next: Option<State>,
}

impl StepLoop {
    fn new() -> StepLoop {
        StepLoop {
            next: Some(State::GameStart),
        }
    }
    fn from(state: State) -> StepLoop {
        StepLoop { next: Some(state) }
    }
}
