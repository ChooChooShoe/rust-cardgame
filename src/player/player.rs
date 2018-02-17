use std::fmt::Debug;
use std::io;
use serde::{Serialize,Deserialize};
use net;

use card::Card;
use game::ZoneCollection;
use player::controller::Controller;
use game::zones::{Zone,Location};

// This is the players reprsentation in the game.
// Player owns the cards and the moves.
pub struct Player
{
    pub pidx: usize,
    pub name: String, 
    pub zones: ZoneCollection,
}

impl net::Networked for Player
{
    fn netid(&self) -> u64 { 0x100 + self.pidx as u64 }
}

impl Player
{
    pub fn new(pidx: usize, name: String) -> Player {
        Player { pidx, name, zones: ZoneCollection::new(42) }
    }

    pub fn name(&self) -> &str { self.name.as_str() }

    pub fn do_turn(&mut self, turn_count: usize) -> Option<u64>
    {
        info!("Player '{}' turn {} start.", self.name, turn_count);
        //self.pc.handle_user_input(self);
        None
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