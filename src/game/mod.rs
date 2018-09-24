pub mod game_state;
//pub mod game_loop;
pub mod core;
pub mod zones;
pub mod action;
pub mod action_result;
pub mod deck;
pub mod player;
pub mod script;

pub use self::script::GameScript;
pub use self::player::Player;
pub use self::game_state::Game;
pub use self::zones::ZoneCollection;
pub use self::zones::Zone;
pub use self::deck::{Deck,Entry as DeckEntry};
pub use self::action::{Action};
pub use self::action_result::{Error as ActionError,OkCode,Result as ActionResult};

pub const MAX_PLAYER_COUNT: usize = 2; 
pub const MAX_TURNS: u32 = 2; 

pub type PlayerId = usize;
pub type ClientId = usize;
pub type CardId = usize;