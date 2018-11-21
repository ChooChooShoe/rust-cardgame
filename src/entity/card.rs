use crate::entity::cardpool::{CardPool,PooledCardData};
use crate::entity::{Dispatch, TagKey, TagVal, Trigger};
use crate::game::player::Player;
use crate::game::script::{Script, ScriptManager};
use crate::utils::vecmap::IndexKey;
use std::collections::HashMap;
use std::fmt;

pub type CardKey = IndexKey;

//#[derive(Clone, Default, PartialEq)]
pub struct Card {
    //key: CardKey,
    name: String,
    text: String,
    tags: HashMap<TagKey, TagVal>,
    script: Script,
}


impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Card {{name: {}, tags.len(): {}}}",
            self.name,
            self.tags.len(),
        )
    }
}
impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}#{:04} ({} tags)",
            self.name(),
            0,//self.key(),
            self.tags_len()
        )
    }
}

impl Card {
    /// Creates a known card using data from the cardpool.
    pub fn from_pool(key: CardKey, data: &PooledCardData) -> Card {
        Card {
            name: (String::from(data.name())),
            text: (String::from(data.text())),
            tags: (data.clone_tags()),
            script: ScriptManager::get(data.script()),
        }
    }
    pub fn new(key: CardKey, name: &str) -> Card {
        if let Some(data) = CardPool::lookup_name(name) {
            Card::from_pool(key, data)
        } else {
            Card {
                name: (String::from("Unknown Card")),
                text: (String::from("")),
                tags: HashMap::new(),
                script: ScriptManager::get("none"),
            }
        }
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
    pub fn tags_len(&self) -> usize {
        self.tags.len()
    }

    pub fn get_tag(&self, key: &TagKey) -> TagVal {
        *self.tags.get(key).unwrap_or(&TagVal::None)
    }
    pub fn set_tag(&mut self, key: TagKey, val: TagVal) -> TagVal {
        self.tags
            .insert(key, val)
            .unwrap_or(TagVal::None)
    }

    pub fn base_attack(&self) -> i32 {
        self.get_tag(&TagKey::BaseAttack).as_i32()
    }
    pub fn current_attack(&self) -> i32 {
        self.get_tag(&TagKey::Attack).as_i32()
    }

    pub fn base_health(&self) -> i32 {
        self.get_tag(&TagKey::BaseHealth).as_i32()
    }
    pub fn current_health(&self) -> i32 {
        self.get_tag(&TagKey::Health).as_i32()
    }
    pub fn max_health(&self) -> i32 {
        self.get_tag(&TagKey::MaxHealth).as_i32()
    }
    pub fn missing_health(&self) -> i32 {
        self.max_health() - self.current_health()
    }

    pub fn base_cost(&self) -> i32 {
        self.get_tag(&TagKey::BaseCost).as_i32()
    }
    pub fn current_cost(&self) -> i32 {
        self.get_tag(&TagKey::Cost).as_i32()
    }

    pub fn on_card_drawn(&mut self, player: &mut Player) {
        info!("on_card_drawn!");
        Dispatch::broadcast(Trigger::OnCardDrawn(player, self))
    }
    pub fn after_card_drawn(&mut self, _player: &mut Player) {
        Dispatch::broadcast(Trigger::AfterCardDrawn(self))
    }
}
