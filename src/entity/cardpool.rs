use std::collections::HashMap;
use std::fmt;
use entity::card::{TagKey,TagVal,Card};
use std::io;
use std::io::ErrorKind;
use std::fs::{self,File};
use serde_json;
use std::borrow::Cow;
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
    pub fn tags(&self) -> &HashMap<TagKey,TagVal> {
        &self.tags
    }
    #[inline]
    pub fn tags_mut(&mut self) -> &mut HashMap<TagKey,TagVal> {
        &mut self.tags
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
    pub all_cards: HashMap<u64,Arc<CardData>>,
}

impl CardPool {
    pub fn new() -> CardPool {
        CardPool {
            all_cards: HashMap::new()
        }
    }

    pub fn get_clone(&self, card_id: u64) -> CardData {
        CardData::clone(self.all_cards.get(&card_id).unwrap())
    }
    pub fn try_get_clone(&self, card_id: u64) -> Option<CardData> {
        match self.all_cards.get(&card_id) {
            Some(s) => Some(CardData::clone(s)),
            None => None
        }
    }

    pub fn from_disk() -> io::Result<CardPool> {
        let file = File::open("./output/cards_out.json")?;
        let in_data: HashMap<u64,CardData> = serde_json::from_reader(file)?;
        let mut all_cards = HashMap::with_capacity(in_data.len());
        for (id,data) in in_data {
            all_cards.insert(id,Arc::new(data));
        }
        Ok(CardPool { all_cards })
    }
    pub fn gen_cards_to_disk() {
        let mut pool = CardPool::new();
        for i in 0..20
        {
            let mut c = ::entity::cardpool::CardData::new(&format!("Card #{:03}", i));
            c.tags_mut().insert(TagKey::Health, TagVal::from(9 + i));
            c.tags_mut().insert(TagKey::Attack, TagVal::from(7 + i));
            c.tags_mut().insert(TagKey::Cost, TagVal::from(3.5 * (i as f32)));
            c.tags_mut().insert(TagKey::Damage, TagVal::from(true));
            pool.all_cards.insert(10000 + i as u64, Arc::new(c));
        }

        pool.write_to_disk().expect("Unable to write to card database");
    }
    pub fn write_to_disk(&self) -> io::Result<()> {
        match fs::create_dir("./output/"){
            Ok(()) => {info!("Created 'output' directory.")},
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
        
        let mut out_data: HashMap<u64,CardData> = HashMap::with_capacity(self.all_cards.len());
        for (id,data) in &self.all_cards {
            //out_data.insert(*id, *data.clone());
        }

        serde_json::to_writer_pretty(writer, &out_data)?;
        Ok(())
    }
}