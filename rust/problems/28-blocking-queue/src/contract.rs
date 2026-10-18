//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

/// Returned by `put`/`offer` when the buffer is closed. The item was not stored.
#[derive(Debug, PartialEq, Eq)]
pub struct Closed;

pub trait BlockingBuffer<T>: Send + Sync {
    /// Waits while full. `Err(Closed)` if the buffer is (or becomes) closed; the item is then not stored.
    fn put(&self, item: T) -> Result<(), Closed>;

    /// Waits while empty and open. `None` only when the buffer is closed and drained.
    fn take(&self) -> Option<T>;

    /// Like `put` but gives up after `timeout_ms` (`Ok(false)`). `0` means do not wait. `Err(Closed)` if closed.
    fn offer(&self, item: T, timeout_ms: u64) -> Result<bool, Closed>;

    /// Like `take` but gives up after `timeout_ms` (`None`). `0` means do not wait.
    fn poll(&self, timeout_ms: u64) -> Option<T>;

    /// Number of stored items.
    fn size(&self) -> usize;

    /// Rejects further puts, wakes all waiters, keeps remaining items for consumers. Idempotent, never blocks.
    fn close(&self);
}
