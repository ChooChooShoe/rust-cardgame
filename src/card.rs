use std::collections::HashMap;
use serde::{Deserialize,Serialize};
use std::fmt;
#[derive(Deserialize,Serialize,Clone,Debug)]
pub struct Card
{
    netid: u64,
    name: String,
    text: String,
    tags: HashMap<TagKey,TagVal>

}
impl fmt::Display for Card
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{} ({} tags)", self.name,self.netid, self.tags.len())
    }
}
impl Card
{
    pub fn new(netid: u64, name: &str) -> Card
    {
        Card {
            netid,
            name: String::from(name),
            text: String::new(),
            tags: HashMap::with_capacity(8)
        }
    }
    
    pub fn tags(&mut self) -> &mut HashMap<TagKey,TagVal>
    {
        &mut self.tags
    }

    pub fn insert_tag(&mut self, key: TagKey, val: TagVal) -> Option<TagVal>
    {
        self.tags.insert(key, val)
    }

    pub fn remove_tag(&mut self, key: &TagKey) -> Option<TagVal> 
    {
        self.tags.remove(key)
    }

    pub fn get_tag_bool(&self, key: &TagKey) -> bool
    {
        match self.tags.get(key) {
            None => false,
            Some(x) => x.get_bool(),
        }
    }

    pub fn get_tag_i32(&self, key: &TagKey) -> i32
    {
        match self.tags.get(key) {
            None => 0,
            Some(x) => x.get_i32(),
        }
    }

    pub fn get_tag_f32(&self, key: &TagKey) -> f32
    {
        self.get_tag_f32_or(key, 0.0)
    }

    pub fn get_tag_f32_or(&self, key: &TagKey, or: f32) -> f32
    {
        match self.tags.get(key) {
            None => or,
            Some(x) => x.get_f32(or),
        }
    }

    pub fn is_tag_set(&self, key: &TagKey) -> bool
    {
        match self.tags.get(key) {
            None => false,
            Some(x) => x.is_set(),
        }
    }
}
const UNKNOWN_SID_NETID: u64 = 7;

pub struct CardPool
{
    pub all_cards : HashMap<String,Card>
}

impl CardPool
{
    pub fn new() -> CardPool {
        CardPool {
            all_cards: HashMap::new()
        }
    }
}

#[derive(Eq,PartialEq,Debug,Hash,Deserialize,Serialize,Clone)]
pub enum TagKey
{
    Cost,
    Attack,
    Health,
    Damage,
    

}

#[derive(PartialEq,Debug,Deserialize,Serialize,Clone)]
#[serde(untagged)]
/// Value that was set for a tag. 
/// One of i32, f32, or bool. 
pub enum TagVal {
    Int(i32),
    Float(f32),
    Bool(bool),
    /// TimedInt has a u16 for # of times sets.
    /// If t (u16) == 0 tag is effectivly unset.
    TimedInt(i32,u16),
}

impl TagVal {
    pub fn get_bool(&self) -> bool {
        match self {
            &TagVal::Bool(x) => x,
            &TagVal::Float(x) => x == 1.0,
            &TagVal::Int(x) => x != 0,
            &TagVal::TimedInt(x,t) => if t > 0 {x != 0} else {false},
        }
    }
    
    pub fn get_i32(&self) -> i32 {
        match self {
            &TagVal::Bool(x) => x as i32,
            &TagVal::Float(x) => x as i32,
            &TagVal::Int(x) => x,
            &TagVal::TimedInt(x,t) => if t > 0 {x} else {0},
        }
    }
    
    pub fn get_f32(&self, or: f32) -> f32 {
        match self {
            &TagVal::Bool(x) => if x {1.0} else {0.0},
            &TagVal::Float(x) => x,
            &TagVal::Int(x) => x as f32,
            &TagVal::TimedInt(x,t) => if t > 0 {x as f32} else {or},
        }
    }

    pub fn get_times_set(&self) -> u16 {
        match self {
            &TagVal::TimedInt(_,t) => t,
            _ => 1,
        }
    }

    pub fn is_set(&self) -> bool {
        match self {
            &TagVal::TimedInt(_,t) => t > 0,
            _ => true,
        }
    }
}