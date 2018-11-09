use crate::entity::{Card, CardPool, CardKey};
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

    fn push_new(&mut self, card: Card) -> CardKey {
        let key = self.idxalloc.allocate();
        self.cards.insert(key, card);
        key
    }
    fn remove(&mut self, key: CardKey) -> bool {
        if self.idxalloc.deallocate(key) {
            self.cards.remove(key);
            true
        } else {
            false
        }
    }
}
