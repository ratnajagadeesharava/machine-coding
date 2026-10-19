//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! A pool of `size` reusable "connections", identified by the numbers `0..size`.

/// The pool is (or became) closed.
#[derive(Debug, PartialEq, Eq)]
pub struct Closed;

#[derive(Debug, PartialEq, Eq)]
pub enum ReleaseError {
    /// The id is outside `0..size`.
    UnknownConnection,
    /// That connection is not currently acquired (double release).
    NotAcquired,
}

pub trait Pool: Send + Sync {
    /// Waits up to `timeout_ms` for a free connection and returns its id (`Ok(Some(id))`), or `Ok(None)` if none
    /// became free in time. `0` means do not wait. `Err(Closed)` if the pool is (or becomes) closed.
    fn acquire(&self, timeout_ms: u64) -> Result<Option<usize>, Closed>;

    /// Gives a connection back. Allowed after `close`.
    fn release(&self, id: usize) -> Result<(), ReleaseError>;

    /// Connections that could be acquired right now.
    fn available(&self) -> usize;

    /// Connections currently acquired. `available() + in_use() == size` at all times.
    fn in_use(&self) -> usize;

    /// Rejects further acquires and wakes every waiting thread (they get `Err(Closed)`). Idempotent.
    fn close(&self);
}
