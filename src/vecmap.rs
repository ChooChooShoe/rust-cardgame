//VecMap<T> sors data in a Vec<T> but sets and gets like a HashMap<usize,T>.

// See also: Generational indexes
// https://kyren.github.io/2018/09/14/rustconf-talk.html

use std::num::NonZeroU32;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct IndexKey {
    index: u32,
    generation: NonZeroU32,
}

impl IndexKey {
    pub fn new(index: u32, generation: u32) -> IndexKey {
        IndexKey {
            index,
            generation: NonZeroU32::new(generation).unwrap(),
        }
    }
    pub fn from_nonzero(index: u32, generation: NonZeroU32) -> IndexKey {
        IndexKey { index, generation }
    }
    pub fn index(&self) -> usize {
        self.index as usize
    }
    pub fn generation(&self) -> u32 {
        self.generation.get()
    }
}

struct Entry<T> {
    generation: NonZeroU32,
    value: T,
}
impl<T> Entry<T> {
    // Creates new entry. generation must not be 0.
    fn new(value: T, generation: NonZeroU32) -> Entry<T> {
        Entry { value, generation }
    }
    fn first(value: T) -> Entry<T> {
        Entry { value, generation: NonZeroU32::new(1).unwrap() }
    }

    fn generation(&self) -> u32 {
        self.generation.get()
    }
}

pub struct VecMap<T> {
    inner: Vec<Option<Entry<T>>>,
    free_head: usize,
    count: usize,
}

impl<T> VecMap<T> {
    pub fn new() -> VecMap<T> {
        Self::with_capacity(0)
    }
    pub fn with_capacity(capacity: usize) -> VecMap<T> {
        VecMap {
            inner: Vec::with_capacity(capacity),
            free_head: 0,
            count: 0,
        }
    }
    pub fn from_vec(vec: Vec<T>) -> VecMap<T> {
        let mut inner = Vec::with_capacity(vec.len());
        for x in vec.into_iter() {
            inner.push(Some(Entry::first(x)));
        }
        VecMap {
            inner,
            free_head: 0,
            count: 0,
        }
    }
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional)
    }
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit()
    }
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn count(&self) -> usize {
        self.count
    }
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    // Returns a reference to the value corresponding to the generational index.
    // The generation must match.
    pub fn get(&self, index: IndexKey) -> Option<&T> {
        match self.inner.get(index.index()) {
            None => None,       // index out of bounds
            Some(None) => None, // inbound, but no value set
            Some(Some(entry)) => {
                // inbound and set, return Some(value) if generations match.
                if entry.generation() == index.generation() {
                    Some(&entry.value)
                } else {
                    None
                }
            }
        }
    }

    pub fn get_mut(&mut self, index: IndexKey) -> Option<&mut T> {
        match self.inner.get_mut(index.index()) {
            None => None,       // index out of bounds
            Some(None) => None, // inbound, but no value set
            Some(Some(entry)) => {
                // inbound and set, return Some(value) if generations match.
                if entry.generation() == index.generation() {
                    Some(&mut entry.value)
                } else {
                    None
                }
            }
        }
    }

    // Set the value for some generational index.  May overwrite past generation
    // values.
    pub fn set(&mut self, index: IndexKey, value: T) {
        if index.index() < self.len() {
            self.inner[index.index()] = Some(Entry::new(value, index.generation))
        } else {
            for _ in 0..(self.len() - index.index()) {
                self.inner.push(None)
            }
            assert_eq!(self.len(), index.index());
            self.inner.push(Some(Entry::new(value, index.generation)))
        }
    }


    pub fn insert(&mut self, value: T) -> IndexKey
    {
        let key = IndexKey::new(self.len() as u32, 1);
        self.set(key,value);
        key
    }
}
