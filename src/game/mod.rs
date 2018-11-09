pub mod game_state;
//pub mod game_loop;
pub mod action;
pub mod action_result;
pub mod active_card_pool;
pub mod core;
pub mod deck;
pub mod player;
pub mod script;
pub mod turn;
pub mod zones;

pub use self::action::Action;
pub use self::action_result::{Error as ActionError, OkCode, Result as ActionResult};
pub use self::active_card_pool::ActiveCardPool;
pub use self::deck::{Deck, Entry as DeckEntry};
pub use self::game_state::Game;
pub use self::player::Player;
pub use self::script::GameScript;
pub use self::turn::{Phase, Turn};
pub use self::zones::Zone;
pub use self::zones::ZoneCollection;

pub const MAX_PLAYER_COUNT: usize = 2;
pub const MAX_TURNS: u32 = 2;

pub type PlayerId = usize;
pub type ClientId = usize;
pub type CardId = usize;
