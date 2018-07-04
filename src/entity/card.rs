use entity::cardpool::PooledCardData;
use entity::{TagKey, TagVal};
use game::script::{Script, ScriptManager};
use game::GameScript;
use std::borrow::Cow;
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

pub type CardId = u64;

#[derive(Clone)]
pub struct Card {
    inner: Rc<Inner<'static>>,
}
struct Inner<'a> {
    uid: CardId,
    name: RefCell<String>,
    text: RefCell<String>,
    tags: RefCell<HashMap<TagKey, TagVal>>,
    script: RefCell<Cow<'a, Script>>,
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Card {{ uid: {}, name: {}, tags.len(): {}}}",
            self.uid(),
            self.name(),
            self.tags().len()
        )
    }
}
impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}#{:04} ({} tags)",
            self.name(),
            self.uid(),
            self.tags().len()
        )
    }
}

impl Card {
    pub fn new(uid: CardId, name: &str, text: &str, script: Cow<'static, Script>) -> Card {
        Card {
            inner: Rc::new(Inner {
                uid,
                name: RefCell::new(String::from(name)),
                text: RefCell::new(String::from(text)),
                tags: RefCell::new(HashMap::new()),
                script: RefCell::new(script),
            }),
        }
    }
    /// Creates a blank card with given id and name.
    pub fn from_string(uid: CardId, name: &str, text: &str) -> Card {
        Card::new(uid, name, text, Cow::Owned(Script::new(())))
    }
    /// Creates a known card using data from the cardpool.
    pub fn from_pool(uid: CardId, data: &PooledCardData) -> Card {
        Card {
            inner: Rc::new(Inner {
                uid,
                name: RefCell::new(String::from(data.name())),
                text: RefCell::new(String::from(data.text())),
                tags: RefCell::new(data.clone_tags()),
                script: RefCell::new(Cow::Borrowed(ScriptManager::get(data.script()))),
            }),
        }
    }

    #[inline]
    pub fn uid(&self) -> u64 {
        self.inner.uid
    }
    #[inline]
    pub fn name(&self) -> Ref<String> {
        self.inner.name.borrow()
    }
    #[inline]
    pub fn text(&self) -> Ref<String> {
        self.inner.text.borrow()
    }
    #[inline]
    pub fn tags(&self) -> Ref<HashMap<TagKey, TagVal>> {
        self.inner.tags.borrow()
    }
    #[inline]
    pub fn tags_mut(&mut self) -> RefMut<HashMap<TagKey, TagVal>> {
        self.inner.tags.borrow_mut()
    }
    #[inline]
    pub fn insert_tag(&mut self, key: TagKey, val: TagVal) -> Option<TagVal> {
        self.tags_mut().insert(key, val)
    }
    #[inline]
    pub fn remove_tag(&mut self, key: &TagKey) -> Option<TagVal> {
        self.tags_mut().remove(key)
    }

    pub fn get_tag(&self, key: &TagKey) -> TagVal {
        match self.tags().get(key) {
            Some(x) => x.clone(),
            None => TagVal::None,
        }
    }
    pub fn set_tag(&mut self, key: TagKey, val: TagVal) -> TagVal {
        match self.tags_mut().insert(key, val) {
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

    pub fn script(&self) -> Ref<Script> {
        self.inner.script.borrow()
    }
    pub fn script_mut(&self) -> RefMut<Script> {
        self.inner.script.borrow_mut().to_mut()
    }
}
