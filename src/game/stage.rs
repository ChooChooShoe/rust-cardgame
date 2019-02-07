use crate::game::action::{Action, Actor, OkCode};
use crate::game::{Game, GameSettings, NetPlayerId, Phase, Turn};
use crate::net::{Connection, NetError};
use crate::utils::timer::Timer;
use std::collections::VecDeque;
use std::sync::mpsc::IntoIter;
use std::sync::mpsc::TryIter;
use std::sync::mpsc::{channel, Receiver, RecvError, RecvTimeoutError, Sender, TryRecvError};
use std::time::{Duration, Instant};

// Message from clients to game loop.
pub enum NetRelay {
    Open(NetPlayerId, Connection),
    Close(NetPlayerId),
    Act(NetPlayerId, Action),
    Start(),
    Shutdown(NetPlayerId),
}

/// The stage is were all events are gatherd and procesed.
pub struct Stage {
    recv: Receiver<NetRelay>,
    settings: GameSettings,
    state: State,
    action_queue: VecDeque<(Actor, Action)>,
    state_end_time: Instant,
}

impl Stage {
    pub fn build(settings: GameSettings) -> (Sender<NetRelay>, Stage) {
        let (send, recv) = channel();
        let stage = Stage {
            recv: recv,
            settings: settings,
            state: State::Waiting,
            action_queue: VecDeque::new(),
            state_end_time: Instant::now(),
        };
        (send, stage)
    }
    /// For srver or hosting client.
    pub fn run_authority(self) {
        self.run()
    }

    /// For clients
    pub fn run(mut self) {
        let mut game = Game::new(&self.settings);

        info!("Running core game loop as {:?}", game.network_mode());

        self.state.enter(&mut game);
        self.inner_loop(&mut game).unwrap_or_else(|e| warn!("Stage loop failed: {}", e));
        self.state.exit(&mut game);

        // All connections are closed after the game.
        // When client only the connection to the server is closed.
        for conn in game.connections() {
            conn.disconnect();
        }
    }
    /// Reads all from recv. Returns `Ok(())` if everything was read.
    /// Returns `Err(RecvError)` if the channel was disconnected
    fn inner_loop(&mut self, game: &mut Game) -> Result<(), NetError> {
        let mut count = 0;
        self.state_end_time = Instant::now() + self.state.get_duration();
        loop {
            debug!("Loop Count: {}", count);
            if self.state.is_done() {
                return Ok(());
            }
            // self.process_actions(game)?;

            self.get_recv(game)?;

            count += 1;
        }
    }


    /// Reads all from recv. Returns `Ok(())` if everything was read.
    /// Returns `Err(RecvError)` if the channel was disconnected
    fn get_recv(&mut self, mut game: &mut Game) -> Result<(), NetError> {
        loop {
            if self.state.is_done() {
                return Ok(());
            }
            self.process_actions(game)?;

            let relay = self.recv.recv_timeout(game.timer.time_left());

            match relay {
                Ok(NetRelay::Open(index, connection)) => {
                    let conn = game.connection(index);
                    conn.destroy();
                    *conn = connection;
                }
                Ok(NetRelay::Close(index)) => {
                    game.connection(index).destroy();
                }
                Ok(NetRelay::Start()) => {
                    debug_assert!(game.network_mode().is_server());
                    debug_assert_eq!(State::Waiting, self.state);
                    self.state.transition_to(&mut game, State::GameSetup);
                }
                Ok(NetRelay::Shutdown(_player_id)) => {
                    self.state
                        .transition_to(game, State::Done(GameResults::StopAndExit));
                }
                Ok(NetRelay::Act(player_id, mut action)) => {
                    let mut actor = Actor::User(player_id);
                    if self.validate_action(&mut actor, &mut action).is_ok() {
                        self.action_queue.push_back((actor, action));
                    }
                }
                Err(RecvTimeoutError::Timeout) => {
                    info!("Timeout");
                    let next = self.state.timeout(game);
                    self.state.transition_to(game, next);
                }
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(NetError::Disconnected);
                }
                // Err(TryRecvError::Empty) => {
                //     info!("Sleep");
                //     std::thread::sleep(Duration::from_nanos(100_000_000));
                // }
                // Err(TryRecvError::Disconnected) => {
                //     return Err(NetError::Disconnected);
                // }
                // Err(RecvError) => {
                //     return Err(NetError::Disconnected);
                // }
            }
            
        }
    }
    fn validate_action(&mut self, actor: &mut Actor, action: &mut Action) -> Result<(), ()> {
        Ok(())
    }
    /// Returns true when a state changse is needed.
    /// TODO all actions in queue are performed with this connection
    /// TODO watch for infinit loops.
    fn process_actions(&mut self, game: &mut Game) -> Result<(), NetError> {
        while let Some(action) = self.action_queue.pop_front() {
            match action.1.perform(game, &action.0) {
                Ok(OkCode::ChangeState) => {
                    let next = self.state.next(game);
                    self.state.transition_to(game, next);
                },
                Ok(OkCode::Done) => (),
                Ok(code) => {
                    let a = Action::OnResponceOk(code);
                    game.connection(action.0.id()).send(&a)?;
                }
                Err(e) => {
                    info!("action err: {:?}", e);
                    let a = Action::OnResponceErr(e);
                    game.connection(action.0.id()).send(&a)?;
                }
            }
        }
        Ok(())
    }

    fn get_timeout(&self) {

    }
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
    PlayerWin(usize),
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
            State::Waiting => Duration::from_secs(15),
            State::GameSetup => Duration::from_millis(50),
            State::GameStart => Duration::from_millis(50),
            State::PlayerTurn(turn) => turn.get_duration(),
            State::Done(_) => Duration::from_millis(10),
        }
    }

    pub fn transition_to(&mut self, game: &mut Game, mut next: State) {
        self.exit(game);
        *self = next;
        next.enter(game);
    }

    fn next(&mut self, _game: &mut Game) -> State {
        match self {
            State::Waiting => State::Waiting,
            State::GameSetup => State::GameStart,
            State::GameStart => State::PlayerTurn(Turn::new(0, 1, Phase::Start)),
            State::PlayerTurn(turn) => State::next_player_turn(turn),
            State::Done(_) => panic!("State::Done can not have a next() state."),
        }
    }
    /// Creates the next state when self is timedout.
    fn timeout(&mut self, _game: &mut Game) -> State {
        match self {
            State::Waiting => State::Done(GameResults::NotAllPlayersConncted),
            State::GameSetup => State::Done(GameResults::NotAllPlayersReady),
            State::GameStart => State::PlayerTurn(Turn::new(0, 1, Phase::Start)),
            State::PlayerTurn(turn) => State::next_player_turn(turn),
            State::Done(_) => panic!("State::Done can not have a next() state."),
        }
    }

    fn enter(&mut self, game: &mut Game) {
        info!("Now entering state {:?}", self);
        game.timer = Timer::from_duration(self.get_duration());
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
            State::Done(_) => (),
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
}
