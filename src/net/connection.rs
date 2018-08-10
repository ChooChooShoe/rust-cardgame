use game::Action;
use game::{Player, Zone};
use net::Command;
use std::io;
use std::time::Instant;
use ws::Sender as WsSender;

pub trait ConnectionVec {
    fn send_all(&mut self, action: Action);
    fn send(&mut self, player_id: usize, action: Action);
    fn add_player(&mut self, player_id: usize, con: Connection);
    fn remove_player(&mut self, player_id: usize);
}
impl ConnectionVec for Vec<Connection> {

    fn add_player(&mut self, player_id: usize, con: Connection) {
        if player_id < self.len() {
            // player_id is before in the vec, overwride other player.
            self[player_id] = con
        } else {
            // player_id is out of bounds, add some empty players as padding.
            let additional = player_id - self.len();
            self.reserve(additional + 1);
            let l = self.len();
            for i in 0..additional {
                self.push(Connection::from_empty(l + i));
            }
            self.push(con)
        }
    }
    fn remove_player(&mut self, player_id: usize) {
        if player_id < self.len() {
            self[player_id] = Connection::from_empty(player_id)
        }
    }


    fn send_all(&mut self, action: Action) {
        let cmd = Command::TakeAction(action);
        for connection in self {
            connection.send_command(&cmd)
        }
    }
    fn send(&mut self, player_id: usize, action: Action) {
        self[player_id].send(action).unwrap_or(());
    }
}

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

    pub fn send(&mut self, action: Action) -> Result<(), ()> {
        self.inner.send(action)
    }

    pub fn send_command(&mut self, cmd: &Command) {
        self.inner.send_command(&cmd)
    }
    pub fn on_connection_lost(&mut self) {
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
    pub fn send(&mut self, action: Action) -> Result<(), ()> {
        // TODO make this not as bad.
        let cmd = Command::TakeAction(action);
        self.send_command(&cmd);
        Ok(())
    }

    pub fn send_command(&mut self, cmd: &Command) {
        match self {
            Inner::WebSocetPlayer(sender) => {
                sender.send(cmd.encode().unwrap()).unwrap()
            }
            Inner::EmptyPlayer() => (),
        }
    }
}
