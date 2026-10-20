//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Time comes from the clock handed to `create`. A lease granted at time t for d milliseconds is valid while
//! `now < t + d`; at `now == t + d` it has expired. Nothing blocks: a caller that does not get the lock gets `None`
//! and decides what to do.

#[derive(Debug, PartialEq, Eq)]
pub enum LockError {
    /// lease of 0 milliseconds
    InvalidLease,
}

pub trait LockService: Send + Sync {
    /// Tries to take the named lock for `lease_ms` (> 0).
    /// * Free, or the previous lease has expired: granted with a NEW fencing token = (highest token ever issued for
    ///   this lock) + 1. Tokens only ever grow, also across different owners and expired leases.
    /// * Held (unexpired) by the same owner: the lease is extended to now + lease_ms and the SAME token is returned.
    /// * Held (unexpired) by somebody else: `Ok(None)`.
    fn acquire(&self, lock: &str, owner: &str, lease_ms: u64) -> Result<Option<u64>, LockError>;

    /// Extends the lease to now + lease_ms. True only if this owner and token still hold an unexpired lease.
    fn renew(&self, lock: &str, owner: &str, token: u64, lease_ms: u64) -> bool;

    /// Gives the lock up. True only if this owner and token still hold an unexpired lease; otherwise nothing changes.
    fn release(&self, lock: &str, owner: &str, token: u64) -> bool;

    /// The owner of the unexpired lease, if any.
    fn holder(&self, lock: &str) -> Option<String>;

    /// The highest token ever issued for the lock; 0 if it was never taken.
    fn latest_token(&self, lock: &str) -> u64;
}
