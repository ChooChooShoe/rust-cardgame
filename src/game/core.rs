use game::MAX_TURNS;
use net::Connection;
use std::sync::mpsc::{Receiver, RecvError, RecvTimeoutError};
use std::time::{Duration, Instant};

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

    let mut state = State::PlayersConnecting;

    // loop until curent state is State::Done.
    while state != State::Done {
        let state_start = Instant::now();
        let deadline = state_start + state.get_duration();

        // Notify current state before loop.
        state.enter(&mut game);

        // Loops until the current state gets an event that will change state.
        // OR until the current states times out and next_on_timeout is used.
        let next_state = loop {
            match recv.recv_deadline(deadline) {
                Ok(event) => {
                    if let Some(next) = state.on_event(&mut game, event) {
                        break next;
                    }
                }
                Err(RecvTimeoutError::Disconnected) => break State::Done,
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
    for conn in game.connections() {
        conn.close();
    }
}

pub fn run_client(recv: Receiver<Event>, mode: NetworkMode, mut game: Game) {
    assert_eq!(mode, NetworkMode::Client);
    let mut state = State::PlayersConnecting;

    // loop until curent state is State::Done.
    while state != State::Done {
        // No Timmers for client.
        //let state_start = Instant::now();
        //let deadline = state_start + state.get_duration();

        // Notify current state before loop.
        state.enter(&mut game);

        // Loops until the current state gets an event that will change state.
        // OR until the current states times out and next_on_timeout is used.
        let next_state = loop {
            match recv.recv() {
                Ok(Event::OpenConnection(_, connection)) => {
                    *game.server() = connection;
                }
                Ok(Event::CloseConnection(_id)) => {
                    game.server().on_close_connection();
                    // TODO A way to do reconecting.
                    // break State::PlayersConnecting
                }
                Ok(Event::AllPlayersConnected()) => (), // Not for clients.
                Ok(Event::StopAndExit()) => {
                    break State::Done
                }
                Ok(Event::OnPlayerAction(_player_id, action)) => {
                    game.queue_action(action);
                    //TODO all actions in queue are performed with this connection
                    //TODO watch for infinit loops.
                    while let Some(action) = game.pop_action() {
                        match ClientAction::perform(action, &mut game) {
                            Ok(code) => info!("action: {:?}", code),
                            Err(e) => info!("action: {:?}", e),
                        }
                    }
                }
                Err(RecvError) => break State::Done,
            }
        };
        // Recv loop ended. Notify current state. Then switch states.
        state.exit(&mut game);
        state = state.transition(&mut game, next_state);
    }

    game.server().close();
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
    pub fn next(self, _game: &mut Game) -> Phase {
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
            State::PlayersConnecting => Duration::from_secs(30),
            State::GameStart => Duration::from_millis(50),
            State::MuliginStart => Duration::from_secs(0),
            State::MuliginEnd => Duration::from_secs(0),
            State::PlayerTurn(_pidx, _turn, _phase) => Duration::from_millis(50),
            State::EndGame(_) => Duration::from_millis(10),
            State::Done => Duration::from_secs(0),
        }
    }

    // Prossesing for a given event.
    // Returns the next state if a state transistion is needed.
    // Returning None will keep looping on_event() untill a State is returned or get_duration() is exceded.
    fn on_event(&mut self, game: &mut Game, event: Event) -> Option<State> {
        match event {
            Event::OpenConnection(id, connection) => {
                info!("Core got connection for player #{}", id);
                *game.connection(id) = connection;
                None
            }
            Event::CloseConnection(id) => {
                game.connection(id).on_close_connection();
                None
            }
            Event::AllPlayersConnected() => {
                if let State::PlayersConnecting = self {
                    Some(State::GameStart)
                } else {
                    None
                }
            }
            Event::StopAndExit() => Some(State::Done),
            Event::OnPlayerAction(player_id, action) => {
                game.queue_action(action);
                //TODO all actions in queue are performed with this connection
                //TODO watch for infinit loops.
                while let Some(action) = game.pop_action() {
                    match ServerAction::perform(action, game, player_id) {
                        Ok(code) => info!("action: {:?}", code),
                        Err(e) => info!("action: {:?}", e),
                    }
                }
                None
            }
        }
    }
    // Consumes self and next_state to return the next current state.
    // Allows for non-standard transtions.
    fn transition(self, game: &mut Game, next_state: State) -> State {
        match (self, next_state) {
            (State::Done, _) => return State::Done,
            (_, State::GameStart) => {
                game.send_all_action(&Action::GameStart());
                game.shuffle_decks();
                game.run_mulligan();
            }
            (_from, _to) => (),
        }
        next_state
    }

    fn enter(&mut self, _game: &mut Game) {
        info!("Now entering state {:?}", self)
    }

    fn exit(&mut self, _game: &mut Game) {}

    // Creates the next state when self is timedout.
    fn next_on_timeout(&mut self, game: &mut Game) -> State {
        match self {
            State::PlayersConnecting => State::Done, //Not all players connected.
            State::MuliginStart => {
                info!("Muligin begin!");
                //TODO muligin.
                State::MuliginEnd
            }
            State::MuliginEnd => {
                info!("Sending muligin results");
                //TODO Send muligin results.
                State::GameStart
            }
            State::GameStart => State::PlayerTurn(0, 1, Phase::Start),
            State::PlayerTurn(id, turn, Phase::Cleanup) => {
                // The final phase of the turn.
                let next_player_id = (*id + 1) % game.players().len();
                let turn_count = if next_player_id == 0 {
                    *turn + 1
                } else {
                    *turn
                };
                if turn_count >= MAX_TURNS {
                    State::EndGame(GameResults::OutOfTurns)
                } else {
                    State::PlayerTurn(next_player_id, turn_count, Phase::Start)
                }
            }
            State::PlayerTurn(id, turn, phase) => State::PlayerTurn(*id, *turn, phase.next(game)),
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
