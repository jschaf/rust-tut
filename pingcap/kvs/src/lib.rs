use std::collections::HashMap;

pub struct KvStore {
    map: HashMap<String, String>,
}

/// An in-memory key-value (KV) store.
///
/// # Examples
///
/// ```
/// use kvs::KvStore;
///
/// let mut store = KvStore::new();
/// store.set(String::from("a"), String::from("alpha"));
/// assert_eq!(store.get(String::from("a")), Some("alpha".to_string()));
///
/// store.remove(String::from("a"));
/// assert_eq!(store.get(String::from("a")), None);
/// ```
impl KvStore {
    pub fn new() -> KvStore {
        KvStore {
            map: HashMap::new(),
        }
    }

    /// Returns a clone of the value corresponding to the key.
    pub fn get(&self, key: String) -> Option<String> {
        self.map.get(key.as_str()).map(|s| s.clone())
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did have this key present, the value is updated.
    pub fn set(&mut self, key: String, value: String) {
        self.map.insert(key, value);
    }

    /// Removes a key from the map.
    pub fn remove(&mut self, key: String) {
        self.map.remove(key.as_str());
    }
}
