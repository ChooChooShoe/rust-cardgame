use entity::cardpool::CardPool;
use entity::cardpool::CardData;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::sync::RwLock;

pub type CardId = [char; 8];

#[derive(Clone, Debug)]
pub struct Card {
    uid: u64,
    name: String,
    text: String,
    tags: HashMap<TagKey, TagVal>,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{} ({} tags)", self.name, self.uid, self.tags.len())
    }
}

impl Card {
    /// Creates a blank card with given id and name.
    pub fn from_string(uid: u64, name: &str, text: &str) -> Card {
        Card {
            uid,
            name: String::from(name),
            text: String::from(text),
            tags: HashMap::new(),
        }
    }
    /// Creates a known card using data from the cardpool. 
    pub fn from_pool(uid: u64, data: &CardData) -> Card {
        Card {
            uid,
            name: String::from(data.name()),
            text: String::from(data.text()),
            tags: data.clone_tags(),
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
    pub fn tags(&self) -> &HashMap<TagKey,TagVal> {
        &self.tags
    }
    #[inline]
    pub fn tags_mut(&mut self) -> &mut HashMap<TagKey,TagVal> {
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
}

#[derive(Eq, PartialEq, Debug, Hash, Deserialize, Serialize, Clone)]
pub enum TagKey {
    Cost,
    Attack,
    Health,
    Damage,
}

#[derive(PartialEq, Debug, Deserialize, Serialize, Clone)]
/// Value that was set for a tag.
/// One of i32, f32, or bool.
pub enum TagVal {
    None,
    Default,
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
impl Into<i32> for TagVal {
    fn into(self) -> i32 {
        self.as_i32()
    }
}
impl Into<f32> for TagVal {
    fn into(self) -> f32 {
        self.as_f32()
    }
}
impl Into<bool> for TagVal {
    fn into(self) -> bool {
        self.as_bool()
    }
}
impl TagVal {
    pub fn as_bool(&self) -> bool {
        match self {
            &TagVal::Bool(x) => x,
            &TagVal::Float(x) => x == 1.0,
            &TagVal::Int(x) => x != 0,
            &TagVal::Pair(x, _) => x != 0,
            _ => false
        }
    }
    pub fn as_i32(&self) -> i32 {
        match self {
            &TagVal::Bool(x) => x as i32,
            &TagVal::Float(x) => x as i32,
            &TagVal::Int(x) => x,
            &TagVal::Pair(x, _) => x,
            _ => 0
        }
    }
    pub fn as_f32(&self) -> f32 {
        match self {
            &TagVal::Bool(x) => if x { 1.0 } else { 0.0 },
            &TagVal::Float(x) => x,
            &TagVal::Int(x) => x as f32,
            &TagVal::Pair(x, _) => x as f32,
            _ => 0.0
        }
    }
}
