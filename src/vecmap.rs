pub struct VecMap<V> {
    inner: Vec<Option<V>>,
    count: usize,
}

impl<V> VecMap<V> {
    pub fn new() -> VecMap<V> {
        VecMap {
            inner: Vec::new(),
            count: 0,
        }
    }
    pub fn with_capacity(capacity: usize) -> VecMap<V> {
        VecMap {
            inner: Vec::with_capacity(capacity),
            count: 0,
        }
    }
    pub fn from_vec(vec: Vec<V>) -> VecMap<V> {
        let count = vec.len();
        let mut inner = Vec::with_capacity(count);
        for x in vec.into_iter() {
            inner.push(Some(x));
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

    /// Returns a reference to the value corresponding to the index.
    pub fn get(&self, index: usize) -> Option<&V> {
        match self.inner.get(index) {
            None => None,                  // index out of bounds
            Some(&None) => None,           // inboud, but no value set
            Some(&Some(ref v)) => Some(v), // inboud and set, return Some(ref)
        }
    }

    /// Will not panic if index is out of bounds
    pub fn insert(&mut self, index: usize, element: V) -> Option<V> {
        if index >= self.len() {
            for _ in 0..(self.len() - index) {
                self.inner.push(None)
            }
            self.inner.push(Some(element));
            None
        } else {
            self.inner.insert(index, Some(element));
            None
        }
    }
    /// Removes at index from the VecMap, returning the value at the  index.
    pub fn remove(&mut self, index: usize) -> Option<V> {
        if index >= self.len() {
            None
        } else {
            self.inner.remove(index)
        }
    }
    /// Appends an element to the back of a collection.
    pub fn push(&mut self, value: V) {
        self.inner.push(Some(value))
    }
    pub fn iter(&self) -> Iter<Option<V>> {
        self.inner.iter()
    }
    // TODO: the other map and vac functions.
}
use std::slice::Iter;
