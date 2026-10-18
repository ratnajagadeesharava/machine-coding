//! Thin test entry point. Your design lives behind it. Single-threaded (`&mut self`).

#[allow(clippy::len_without_is_empty)]
pub trait Cache<K, V> {
    /// Present: returns a clone of the value and counts as a use. Absent: `None`, nothing changes.
    fn get(&mut self, key: &K) -> Option<V>;

    /// Insert or replace; the key becomes the most recently used. Evicts the least recently used entry on overflow.
    fn put(&mut self, key: K, value: V);

    /// `true` if an entry was removed.
    fn remove(&mut self, key: &K) -> bool;

    fn len(&self) -> usize;
}
