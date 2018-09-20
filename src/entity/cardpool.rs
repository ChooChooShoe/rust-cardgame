use crate::entity::card::{Card, CardId};
use crate::game::script::Script;
use crate::entity::{TagKey, TagVal};
use serde_json;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::fs::{self, File};
use std::io;
use std::io::ErrorKind;
use std::sync::Arc;
use std::sync::Mutex;

/// This is all the data needed to create a card
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PooledCardData {
    name: String,
    text: String,
    script: String,
    tags: HashMap<TagKey, TagVal>,
}
impl PooledCardData {
    pub fn new(name: &str) -> PooledCardData {
        PooledCardData {
            name: String::from(name),
            text: String::new(),
            script: String::from("none"),
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
    #[inline]
    pub fn script(&self) -> &str {
        &self.script
    }
}

lazy_static! {
    static ref INSTANCE: Mutex<CardPool> = Mutex::new(CardPool {
        all_cards: HashMap::new(),
        last_instance_id: 0,
    });
}
pub struct CardPool {
    all_cards: HashMap<String, PooledCardData>,
    last_instance_id: u64,
}

impl CardPool {
    /// Makes a Card from the shared name or generates an 'Unknown Card' if name is not known.
    /// The CardId is set not set by the pool.
    pub fn make_card_with_id(id: CardId, name: &str) -> Card {
        let pool = INSTANCE.lock().unwrap();
        match pool.all_cards.get(name) {
            Some(s) => Card::from_pool(id, s),
            None => Card::new(
                id,
                "Unknown Card",
                &format!("No card named '{}'", name),
                Script::None,
            ),
        }
    }
    /// Makes a Card from the shared name or generates an 'Unknown Card' if name is not known.
    pub fn make_card(name: &str) -> Card {
        let mut pool = INSTANCE.lock().unwrap();
        pool.last_instance_id += 1;
        match pool.all_cards.get(name) {
            Some(s) => Card::from_pool(pool.last_instance_id, s),
            None => Card::new(
                pool.last_instance_id,
                "Unknown Card",
                &format!("No card named '{}'", name),
                Script::None,
            ),
        }
    }
    // Makes a Card from the shared name only if the name is known.
    pub fn try_make_card(name: &str) -> Option<Card> {
        let mut pool = INSTANCE.lock().unwrap();
        let res = match pool.all_cards.get(name) {
            Some(s) => Some(Card::from_pool(pool.last_instance_id, s)),
            None => None,
        };
        if res.is_some() {
            pool.last_instance_id += 1;
        }
        res
    }

    pub fn from_disk() -> io::Result<CardPool> {
        let file = File::open("./output/cards_out.json")?;
        let in_data: HashMap<String, PooledCardData> = serde_json::from_reader(file)?;
        Ok(CardPool {
            all_cards: in_data,
            last_instance_id: 0,
        })
    }
    fn new() -> CardPool {
        let mut pool = CardPool {
            all_cards: HashMap::with_capacity(20),
            last_instance_id: 0,
        };
        for i in 0..20 {
            let mut tags = HashMap::new();
            tags.insert(TagKey::Health, TagVal::from(9 + i));
            tags.insert(TagKey::Attack, TagVal::from(4 + i));
            tags.insert(TagKey::Cost, TagVal::from(3.5 * (i as f32)));
            tags.insert(TagKey::Damage, TagVal::None);
            pool.all_cards.insert(
                format!("GEN{:03}", 10000 + i),
                PooledCardData {
                    name: format!("Card #{:03}", i),
                    text: format!("No Text"),
                    script: String::from("none"),
                    tags,
                },
            );
        }
        pool
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
