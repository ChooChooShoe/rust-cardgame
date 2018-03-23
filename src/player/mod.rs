pub mod controller;
pub mod command_line;
pub mod player;

pub use self::controller::Controller;
pub use self::player::Player;

pub type PlayerId = u8;