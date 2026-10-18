//! Provided: the observer trait and the error type.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeedError {
    /// Blank symbol.
    InvalidSymbol,
    /// Price of 0.
    InvalidPrice,
}

pub type SubscriptionId = u64;

/// Observers are notified with `&self` (use interior mutability for state). Returning `Err` means the observer
/// failed; the feed must keep going.
pub trait PriceObserver {
    fn on_price(&self, symbol: &str, price_cents: u64) -> Result<(), String>;
}
