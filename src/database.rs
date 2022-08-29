use std::collections::HashMap;

use bytes::Bytes;

pub struct Database {
    // the value should be abstracted
    store: HashMap<String, Bytes>,
}

enum Error {
    Other,
}

impl Database {
    pub fn new() -> Self {
        Database {
            store: HashMap::new(),
        }
    }
    pub fn get<'a>(&'a self, key: &String) -> Option<Bytes> {
        self.store.get(key).map(|bs| bs.clone())
    }
    pub fn set(&mut self, key: String, val: Bytes) -> Option<Bytes> {
        self.store.insert(key, val)
    }
}
