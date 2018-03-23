pub mod game_board;
//pub mod game_loop;
pub mod core;
pub mod zones;
pub mod action;
pub mod action_result;
pub mod command;
pub mod deck;

pub use self::game_board::GameBoard;
pub use self::zones::ZoneCollection;
pub use self::zones::Zone;
pub use self::deck::{Deck,Entry as DeckEntry};
pub use self::action::{Action,ClientAction};
pub use self::action_result::{Error as ActionError,OkCode,Result as ActionResult};

pub const MAX_PLAYER_COUNT: u8 = 2; 
pub const MAX_TURNS: u32 = 2; 

use std::sync::{Arc,Mutex,MutexGuard};
use bincode::Error as StdError;
use entity::CardPool;

#[derive(Clone)]
pub struct Game {
    pub board: Arc<Mutex<GameBoard>>,
    pub pool: Arc<Mutex<CardPool>>,
    pub next_netid: Arc<Mutex<u64>>,
}

impl Game {
    pub fn new(board: GameBoard, pool: CardPool) -> Game {
        Game {
            board: Arc::new(Mutex::new(board)), 
            pool: Arc::new(Mutex::new(pool)),
            next_netid: Arc::new(Mutex::new(1)),
        }
    }
    // pub fn lock_board_and_then<U, F>(&self, op: F) -> Result<U, StdError>
    //     where F: FnOnce(MutexGuard<GameBoard>) -> Result<U, StdError> {
    //         self.board.lock().map_err(|e| ActionError::from(e)).and_then(op)
    //         // PoisonError is drop silently.
    // }
    pub fn board_lock(&self) -> MutexGuard<GameBoard> {
        self.board.lock().unwrap()
    }
    pub fn pool_lock(&self) -> MutexGuard<CardPool> {
        self.pool.lock().unwrap()
    }
    pub fn next_netid(&self) -> u64 {
        let x = self.next_netid.lock();
        let res = x.unwrap();
        *res
    }
}