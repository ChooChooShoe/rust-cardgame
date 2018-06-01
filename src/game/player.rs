use net;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::io;

use entity::Card;
use entity::CardPool;
use game::Deck;
use game::ZoneCollection;
use game::zones::ZoneName;
use game::zones::{Location, Zone};
use game::{ActionError, ActionResult, OkCode};
use player::controller::Controller;

// This is the players reprsentation in the game.
// Player owns the cards and the moves.
#[derive(Clone)]
pub struct Player {
    pub pidx: usize,
    pub name: String,
    pub deck: Option<Deck>,
    pub zones: ZoneCollection,
}

impl Player {
    pub fn new(pidx: usize, name: String) -> Player {
        Player {
            pidx,
            name,
            deck: None,
            zones: ZoneCollection::new(42),
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn do_turn(&mut self, turn_count: usize) -> Option<u64> {
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

    pub fn set_deck(&mut self, deck: Deck, pool: &CardPool, start_netid: u64) -> u64 {
        let mut id = start_netid;

        for entry in deck.cards_for_zone(ZoneName::Deck) {
            let zone = self.zones.get_mut(ZoneName::Deck);
            for _ in 0..entry.count() {
                zone.insert_at(Location::Default, pool.make_card(id, entry.card()));
                id += 1;
            }
        }
        self.deck = Some(deck);
        id
    }

    pub fn draw_x_cards(&mut self, x: usize) {
        if x == 0 { return }

        let drawn_cards = self.zones.deck.remove_x_at(x, Location::Top);

        //TODO on card drawn event
        for c in drawn_cards {
            match c {
                Some(card) => {
                    info!("on_card_drawn  event: deck -> hand : {:?}", card.name());
                    self.zones.hand.insert_at(Location::Top, card);
                    info!("after_card_drawn event:");
                }
                None => {
                    info!("on_card_draw_fail event: deck -> hand : NONE");
                }
            }
        }
    }
}
