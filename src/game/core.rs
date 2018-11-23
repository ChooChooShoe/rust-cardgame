use crate::game::PlayerId;
use crate::net::Connection;
use std::sync::mpsc::{Receiver, RecvError, RecvTimeoutError};
use std::time::{Duration, Instant};

use crate::game::action::Action;
use crate::game::{Game, Phase, Turn};
use crate::net::NetError;

// Message from clients to game loop.
pub enum Event {
    OpenConnection(usize, Connection),
    CloseConnection(usize),
    AllPlayersConnected(),
    StopAndExit(),
    OnPlayerAction(PlayerId, Action),
}

pub fn run(recv: Receiver<Event>, mut game: Game) {
    info!("Running core game loop as {:?}", game.network_mode());

    let mut state = State::Waiting;
    // last_player_id is set when an Event::OnPlayerAction causes a break.
    // loop until curent state is State::Done.
    while !state.is_done() {
        // Notify current state before loop.
        state.enter(&mut game);

        // Loops until the current state gets an event that will change state.
        // OR until the current states times out and next_on_timeout is used.
        // TODO not unwrap the NetError.
        let next_state = get_next_state(&recv, &mut state, &mut game).unwrap();

        // Recv loop ended. Notify current state. Then switch states.
        state.exit(&mut game);
        state = state.transition(&mut game, next_state);
    }

    // All connections are closed after the game.
    // When client only the connection to the server is closed.
    for conn in game.connections() {
        conn.disconnect();
    }
}

fn get_next_state(recv: &Receiver<Event>, state:  &mut State, game: &mut Game) -> Result<State,NetError> {
    let state_start = Instant::now();
    let deadline = state_start + state.get_duration();
    Ok(loop {
        // process before we wait to recive. Can break again before we recv anything.
        if game.process_actions()? {
            break state.next_on_request(game);
        }
        match recv.recv_deadline(deadline) {
            Ok(Event::OpenConnection(index, connection)) => {
                info!("Core got connection #{}", index);
                let conn = game.connection(index);
                conn.destroy();
                *conn = connection;
            }
            Ok(Event::CloseConnection(index)) => {
                game.connection(index).destroy();
            }
            Ok(Event::AllPlayersConnected()) => {
                assert!(game.network_mode().is_server());
                if *state == State::Waiting {
                    // game.send_all_action(&Action::GameStart());
                    break State::GameSetup;
                }
            }
            Ok(Event::StopAndExit()) => break State::Done(GameResults::StopAndExit),
            Ok(Event::OnPlayerAction(player_id, action)) => {
                game.queue_action(player_id, action);
                if game.process_actions()? {
                    break state.next_on_request(game);
                }
            }
            Err(RecvTimeoutError::Disconnected) => break State::Done(GameResults::StopAndExit),
            Err(RecvTimeoutError::Timeout) => {
                info!("State {:?} timeout!", state);
                break state.next_on_timeout(game);
            }
        }
    })
}


#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Waiting,
    GameSetup,
    GameStart,
    PlayerTurn(Turn),
    Done(GameResults),
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum GameResults {
    PlayerWin(PlayerId),
    OutOfTurns,
    StopAndExit,
    NotAllPlayersReady,
    NotAllPlayersConncted,
}

impl State {
    pub fn is_done(&self) -> bool {
        if let State::Done(_) = self {
            true
        } else {
            false
        }
    }
    pub fn get_duration(&self) -> Duration {
        match self {
            State::Waiting => Duration::from_secs(30),
            State::GameSetup => Duration::from_millis(50),
            State::GameStart => Duration::from_millis(50),
            State::PlayerTurn(turn) => turn.get_duration(),
            State::Done(_) => Duration::from_millis(10),
        }
    }

    // Consumes self and next_state to return the next current state.
    // Allows for non-standard transtions.
    fn transition(self, game: &mut Game, next_state: State) -> State {
        match (self, next_state) {
            (_from, _to) => (),
        }
        next_state
    }

    fn enter(&mut self, game: &mut Game) {
        debug!("Now entering state {:?}", self);
        match self {
            State::GameStart => {
                if game.network_mode().is_server() {
                    game.send_all_action(&Action::GameStart());
                    game.shuffle_decks();
                    game.run_mulligan();
                    // game.queue_action(0, Action::StartNextTurn());
                }
            }
            State::GameSetup => {
                if game.network_mode().is_server() {
                    game.send_all_action(&Action::BeginGameSetup());
                }
            }
            State::PlayerTurn(turn) => {
                if game.network_mode().is_server() {
                    let act = &Action::SwitchTurn(*turn);
                    for player in game.connections() {
                        player.send(act).unwrap();
                    }
                }
            }
            State::Waiting => (),
            State::Done(_) => unreachable!(),
        }
    }

    fn exit(&mut self, _game: &mut Game) {}

    fn next_player_turn(last_turn: &Turn) -> State {
        if let Some(turn) = last_turn.next() {
            State::PlayerTurn(turn)
        } else {
            State::Done(GameResults::OutOfTurns)
        }
    }
    fn next_on_request(&mut self, _game: &mut Game) -> State {
        match self {
            State::Waiting => State::Waiting,
            State::GameSetup => State::GameStart,
            State::GameStart => State::PlayerTurn(Turn::new(0, 1, Phase::Start)),
            State::PlayerTurn(turn) => State::next_player_turn(turn),
            State::Done(_) => panic!("State::Done can not have a next() state."),
        }
    }
    // Creates the next state when self is timedout.
    fn next_on_timeout(&mut self, _game: &mut Game) -> State {
        match self {
            State::Waiting => State::Done(GameResults::NotAllPlayersConncted),
            State::GameSetup => State::Done(GameResults::NotAllPlayersReady),
            State::GameStart => State::PlayerTurn(Turn::new(0, 1, Phase::Start)),
            State::PlayerTurn(turn) => State::next_player_turn(turn),
            State::Done(_) => panic!("State::Done can not have a next() state."),
        }
    }
}
