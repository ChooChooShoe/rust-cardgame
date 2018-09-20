use crate::entity::cardpool::PooledCardData;
use crate::entity::{Dispatch, TagKey, TagVal, Trigger};
use crate::game::player::Player;
use crate::game::script::{Script, ScriptManager};
use crate::game::GameScript;
use std::borrow::Cow;
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

pub type CardId = u64;

#[derive(Clone)]
pub struct Card {
    inner: Rc<RefCell<CardData>>,
}
pub struct CardData {
    uid: CardId,
    name: String,
    text: String,
    tags: HashMap<TagKey, TagVal>,
    script: Script,
}
impl ToOwned for CardData {
    type Owned = CardData;
    fn to_owned(&self) -> CardData {
        CardData {
            uid: self.uid,
            name: self.name.clone(),
            text: self.text.clone(),
            tags: self.tags.clone(),
            script: self.script.clone(),
        }
    }
}

impl CardData {
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
    #[inline]
    pub fn get_tag(&self, key: &TagKey) -> Option<&TagVal> {
        self.tags.get(key)
    }
    #[inline]
    pub fn set_tag(&mut self, key: TagKey, val: TagVal) -> Option<TagVal> {
        self.tags.insert(key, val)
    }
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Card {{ uid: {}, name: {}, tags.len(): {}}}",
            self.uid(),
            self.data().name(),
            self.data().tags_len(),
        )
    }
}
impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}#{:04} ({} tags)",
            self.data().name(),
            self.uid(),
            self.data().tags_len()
        )
    }
}

impl Card {
    pub fn new(uid: CardId, name: &str, text: &str, script: Script) -> Card {
        Card {
            inner: Rc::new(RefCell::new(CardData {
                uid,
                name: (String::from(name)),
                text: (String::from(text)),
                tags: (HashMap::new()),
                script: script,
            })),
        }
    }
    /// Creates a known card using data from the cardpool.
    pub fn from_pool(uid: CardId, data: &PooledCardData) -> Card {
        Card {
            inner: Rc::new(RefCell::new(CardData {
                uid,
                name: (String::from(data.name())),
                text: (String::from(data.text())),
                tags: (data.clone_tags()),
                script: ScriptManager::get(data.script()),
            })),
        }
    }

    #[inline]
    pub fn uid(&self) -> u64 {
        self.inner.borrow().uid
    }
    #[inline]
    pub fn data(&self) -> Ref<CardData> {
        self.inner.borrow()
    }
    #[inline]
    pub fn data_mut(&self) -> RefMut<CardData> {
        self.inner.borrow_mut()
    }

    #[inline]
    pub fn script(&self) -> Ref<CardData> {
        self.inner.borrow()
    }

    pub fn get_tag(&self, key: &TagKey) -> TagVal {
        *self.data().tags.get(key).unwrap_or(&TagVal::None)
    }
    pub fn set_tag(&mut self, key: TagKey, val: TagVal) -> TagVal {
        self.data_mut().tags.insert(key,val).unwrap_or(TagVal::None)
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
