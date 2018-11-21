use crate::entity::{Dispatch, Trigger};
use crate::game::zones::{Location, Zone, ZoneName};
use crate::game::ZoneCollection;
use crate::game::{Deck, PlayerId};

// This is the players reprsentation in the game.
// Player owns the cards and the moves.
#[derive(Clone)]
pub struct Player {
    pub player_id: PlayerId,
    pub name: String,
    pub deck: Option<Deck>,
    pub zones: ZoneCollection,
}

impl Player {
    pub fn new(player_id: PlayerId, name: String) -> Player {
        Player {
            player_id,
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
                //TODO zone.insert_at(Location::Default, CardPool::make_card(entry.card()));
            }
        }
        self.deck = Some(deck);
    }

    pub fn draw_x_cards(&mut self, x: usize) {
        if x == 0 {
            return;
        }

        let drawn_cards = self.zones.deck.remove_x_at(x, Location::Top);

        for c in drawn_cards {
            match c {
                Some(mut card) => {
                    //TODO card.on_card_drawn(self);
                    self.zones.hand.insert_at(Location::Top, card);
                    //card_moved.after_card_drawn(self);
                }
                None => {
                    Dispatch::broadcast(Trigger::OnCardDrawFail(self));
                }
            }
        }
    }
}
