use game::Action;
use game::{Player, Zone};
use net::Codec;
use std::io;
use std::time::Instant;
use ws::{Sender as WsSender,CloseCode};

pub struct Connection {
    player_id: usize,
    inner: Inner,
}
impl Connection {
    pub fn from_network(player_id: usize, sender: WsSender) -> Connection {
        Connection {
            player_id,
            inner: Inner::WebSocetPlayer(sender),
        }
    }
    pub fn from_empty(player_id: usize) -> Connection {
        Connection {
            player_id,
            inner: Inner::EmptyPlayer(),
        }
    }
    pub fn player_id(&self) -> usize {
        self.player_id
    }
    pub fn set_player_id(&mut self, player_id: usize) {
        self.player_id = player_id
    }

    pub fn send(&mut self, action: &Action) -> Result<(), ()> {
        self.inner.send(action);
        Ok(())
    }

    pub fn close(&self) {
        match &self.inner {
            Inner::WebSocetPlayer(ws) => ws.close(CloseCode::Normal).unwrap_or(()),
            Inner::EmptyPlayer() => (),
        }
    }
    pub fn shutdown(&self) {
        match &self.inner {
            Inner::WebSocetPlayer(ws) => ws.shutdown().unwrap_or(()),
            Inner::EmptyPlayer() => (),
        }
    }
    pub fn on_close_connection(&mut self) {
        self.inner = Inner::EmptyPlayer();
    }
}

enum Inner {
    WebSocetPlayer(WsSender),
    EmptyPlayer(),
}

impl Inner {
    pub fn handle_user_input(&mut self, player: &mut Player, args: Vec<&str>) {
        match args.len() {
            0 => println!("No command entered"),
            1 => match args[0] {
                "draw" => {
                    player.draw_x_cards(1);
                }
                _ => println!("Unknown command: {:?}", args),
            },
            _ => println!("Unknown command: {:?}", args),
        }
    }
    pub fn do_turn(&mut self, player: &mut Player, turn_count: usize) -> Option<u64> {
        //info!("Player #{} turn {} start.", self.player_index, turn_count);

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_num_bytes) => {
                self.handle_user_input(player, input.trim().split(" ").collect());
            }
            Err(error) => println!("Read input line error: {}", error),
        }

        None
    }
    pub fn send(&mut self, action: &Action) {
        // TODO make this not as bad.
        match self {
            Inner::WebSocetPlayer(sender) => sender.send(action.encode().unwrap()).unwrap(),
            Inner::EmptyPlayer() => (),
        }
    }
}
