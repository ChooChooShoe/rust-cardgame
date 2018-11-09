//VecMap<T> sors data in a Vec<T> but sets and gets like a HashMap<usize,T>.

// See also: Generational indexes
// https://kyren.github.io/2018/09/14/rustconf-talk.html

use std::num::NonZeroU32;

type Idx = u32;
type Gen = NonZeroU32;
type OptGen = u32;

#[derive(Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct IndexKeyAllocator {
    free_indexes: Vec<Idx>,
    generation_entries: Vec<u32>,
}

impl IndexKeyAllocator {
    pub fn new() -> IndexKeyAllocator {
        IndexKeyAllocator {
            free_indexes: Vec::new(),
            generation_entries: Vec::new(),
        }
    }
    /// Returns the next avalable key.
    pub fn allocate(&mut self) -> IndexKey {
        if let Some(index) = self.free_indexes.pop() {
            // Reuse a 'free' index
            // This assumes that any free_index has a valid generation_entries entry.
            let gen = self.generation_entries.get_mut(index as usize).unwrap();
            // Increment to activate.
            *gen += 1;
            IndexKey::new(index, *gen)
        } else {
            // All IndexKeys are in use. Make a new one.
            let index = self.generation_entries.len() as u32;
            self.generation_entries.push(1);
            IndexKey::new_first_gen(index)
        }
    }

    /// Returns true if the index was allocated before and is now deallocated
    /// A deallocated key will be reused in a future allocate with a greater generation.
    pub fn deallocate(&mut self, key: IndexKey) -> bool {
        if self.is_live(key) {
            if let Some(gen) = self.generation_entries.get_mut(key.index()) {
                // Increment to deactivate.
                *gen += 1;
                // Add to free to be reused.
                self.free_indexes.push(key.index);
                debug_assert_eq!(*gen % 2, 0); // gen is even == true
                debug_assert_eq!(self.is_live(key), false);
                true
            } else {
                // The key was never allocated in the first place.
                debug!(
                    "Attempted to deallocate key '{:?}' that was not allocated with this IndexKeyAllocator.",
                    key
                );
                false
            }
        } else {
            false
        }
    }

