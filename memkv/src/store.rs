//! Thread-safe in-memory key-value store.
//!
//! Backed by a `HashMap` protected by an `RwLock` so that multiple readers
//! can access the store concurrently while a single writer holds exclusive
//! access — a direct illustration of the read/write lock pattern discussed
//! in Chapter 7 of *Designing Data-Intensive Applications*.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A cloneable handle to the shared in-memory store.
///
/// Cloning a `Store` produces another handle to the *same* underlying data,
/// making it cheap to hand out to each connection-handler thread.
#[derive(Clone)]
pub struct Store {
    inner: Arc<RwLock<HashMap<String, String>>>,
}

impl Store {
    /// Create a new, empty store.
    pub fn new() -> Self {
        Store {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Insert or update `key` with `value`. Returns the previous value, if any.
    pub fn set(&self, key: String, value: String) -> Option<String> {
        self.inner
            .write()
            .expect("store lock poisoned")
            .insert(key, value)
    }

    /// Return the value stored under `key`, or `None` if it does not exist.
    pub fn get(&self, key: &str) -> Option<String> {
        self.inner
            .read()
            .expect("store lock poisoned")
            .get(key)
            .cloned()
    }

    /// Remove `key` from the store. Returns `true` if the key existed.
    pub fn del(&self, key: &str) -> bool {
        self.inner
            .write()
            .expect("store lock poisoned")
            .remove(key)
            .is_some()
    }

    /// Return `true` if `key` is present in the store.
    pub fn exists(&self, key: &str) -> bool {
        self.inner
            .read()
            .expect("store lock poisoned")
            .contains_key(key)
    }

    /// Return the number of keys currently stored.
    pub fn len(&self) -> usize {
        self.inner
            .read()
            .expect("store lock poisoned")
            .len()
    }

    /// Return `true` if the store contains no keys.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return a snapshot of all key-value pairs as a sorted `Vec`.
    pub fn snapshot(&self) -> Vec<(String, String)> {
        let guard = self.inner.read().expect("store lock poisoned");
        let mut pairs: Vec<(String, String)> = guard
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        pairs
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get() {
        let store = Store::new();
        assert_eq!(store.get("k"), None);
        store.set("k".into(), "v".into());
        assert_eq!(store.get("k"), Some("v".into()));
    }

    #[test]
    fn overwrite_returns_old_value() {
        let store = Store::new();
        store.set("k".into(), "first".into());
        let old = store.set("k".into(), "second".into());
        assert_eq!(old, Some("first".into()));
        assert_eq!(store.get("k"), Some("second".into()));
    }

    #[test]
    fn del_existing_key() {
        let store = Store::new();
        store.set("k".into(), "v".into());
        assert!(store.del("k"));
        assert_eq!(store.get("k"), None);
    }

    #[test]
    fn del_missing_key() {
        let store = Store::new();
        assert!(!store.del("missing"));
    }

    #[test]
    fn exists() {
        let store = Store::new();
        assert!(!store.exists("k"));
        store.set("k".into(), "v".into());
        assert!(store.exists("k"));
    }

    #[test]
    fn len_and_is_empty() {
        let store = Store::new();
        assert!(store.is_empty());
        store.set("a".into(), "1".into());
        store.set("b".into(), "2".into());
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn snapshot_is_sorted() {
        let store = Store::new();
        store.set("c".into(), "3".into());
        store.set("a".into(), "1".into());
        store.set("b".into(), "2".into());
        let snap = store.snapshot();
        assert_eq!(
            snap,
            vec![
                ("a".into(), "1".into()),
                ("b".into(), "2".into()),
                ("c".into(), "3".into()),
            ]
        );
    }

    #[test]
    fn concurrent_reads_and_writes() {
        use std::thread;
        let store = Store::new();
        let mut handles = vec![];
        for i in 0..10 {
            let s = store.clone();
            handles.push(thread::spawn(move || {
                s.set(format!("key{}", i), format!("val{}", i));
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(store.len(), 10);
    }
}
