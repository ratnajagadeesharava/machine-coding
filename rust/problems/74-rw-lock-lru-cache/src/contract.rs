//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

pub trait SharedCache: Send + Sync {
    /// Returns the value and marks the key as the most recently used.
    fn get(&self, key: u64) -> Option<String>;

    /// Returns the value WITHOUT changing the usage order (a pure read).
    fn peek(&self, key: u64) -> Option<String>;

    /// Inserts or replaces; the key becomes the most recently used. Evicts the least recently used key if over capacity.
    fn put(&self, key: u64, value: String);

    /// Returns true if the key was present.
    fn remove(&self, key: u64) -> bool;

    /// Number of entries; never above the capacity.
    fn len(&self) -> usize;
}
