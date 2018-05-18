use entity::card::{Card};
use entity::{TagKey,TagVal};
use serde_json;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::fs::{self, File};
use std::io;
use std::io::ErrorKind;
use std::sync::Arc;
use std::sync::RwLock;

/// This is all the data needed to create a card
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct CardData {
    name: String,
    text: String,
    tags: HashMap<TagKey, TagVal>,
}
impl CardData {
    pub fn new(name: &str) -> CardData {
        CardData {
            name: String::from(name),
            text: String::new(),
            tags: HashMap::with_capacity(8),
        }
    }

    #[inline]
    pub fn clone_tags(&self) -> HashMap<TagKey, TagVal> {
        self.tags.clone()
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn text(&self) -> &str {
        &self.text
    }
}

pub struct CardPool {
    all_cards: HashMap<String, CardData>,
}

impl CardPool {
    /// Makes a Card from the shared name or generates an 'Unknown Card' if name is not known.
    pub fn make_card(&self, uid: u64, name: &str) -> Card {
        match self.all_cards.get(name) {
            Some(s) => Card::from_pool(uid, s),
            None => Card::from_string(uid, "Unknown Card", &format!("No card named '{}'", name)),
        }
    }
    // Makes a Card from the shared name only if the name is known.
    pub fn try_make_card(&self, uid: u64, name: &str) -> Option<Card> {
        match self.all_cards.get(name) {
            Some(s) => Some(Card::from_pool(uid, s)),
            None => None,
        }
    }

    pub fn from_disk() -> io::Result<CardPool> {
        let file = File::open("./output/cards_out.json")?;
        let in_data: HashMap<String, CardData> = serde_json::from_reader(file)?;
        Ok(CardPool { all_cards: in_data })
    }
    pub fn gen_cards_to_disk() {
        let mut pool = CardPool {
            all_cards: HashMap::with_capacity(20),
        };
        for i in 0..20 {
            let mut tags = HashMap::new();
            tags.insert(TagKey::Health, TagVal::from(9 + i));
            tags.insert(TagKey::Attack, TagVal::from(4 + i));
            tags.insert(TagKey::Cost, TagVal::from(3.5 * (i as f32)));
            tags.insert(TagKey::Damage, TagVal::None);
            pool.all_cards.insert(
                format!("GEN{:03}", 10000 + i),
                CardData {
                    name: format!("Card #{:03}", i),
                    text: format!("No Text"),
                    tags,
                },
            );
        }

        pool.write_to_disk()
            .expect("Unable to write to card database");
    }
    pub fn write_to_disk(&self) -> io::Result<()> {
        match fs::create_dir("./output/") {
            Ok(()) => info!("Created 'output' directory."),
            Err(e) => {
                if e.kind() == ErrorKind::AlreadyExists {
                    info!("The 'output' directory already exists.");
                    return Ok(());
                } else {
                    warn!("Could not create 'output' directory. Error: {}", e)
                }
            }
        }
        let writer = File::create("./output/cards_out.json")?;
        serde_json::to_writer_pretty(writer, &self.all_cards)?;
        Ok(())
    }
}
