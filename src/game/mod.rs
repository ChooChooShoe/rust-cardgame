pub mod game_board;
pub mod game_loop;
pub mod zones;
pub mod action;

pub use self::game_board::GameBoard;
pub use self::zones::ZoneCollection;
pub use self::zones::Zone;

pub use self::action::Error as ActionError;
pub use self::action::Result as ActionResult;
pub use self::action::Action as Action;

pub const MAX_PLAYER_COUNT: usize = 2; 
pub const MAX_TURNS: u32 = 2; 
