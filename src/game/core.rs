use crate::config;
use crate::game::PlayerId;
use crate::game::MAX_TURNS;
use crate::net::Connection;
use std::sync::mpsc::{Receiver, RecvError, RecvTimeoutError};
use std::time::{Duration, Instant};

use crate::game::action::Action;
use crate::game::{ActionError, Game, OkCode};
use crate::net::NetworkMode;

// Message from clients to game loop.
pub enum Event {
    OpenConnection(PlayerId, Connection),
    CloseConnection(PlayerId),
    AllPlayersConnected(),
    StopAndExit(),
    OnPlayerAction(PlayerId, Action),
}

pub fn run(recv: Receiver<Event>, mut game: Game) {
    info!("Running core game loop as {:?}", game.network_mode());

    let mut state = State::Waiting;
    // last_player_id is set when an Event::OnPlayerAction causes a break.
    let mut last_player_id: PlayerId = 0;
    // loop until curent state is State::Done.
    while !state.is_done() {
        let state_start = Instant::now();
        let deadline = state_start + state.get_duration();
        // Notify current state before loop.
        state.enter(&mut game);

        // Loops until the current state gets an event that will change state.
        // OR until the current states times out and next_on_timeout is used.
        let next_state = loop {
            // process before we wait to recive. Can break again before we recv anything.
            if process_actions(&mut game, last_player_id) {
                break state.next_on_request(&mut game);
            }
            match recv.recv_deadline(deadline) {
                Ok(Event::OpenConnection(id, connection)) => {
                    info!("Core got connection #{}", id);
                    *game.connection(id) = connection;
                }
                Ok(Event::CloseConnection(id)) => {
                    game.connection(id).on_close_connection();
                }
                Ok(Event::AllPlayersConnected()) => {
                    assert!(game.network_mode().is_server());
                    if state == State::Waiting {
                        // game.send_all_action(&Action::GameStart());
                        break State::GameSetup;
                    }
                }
                Ok(Event::StopAndExit()) => break State::Done(GameResults::StopAndExit),
                Ok(Event::OnPlayerAction(player_id, action)) => {
                    game.queue_action(action);
                    //TODO all actions in queue are performed with this connection
                    //TODO watch for infinit loops.
                    if process_actions(&mut game, player_id) {
                        last_player_id = player_id;
                        break state.next_on_request(&mut game);
                    }
                }
                Err(RecvTimeoutError::Disconnected) => break State::Done(GameResults::StopAndExit),
                Err(RecvTimeoutError::Timeout) => {
                    info!("State {:?} timeout!", state);
                    break state.next_on_timeout(&mut game);
                }
            }
        };
        // Recv loop ended. Notify current state. Then switch states.
        state.exit(&mut game);
        state = state.transition(&mut game, next_state);
    }

    // All connections are closed after the game.
    // When client only the connection to the server is closed.
    for conn in game.connections() {
        conn.close();
    }
}
fn process_actions(game: &mut Game, player_id: PlayerId) -> bool {
    while let Some(action) = game.pop_action() {
        match action.perform(game, player_id) {
            Ok(OkCode::ChangeState) => return true,
            Ok(_) => (),
            Err(e) => info!("action err: {:?}", e),
        }
    }
    false
}
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Waiting,
    GameSetup,
    GameStart,
    PlayerTurn(PlayerId, u32, Phase),
    Done(GameResults),
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Start,
    Draw,
    Play,
    End,
}
impl Phase {
    pub fn next(self) -> Phase {
        match self {
            Phase::Start => Phase::Draw,
            Phase::Draw => Phase::Play,
            Phase::Play => Phase::End,
            Phase::End => Phase::Start,
        }
    }
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
            State::PlayerTurn(_pidx, _turn, _phase) => Duration::from_millis(50),
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
        info!("Now entering state {:?}", self);
        match self {
            State::GameStart => {
                if game.network_mode().is_server() {
                    game.send_all_action(&Action::GameStart());
                    game.shuffle_decks();
                    game.run_mulligan();
                    game.queue_action(Action::StartNextTurn());
                }
            }
            State::GameSetup => {
                if game.network_mode().is_server() {
                    game.send_all_action(&Action::BeginGameSetup());
                }
            }
            State::PlayerTurn(player_id, turn, phase) => {
                if game.network_mode().is_server() {
                    let act = &Action::StartPlayerTurn(*player_id, *turn, *phase);
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

    fn next_player_turn(id: PlayerId, turn: u32, phase: Phase) -> State {
        if phase == Phase::End {
            // The final phase of the turn.
            let next_player_id = (id + 1) % config::active().player_count;
            let turn_count = if next_player_id == 0 { turn + 1 } else { turn };
            if turn_count >= config::active().turn_limit {
                State::Done(GameResults::OutOfTurns)
            } else {
                State::PlayerTurn(next_player_id, turn_count, Phase::Start)
            }
        } else {
            State::PlayerTurn(id, turn, phase.next())
        }
    }
    fn next_on_request(&mut self, _game: &mut Game) -> State {
        match self {
            State::Waiting => State::Waiting,
            State::GameSetup => State::GameStart,
            State::GameStart => State::PlayerTurn(0, 1, Phase::Start),
            State::PlayerTurn(id, turn, phase) => State::next_player_turn(*id, *turn, *phase),
            State::Done(_) => panic!("State::Done can not have a next() state."),
        }
    }
    // Creates the next state when self is timedout.
    fn next_on_timeout(&mut self, _game: &mut Game) -> State {
        match self {
            State::Waiting => State::Done(GameResults::NotAllPlayersConncted),
            State::GameSetup => State::Done(GameResults::NotAllPlayersReady),
            State::GameStart => State::PlayerTurn(0, 1, Phase::Start),
            State::PlayerTurn(id, turn, phase) => State::next_player_turn(*id, *turn, *phase),
            State::Done(_) => panic!("State::Done can not have a next() state."),
        }
    }
}