    /// Tests the key to see if it is live with this allocater.
    /// Returns true if the key is 'live' and it's generation matches the active entry's generation.
    /// Generation 0 is not-live. (and not allocated)
    /// Any even generation is not-live. (2,4,6...)
    /// Any odd generation is live. (1,3,5...)
    pub fn is_live(&self, key: IndexKey) -> bool {
        if let Some(gen) = self.generation_entries.get(key.index()) {
            if *gen == key.generation() {
                return *gen % 2 == 1; // true if gen is odd.
            }
        }
        false
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct IndexKey {
    generation: Gen,
    index: Idx,
}

impl IndexKey {
    pub fn new(index: Idx, generation: u32) -> IndexKey {
        assert!(generation != 0);
        IndexKey {
            generation: Gen::new(generation).unwrap(),
            index: index,
        }
    }
    pub fn new_first_gen(index: Idx) -> IndexKey {
        IndexKey {
            generation: Gen::new(1).unwrap(),
            index,
        }
    }
    pub fn with_generation(index: Idx, generation: Gen) -> IndexKey {
        IndexKey { generation, index }
    }
    pub fn index(&self) -> usize {
        self.index as usize
    }
    pub fn generation(&self) -> u32 {
        self.generation.get()
    }
    pub fn is_active(&self) -> bool {
        self.generation.get() % 2 == 1
    }
}

//#[derive(Clone, Eq, PartialEq)]
enum Entry<T> {
    Empty,
    Full(NonZeroU32, T),
}
impl<T> Entry<T> {
    /// Creates an empty ventry
    fn empty() -> Entry<T> {
        Entry::Empty
    }
    /// Creates new entry. It's generation is 1.
    fn first(value: T) -> Entry<T> {
        Entry::Full(NonZeroU32::new(1).unwrap(), value)
    }
    /// Creates new entry. Given generation must not be 0.
    fn from(value: T, generation: NonZeroU32) -> Entry<T> {
        Entry::Full(generation, value)
    }
    fn generation(&self) -> u32 {
        match self {
            Entry::Full(gen, _) => gen.get(),
            Entry::Empty => 0,
        }
    }
    fn unwrap(self) -> T {
        match self {
            Entry::Full(_, value) => value,
            Entry::Empty => panic!("Empty Entry has no value on Entry::unwrap()"),
        }
    }
    fn value(self) -> Option<T> {
        match self {
            Entry::Full(_, value) => Some(value),
            Entry::Empty => None,
        }
    }
}

impl<T> Default for Entry<T> {
    fn default() -> Entry<T> {
        Entry::Empty
    }
}

//#[derive(Clone, Eq, PartialEq)]
pub struct VecMap<T> {
    inner: Vec<Entry<T>>,
    count: usize,
}
impl<T> Default for VecMap<T> {
    fn default() -> VecMap<T> {
        VecMap::new()
    }
}

impl<T> VecMap<T> {
    pub fn new() -> VecMap<T> {
        Self::with_capacity(0)
    }
    pub fn with_capacity(capacity: usize) -> VecMap<T> {
        VecMap {
            inner: Vec::with_capacity(capacity),
            count: 0,
        }
    }
    pub fn from_vec(vec: Vec<T>) -> VecMap<T> {
        let mut inner = Vec::with_capacity(vec.len());
        let mut count = 0;
        for x in vec.into_iter() {
            count += 1;
            inner.push(Entry::first(x));
        }
        VecMap { inner, count }
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
    /// This is the ditance from the first to the last element.
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    /// This is number of non-empty elements.
    pub fn count(&self) -> usize {
        self.count
    }
    /// True of there are not elements or only empty elements.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    /// Returns a reference to the value corresponding to the generational index.
    /// The generation must match.
    pub fn get(&self, index: IndexKey) -> Option<&T> {
        match self.inner.get(index.index()) {
            None => None,               // index out of bounds
            Some(Entry::Empty) => None, // inbound, but no value set
            Some(Entry::Full(gen, value)) => {
                // inbound and set, return Some(value) if generations match.
                if gen.get() == index.generation() {
                    Some(value)
                } else {
                    None
                }
            }
        }
    }

    /// Returns a mutable reference to the value corresponding to the generational index.
    /// The generation must match.
    pub fn get_mut(&mut self, index: IndexKey) -> Option<&mut T> {
        match self.inner.get_mut(index.index()) {
            None => None,               // index out of bounds
            Some(Entry::Empty) => None, // inbound, but no value set
            Some(Entry::Full(gen, value)) => {
                // inbound and set, return Some(value) if generations match.
                if gen.get() == index.generation() {
                    Some(value)
                } else {
                    // Gens are not the same, value is not the correct value.
                    None
                }
            }
        }
    }

    /// Set the value for some generational index. May overwrite past generation values.
    /// The overwritten value is returned.
    pub fn insert(&mut self, index: IndexKey, value: T) -> Option<T> {
        if index.index() < self.len() {
            // Count +1 if we are not replacing a value.
            if let Entry::Empty = self.inner[index.index()] {
                self.count += 1;
            }
            self.inner.push(Entry::from(value, index.generation));
            self.inner.swap_remove(index.index()).value()
        } else {
            let diff = self.len() - index.index();
            for _ in 0..diff {
                self.inner.push(Entry::Empty)
            }
            debug_assert_eq!(self.len(), index.index());
            self.count += 1;
            self.inner.push(Entry::from(value, index.generation));
            None
        }
    }
    /// Removes the value at the matching key.
    pub fn remove(&mut self, index: IndexKey) -> Option<T> {
        if index.index() < self.len() {
            self.count -= 1;
            self.inner.push(Entry::Empty);
            self.inner.swap_remove(index.index()).value()
        } else {
            None
        }
    }
}
