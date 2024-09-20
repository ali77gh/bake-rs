use std::{collections::HashMap, hash::Hash};

pub struct OrderedMap<Key: Eq + Hash, Value> {
    data: Vec<Value>,
    positions: HashMap<Key, usize>,
}

impl<Key: Eq + Hash, Value> OrderedMap<Key, Value> {
    pub fn new() -> Self {
        OrderedMap {
            data: Vec::new(),
            positions: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: Key, value: Value) {
        let index = self.data.len();
        self.data.push(value);
        self.positions.insert(key, index);
    }

    pub fn get(&self, key: &Key) -> Option<&Value> {
        if let Some(x) = self.positions.get(key) {
            return self.data.get(*x);
        }
        None
    }

    pub fn get_all(&self) -> &[Value] {
        &self.data
    }

    pub fn get_at(&self, index: usize) -> Option<&Value> {
        self.data.get(index)
    }
}

impl<Key: Eq + Hash, Value> Default for OrderedMap<Key, Value> {
    fn default() -> Self {
        Self::new()
    }
}
