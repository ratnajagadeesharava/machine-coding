//! Thin test entry point. Your design lives behind it. Single-threaded (`&mut self`).

#[allow(clippy::len_without_is_empty)]
pub trait Cache<K, V> {
    /// Present: returns a clone of the value and counts as a use. Absent: `None`, nothing changes.
    fn get(&mut self, key: &K) -> Option<V>;

    /// Insert (count 1) or replace (counts as a use). Evicts the least frequently, then least recently, used entry on overflow.
    fn put(&mut self, key: K, value: V);

    /// `true` if an entry was removed; its use count is forgotten.
    fn remove(&mut self, key: &K) -> bool;

    fn len(&self) -> usize;
}
