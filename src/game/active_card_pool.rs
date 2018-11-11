use crate::entity::{Card, CardKey, CardPool};
use crate::utils::vecmap::{IndexKey, IndexKeyAllocator, VecMap};

//#[derive(Clone, Default)]
pub struct ActiveCardPool {
    idxalloc: IndexKeyAllocator,
    cards: VecMap<Card>,
}

impl ActiveCardPool {
    pub fn new() -> ActiveCardPool {
        ActiveCardPool {
            idxalloc: IndexKeyAllocator::new(),
            cards: VecMap::new(),
        }
    }

    pub fn push_new(&mut self, card_name: &str) -> CardKey {
        let key = self.idxalloc.allocate();
        self.cards.insert(key, Card::new(key, card_name));
        key
    }
    pub fn remove(&mut self, key: CardKey) -> Option<Card> {
        if self.idxalloc.deallocate(key) {
            self.cards.remove(key)
        } else {
            None
        }
    }
    pub fn get(&self, key: CardKey) -> Option<&Card> {
        self.cards.get(key)
    }
    pub fn get_mut(&mut self, key: CardKey) -> Option<&mut Card> {
        self.cards.get_mut(key)
    }
}
