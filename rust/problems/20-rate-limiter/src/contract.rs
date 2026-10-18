//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

pub trait RateLimiterApi: Send + Sync {
    /// Decides whether a request with this key may proceed right now (time comes from the clock given at
    /// creation). Allowed requests are recorded; denied requests are not.
    fn allow(&self, key: &str) -> bool;
}
