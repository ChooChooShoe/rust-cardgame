use std::time::Instant;
use game::{Zone,Player};
use std::io;
use ws::Sender as WsSender;
use game::Action;


pub trait ControllerCollection {
    fn send_all(&mut self, action: &Action);
    fn send(&mut self, index: usize, action: &Action);
}
impl ControllerCollection for Vec<Controller> {
    fn send_all(&mut self, action: &Action) {
        for controller in self {
            controller.send(action).unwrap_or(())
        }
    }
    fn send(&mut self, index: usize, action: &Action) {
        self[index].send(action).unwrap_or(())
    }
}

pub struct Controller {
    inner: WsNetController,
}
impl Controller {
    pub fn index(&self) -> usize {
        self.inner.player_index
    }
    pub fn send(&mut self, action: &Action) -> Result<(), ()> {
        self.inner.send(action)
    }
}

pub struct WsNetController {
    player_index: usize,
    ws_sender: WsSender,
}

impl WsNetController {
    pub fn new(pidx: usize, send: WsSender) -> WsNetController {
        WsNetController {
            player_index: pidx,
            ws_sender: send,
        }
    }
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
        info!("Player #{} turn {} start.", self.player_index, turn_count);

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_num_bytes) => {
                self.handle_user_input(player, input.trim().split(" ").collect());
            }
            Err(error) => println!("Read input line error: {}", error),
        }

        None
    }
    pub fn send(&mut self, action: &Action) -> Result<(), ()> {
        // TODO make this not as bad.
        Ok(self.ws_sender.send(action.encode().unwrap()).unwrap())
    }
}
impl Into<Controller> for WsNetController {
    fn into(self) -> Controller {
        Controller { inner: self }
    }
}