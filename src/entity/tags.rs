use std::collections::HashMap;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TagKey {
    Cost,
    BaseCost,
    Attack,
    BaseAttack,
    Health,
    BaseHealth,
    MaxHealth,
    Damage,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
/// Value that was set for a tag.
/// One of i32, f32, or bool.
pub enum TagVal {
    None,
    Int(i32),
    Float(f32),
    Bool(bool),
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
    /// Same as try_as_bool but retuens false instead of None.
    pub fn as_bool(&self) -> bool {
        match self {
            &TagVal::Bool(x) => x,
            &TagVal::Int(x) => x != 0,
            &TagVal::Float(x) => x != 0.0,
            _ => false,
        }
    }
    /// Converts this tag to a bool for Bool, Int, and Float tags.
    pub fn try_as_bool(&self) -> Option<bool> {
        match self {
            &TagVal::Bool(x) => Some(x),
            &TagVal::Int(x) => Some(x != 0),
            &TagVal::Float(x) => Some(x != 0.0),
            _ => None,
        }
    }
    /// Same as try_as_i32 but returns 0 instead of None
    pub fn as_i32(&self) -> i32 {
        match self {
            &TagVal::Bool(x) => x as i32,
            &TagVal::Int(x) => x,
            &TagVal::Float(x) => x.round() as i32,
            _ => 0,
        }
    }
    /// Converts this tag to an i32 for Bool, Int, and Float tags.
    pub fn try_as_i32(&self) -> Option<i32> {
        match self {
            &TagVal::Bool(x) => Some(x as i32),
            &TagVal::Int(x) => Some(x),
            &TagVal::Float(x) => Some(x.round() as i32),
            _ => None,
        }
    }
    /// Same as try_as_f32 but returns 0.0 instead of None
    pub fn as_f32(&self) -> f32 {
        match self {
            &TagVal::Bool(x) => x as i32 as f32,
            &TagVal::Int(x) => x as f32,
            &TagVal::Float(x) => x,
            _ => 0.0,
        }
    }
    /// Converts this tag to a f32 for Bool, Int, and Float tags.
    pub fn try_as_f32(&self) -> Option<f32> {
        match self {
            &TagVal::Bool(x) => Some(x as i32 as f32),
            &TagVal::Int(x) => Some(x as f32),
            &TagVal::Float(x) => Some(x),
            _ => None,
        }
    }
}
