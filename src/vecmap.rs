pub struct VecMap<V>(Vec<Option<V>>);


impl<V> VecMap<V> {
    pub fn new() -> VecMap<V> {
        VecMap(Vec::new())
    }
    pub fn with_capacity(capacity: usize) -> VecMap<V> {
        VecMap(Vec::with_capacity(capacity))
    }
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional)
    }
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn clear(&mut self) {
        self.0.clear()
    }

    /// Returns a reference to the value corresponding to the index.
    pub fn get(&self, index: usize) -> Option<&V> {
        match self.0.get(index) {
            None => None,                   // index out of bounds
            Some(&None) => None,            // inboud, but no value set
            Some(&Some(ref v)) => Some(v)   // inboud and set, return Some(ref)
        }
    }

    /// Will not panic if index is out of bounds
    pub fn insert(&mut self, index: usize, element: V) -> Option<V> {
        if index >= self.len() {
            for _ in 0..(self.len() - index){
                self.0.push(None)
            }
            self.0.push(Some(element));
            None
        } else {
            self.0.insert(index, Some(element));
            None
        }
    }
    /// Removes at index from the VecMap, returning the value at the  index.
    pub fn remove(&mut self, index: usize) -> Option<V> {
        if index >= self.len() {
            None
        } else {
            self.0.remove(index)
        }
    }
    /// Appends an element to the back of a collection.
    pub fn push(&mut self, value: V) {
        self.0.push(Some(value))
    }

    // TODO: the other map and vac functions.
}