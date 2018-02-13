use std::fmt::Debug;
use std::io;
use serde::{Serialize,Deserialize};

use card::Card;
use game::ZoneCollection;
use game::zones::{Zone,Location};

pub struct Player
{
    pub name: String, 
    pub zones: ZoneCollection,
    pub pc: Controller
}

pub enum Controller
{
    CmdLinePlayer(),
    AiPlayer(u32),
}


impl Player
{
    pub fn new(name: String, pc: Controller) -> Player {
        Player {name, zones: ZoneCollection::new(42), pc}
    }

    pub fn name(&self) -> &str { self.name.as_str() }

    pub fn do_turn(&mut self) -> Option<u64>
    {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                let cmds: Vec<_> = input.split(" ").collect();
                self.handle_user_input(cmds, n);
            }
            Err(error) => println!("error: {}", error),
        }
        None
    }

    fn handle_user_input(&mut self, input: Vec<&str>, bytes_read: usize)
    {
        println!("{} bytes read", bytes_read);
        println!("{:?}", &input);

            let x = 1;
            self.draw_x_cards(x);
    }



    pub fn zones(&self) -> &ZoneCollection {
        &self.zones
    }
    pub fn zones_mut(&mut self) -> &mut ZoneCollection {
        &mut self.zones
    }


    
    pub fn draw_x_cards(&mut self, x: usize) -> Result<(),()>
    {
        let drawn_cards = self.zones.deck.take_x_cards(x, Location::Top);

        let mut cards_to_add = Vec::with_capacity(x);
        //TODO on card drawn event
        for c in drawn_cards{
            match c{
                Some(card) => {
                    info!("on_card_drawn: deck -> hand : {:?}", card.borrow().name());
                    cards_to_add.push(card);
                },
                None => 
                    {info!("on_card_drawn: deck -> hand : NONE");},
            }
        }
        self.zones.hand.add_cards(cards_to_add,Location::Top);
        Ok(())
    }
}