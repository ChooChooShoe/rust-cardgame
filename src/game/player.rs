use std::fmt::Debug;
use std::io;
use serde::{Serialize,Deserialize};

use card::Card;

pub trait Player: Sized + Debug
{
    //fn new() -> Self;
    fn get_name(&self) -> &str;
    fn do_turn(&self) -> Option<u64>;
}

#[derive(Debug,Serialize,Deserialize)]
pub struct HumanPlayer
{
    name: String,
}

impl HumanPlayer
{
    pub fn new(name: String) -> HumanPlayer
    {
        HumanPlayer { name }
    }
}

impl Player for HumanPlayer
{
    fn get_name(&self) -> &str { self.name.as_str() }

    fn do_turn(&self) -> Option<u64>
    {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                println!("{} bytes read", n);
                println!("{}", input);
            }
            Err(error) => println!("error: {}", error),
        }
        None
    }
}