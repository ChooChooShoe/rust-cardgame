use entity::cardpool::CardData;
use entity::cardpool::CardPool;
use entity::{TagKey,TagVal};
use game::GameScript;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::sync::RwLock;

pub type CardId = [char; 8];

pub struct Card {
    uid: u64,
    name: String,
    text: String,
    tags: HashMap<TagKey, TagVal>,
    script: Box<GameScript>,
}

impl Clone for Card {
    fn clone(&self) -> Card {
        Card {
            uid: self.uid,
            name: self.name.clone(),
            text: self.text.clone(),
            tags: self.tags.clone(),
            script: self.script.box_clone(),
        }
    }
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Card {{ uid: {}, name: {}, tags.len(): {}}}", self.uid, self.name, self.tags.len())
    }
}
impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}#{:04} ({} tags)", self.name, self.uid, self.tags.len())
    }
}

impl Card {
    pub fn new(uid: u64, name: &str, text: &str, script: Box<GameScript>) -> Card
    {
        Card {
            uid,
            name: String::from(name),
            text: String::from(text),
            tags: HashMap::new(),
            script: script,
        }
    }
    /// Creates a blank card with given id and name.
    pub fn from_string(uid: u64, name: &str, text: &str) -> Card {
        Card::new(uid,name,text,Box::new(()))
    }
    /// Creates a known card using data from the cardpool.
    pub fn from_pool(uid: u64, data: &CardData) -> Card {
        Card {
            uid,
            name: String::from(data.name()),
            text: String::from(data.text()),
            tags: data.clone_tags(),
            script: Box::new(()),
        }
    }

    #[inline]
    pub fn uid(&self) -> u64 {
        self.uid
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
    pub fn unsafe_tags(&self) -> &HashMap<TagKey, TagVal> {
        &self.tags
    }
    #[inline]
    pub fn unsafe_tags_mut(&mut self) -> &mut HashMap<TagKey, TagVal> {
        &mut self.tags
    }
    #[inline]
    pub fn insert_tag(&mut self, key: TagKey, val: TagVal) -> Option<TagVal> {
        self.tags.insert(key, val)
    }
    #[inline]
    pub fn remove_tag(&mut self, key: &TagKey) -> Option<TagVal> {
        self.tags.remove(key)
    }

    pub fn get_tag(&self, key: &TagKey) -> &TagVal {
        match self.tags.get(key) {
            Some(x) => x,
            None => &TagVal::None,
        }
    }
    pub fn set_tag(&mut self, key: TagKey, val: TagVal) -> TagVal {
        match self.tags.insert(key, val) {
            Some(x) => x,
            None => TagVal::None,
        }
    }

    pub fn cost(&self) -> i32 {
        self.get_tag(&TagKey::Cost).as_i32()
    }
    pub fn set_cost(&mut self, v: i32) -> i32 {
        self.set_tag(TagKey::Cost, TagVal::from(v)).as_i32()
    }

    pub fn script(&mut self) -> &mut Box<GameScript> {
        &mut self.script
    }
}