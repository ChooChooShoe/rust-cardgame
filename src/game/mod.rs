pub mod game_board;
//pub mod game_loop;
pub mod core;
pub mod zones;
pub mod action;
pub mod action_result;
pub mod command;
pub mod deck;
pub mod player;
pub mod script;

pub use self::script::GameScript;
pub use self::player::Player;
pub use self::game_board::GameBoard;
pub use self::zones::ZoneCollection;
pub use self::zones::Zone;
pub use self::deck::{Deck,Entry as DeckEntry};
pub use self::action::{Action};
pub use self::action_result::{Error as ActionError,OkCode,Result as ActionResult};

pub const MAX_PLAYER_COUNT: usize = 2; 
pub const MAX_TURNS: u32 = 2; 

use std::sync::{Arc,Mutex,MutexGuard,RwLock,RwLockReadGuard};
use bincode::Error as StdError;
use entity::CardPool;

#[derive(Clone)]
pub struct Game {
    pub board: Arc<Mutex<GameBoard>>,
    pub pool: Arc<RwLock<CardPool>>,
    pub next_netid: Arc<Mutex<u64>>,
}

impl Game {
    pub fn new(board: GameBoard, pool: CardPool) -> Game {
        Game {
            board: Arc::new(Mutex::new(board)), 
            pool: Arc::new(RwLock::new(pool)),
            next_netid: Arc::new(Mutex::new(1)),
        }
    }
    // Create a new version the Game with same CardPool and a spearate GameBoard.
    pub fn new_with_board(&self, board: GameBoard) -> Game {
        Game {
            board: Arc::new(Mutex::new(board)), 
            pool: self.pool.clone(),
            next_netid: Arc::new(Mutex::new(1)),
        }
    }
    // pub fn lock_board_and_then<U, F>(&self, op: F) -> Result<U, StdError>
    //     where F: FnOnce(MutexGuard<GameBoard>) -> Result<U, StdError> {
    //         self.board.lock().map_err(|e| ActionError::from(e)).and_then(op)
    //         // PoisonError is drop silently.
    // }
    pub fn board_lock(&self) -> MutexGuard<GameBoard> {
        self.board.lock().expect("GameBoard lock failed")
    }
    pub fn pool_lock(&self) -> RwLockReadGuard<CardPool> {
        self.pool.read().expect("CardPool lock failed")
    }
    pub fn next_netid(&self) -> u64 {
        let mut guard = self.next_netid.lock().expect("CardPool lock failed");
        let curr_num = *guard;
        *guard = curr_num + 1;
        drop(guard);
        curr_num
    }
    pub fn max_players(&self) -> usize {
        2
    }
}