//! Thin test entry point. Your design lives behind it.
//! Every operation is atomic on its own (`&self`, `Send + Sync`); see the README.

#[derive(Debug, PartialEq, Eq)]
pub enum MapError {
    /// initial capacity outside 1..=1_048_576, or load factor outside 10..=100
    InvalidArgument,
}

/// Provided hash function, implemented by the tests. Any u64 is a legal result (0, u64::MAX, ...), and different
/// keys may return the same value. The map must use this and nothing else to decide where a key lives.
pub trait KeyHasher: Send + Sync {
    fn hash(&self, key: &str) -> u64;
}

/// The hasher is called exactly once per call to `put`, `put_if_absent`, `get`, `get_or_default`, `remove` and
/// `contains_key`, and never by any other method.
pub trait StringMap: Send + Sync {
    /// Stores the value and returns the previous value for the key, or `None` if the key was new.
    fn put(&self, key: &str, value: &str) -> Option<String>;

    /// Stores the value only if the key is absent. Returns the existing value (nothing changes) or `None` (stored).
    fn put_if_absent(&self, key: &str, value: &str) -> Option<String>;

    fn get(&self, key: &str) -> Option<String>;

    /// The value for the key, or `default` if absent. Never stores anything.
    fn get_or_default(&self, key: &str, default: &str) -> String;

    /// Removes the key and returns its value, or `None` if it was absent.
    fn remove(&self, key: &str) -> Option<String>;

    fn contains_key(&self, key: &str) -> bool;

    fn size(&self) -> usize;

    fn is_empty(&self) -> bool;

    /// Removes every entry. The capacity does not change.
    fn clear(&self);

    /// Number of buckets. See the README for the exact growth rule.
    fn capacity(&self) -> usize;

    /// A new vector of all keys in insertion order (a snapshot: later changes to the map do not affect it).
    fn keys(&self) -> Vec<String>;

    /// A new vector of `key=value` strings for every entry, in the same order as `keys`.
    fn entries(&self) -> Vec<String>;
}
