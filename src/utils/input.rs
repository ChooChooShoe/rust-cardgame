use crate::game::{Action, Game};
use std::io::{self, BufRead, Write};
use std::{thread, time::Duration};

pub struct Input;

impl Input {
    pub fn flush() {
        io::stdout().flush().unwrap_or(());
    }
    pub fn handle_input(sender: usize, game: &mut Game) {
        thread::sleep(Duration::from_millis(4));
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        print!("> ");
        Input::flush();
        let mut buffer = String::new();
        match handle.read_line(&mut buffer) {
            Ok(_num_bytes) => {
                if Input::handle_user_input(sender, game, buffer.trim().split(" ").collect()) {
                    game.queue_action(sender, Action::HandleInput());
                }
            }
            Err(error) => println!("Read input line error: {}", error),
        }
    }

    pub fn handle_user_input(sender: usize, game: &mut Game, args: Vec<&str>) -> bool {
        match args.len() {
            0 => println!("No command entered"),
            1 => match args[0] {
                "draw" => {
                    println!("drawing: {:?}", "card");
                    game.send_action(0, &Action::DrawCardAnon(2, 3));
                }
                "pass" => {
                    println!("passing the turn");
                    game.send_action(0, &Action::EndTurn(0));
                    return false;
                }
                _ => println!("Unknown command: {:?}", args),
            },
            _ => println!("Unknown command: {:?}", args),
        }
        true
    }
}
