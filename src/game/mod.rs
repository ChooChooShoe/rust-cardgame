pub mod game_board;
//pub mod game_loop;
pub mod core;
pub mod zones;
pub mod action;
pub mod command;

pub use self::game_board::GameBoard;
pub use self::zones::ZoneCollection;
pub use self::zones::Zone;

pub use self::action::Error as ActionError;
pub use self::action::Result as ActionResult;
pub use self::action::Action as Action;

pub const MAX_PLAYER_COUNT: u8 = 2; 
pub const MAX_TURNS: u32 = 2; 

use game::action::Error;
use std::sync::{Arc,Mutex,MutexGuard};
use entity::card::CardPool;

#[derive(Clone)]
pub struct Game {
    pub board: Arc<Mutex<GameBoard>>,
    pub pool: Arc<Mutex<CardPool>>,
}

impl Game {
    pub fn new(board: GameBoard, pool: CardPool) -> Game {
        Game {
            board: Arc::new(Mutex::new(board)), 
            pool: Arc::new(Mutex::new(pool)),
        }
    }
    pub fn lock_board_and_then<U, F>(&self, op: F) -> Result<U, Error>
        where F: FnOnce(MutexGuard<GameBoard>) -> Result<U, Error> {
            self.board.lock().map_err(Error::from).and_then(op)
            // PoisonError is drop silently.
    }
    pub fn board_lock(&self) -> MutexGuard<GameBoard> {
        self.board.lock().unwrap()
    }
    pub fn pool_lock(&self) -> MutexGuard<CardPool> {
        self.pool.lock().unwrap()
    }
}