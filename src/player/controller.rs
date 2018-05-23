use game::Zone;
use player::Player;
use std::io;
use ws::Sender as WsSender;

pub struct Controller
{
    player_index: usize,
    ws_sender: WsSender,

}

//impl Networked for Controller
//{
//    fn netid(&self) -> u64 {
//        match self {
//            &Controller::CmdLinePlayer(netid) => netid as u64,
//            &Controller::AiPlayer(netid) => netid as u64,
//        }
//    }
//}

impl Controller
{
    pub fn new(pidx: usize, send: WsSender) -> Controller
    {
        Controller {
            player_index: pidx,
            ws_sender: send,
        }
    }
    pub fn index(&self) -> usize {
        return self.player_index
    }
    pub fn on_game_start() -> Result<(),()>
    {
        Ok(())
    }


    pub fn handle_user_input(&mut self, player: &mut Player, args: Vec<&str>)
    {
        match args.len() {
            0 => println!("No command entered"),
            1 => {
                match args[0] {
                    "draw" => {
                        player.draw_x_cards(1).unwrap();
                    },
                    _ => println!("Unknown command: {:?}", args),
                }
            },
            _ => println!("Unknown command: {:?}", args),
        }
    }
    pub fn do_turn(&mut self, player: &mut Player, turn_count: usize) -> Option<u64>
    {
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

}
