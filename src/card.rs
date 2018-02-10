use std::collections::HashMap;
use serde::{Deserialize,Serialize};

#[derive(Deserialize,Serialize)]
pub struct Card
{
    netid: u64,
    name: String,
    text: String,
    tags: HashMap<TagKey,TagVal>

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

    pub fn set_tag(&mut self, key: TagKey, val: TagVal) -> Option<TagVal>
    {
        self.tags.insert(key, val)
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
            Some(x) => x.get_f32(),
        }
    }
}

#[derive(Eq,PartialEq,Debug,Hash,Deserialize,Serialize)]
pub enum TagKey
{
    Cost,
    Attack,
    Health,
    Damage,
    

}

#[derive(PartialEq,Debug,Deserialize,Serialize)]
//#[serde(untagged)]
pub enum TagVal
{
    Int(i32),
    Float(f32),
    Bool(bool),
}

impl TagVal
{

    pub fn get_bool(&self) -> bool
    {
        match(self)
        {
            &TagVal::Bool(x) => x,
            &TagVal::Float(x) => x == 1.0,
            &TagVal::Int(x) => x != 0,
        }
    }
    
    pub fn get_i32(&self) -> i32
    {
        match(self)
        {
            &TagVal::Bool(x) => x as i32,
            &TagVal::Float(x) => x as i32,
            &TagVal::Int(x) => x,
        }
    }
    
    pub fn get_f32(&self) -> f32
    {
        match(self)
        {
            &TagVal::Bool(x) => if x {1.0} else {0.0},
            &TagVal::Float(x) => x,
            &TagVal::Int(x) => x as f32,
        }
    }
}