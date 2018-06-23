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
use entity::Dispatch;
use entity::Trigger;

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

    // Sets the players starting deck and fills, return the old startng deck if one exists.
    pub fn set_deck(&mut self, deck: Deck) {
        let zone = self.zones.get_mut(ZoneName::Deck);
        for entry in deck.cards_for_zone(ZoneName::Deck) {
            for _ in 0..entry.count() {
                zone.insert_at(Location::Default, CardPool::make_card(entry.card()));
            }
        }
        self.deck = Some(deck);
    }

    pub fn draw_x_cards(&mut self, x: usize) {
        if x == 0 { return }

        let drawn_cards = self.zones.deck.remove_x_at(x, Location::Top);

        //TODO on card drawn event
        for c in drawn_cards {
            match c {
                Some(mut card) => {
                    Dispatch::broadcast(Trigger::OnCardDrawn(self, &mut card));
                    let mut card_moved = self.zones.hand.insert_at(Location::Top, card);
                    Dispatch::broadcast(Trigger::AfterCardDrawn(&mut card_moved));
                }
                None => {
                    Dispatch::broadcast(Trigger::OnCardDrawFail(self));
                }
            }
        }
    }
}
