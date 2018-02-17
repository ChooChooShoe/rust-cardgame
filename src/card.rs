use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Card {
    netid: u64,
    name: String,
    text: String,
    tags: HashMap<TagKey, TagVal>,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{} ({} tags)", self.name, self.netid, self.tags.len())
    }
}

impl Card {
    pub fn new(netid: u64, name: &str) -> Card {
        Card {
            netid,
            name: String::from(name),
            text: String::new(),
            tags: HashMap::with_capacity(8),
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn tags(&mut self) -> &mut HashMap<TagKey, TagVal> {
        &mut self.tags
    }

    pub fn insert_tag(&mut self, key: TagKey, val: TagVal) -> Option<TagVal> {
        self.tags.insert(key, val)
    }

    pub fn remove_tag(&mut self, key: &TagKey) -> Option<TagVal> {
        self.tags.remove(key)
    }

    pub fn to_tag_bool(&self, key: &TagKey) -> bool {
        match self.tags.get(key) {
            None => false,
            Some(x) => x.to_bool(),
        }
    }

    pub fn get_tag_i32(&self, key: &TagKey) -> i32 {
        match self.tags.get(key) {
            None => 0,
            Some(x) => x.to_i32(),
        }
    }

    pub fn get_tag_f32(&self, key: &TagKey) -> f32 {
        match self.tags.get(key) {
            None => 0.0,
            Some(x) => x.to_f32(),
        }
    }

    pub fn get_tag_f32_or(&self, key: &TagKey, or: f32) -> f32 {
        match self.tags.get(key) {
            None => or,
            Some(x) => x.to_f32(),
        }
    }

    pub fn is_tag_set(&self, key: &TagKey) -> bool {
        match self.tags.get(key) {
            None => false,
            Some(x) => true,
        }
    }
}
const UNKNOWN_SID_NETID: u64 = 7;

pub struct CardPool {
    pub all_cards: HashMap<String, Card>,
}

impl CardPool {
    pub fn new() -> CardPool {
        CardPool {
            all_cards: HashMap::new(),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Hash, Deserialize, Serialize, Clone)]
pub enum TagKey {
    Cost,
    Attack,
    Health,
    Damage,
}

#[derive(PartialEq, Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
/// Value that was set for a tag.
/// One of i32, f32, or bool.
pub enum TagVal {
    Int(i32),
    Float(f32),
    Bool(bool),
    Pair(i32, u16),
}

impl From<i32> for TagVal {
    fn from(num: i32) -> Self {
        TagVal::Int(num)
    }
}
impl From<f32> for TagVal {
    fn from(num: f32) -> Self {
        TagVal::Float(num)
    }
}
impl From<bool> for TagVal {
    fn from(b: bool) -> Self {
        TagVal::Bool(b)
    }
}

impl TagVal {
    pub fn to_bool(&self) -> bool {
        match self {
            &TagVal::Bool(x) => x,
            &TagVal::Float(x) => x == 1.0,
            &TagVal::Int(x) => x != 0,
            &TagVal::Pair(x, _) => x != 0,
        }
    }

    pub fn to_i32(&self) -> i32 {
        match self {
            &TagVal::Bool(x) => x as i32,
            &TagVal::Float(x) => x as i32,
            &TagVal::Int(x) => x,
            &TagVal::Pair(x, _) => x,
        }
    }

    pub fn to_f32(&self) -> f32 {
        match self {
            &TagVal::Bool(x) => if x { 1.0 } else { 0.0 },
            &TagVal::Float(x) => x,
            &TagVal::Int(x) => x as f32,
            &TagVal::Pair(x, _) => x as f32,
        }
    }
}
