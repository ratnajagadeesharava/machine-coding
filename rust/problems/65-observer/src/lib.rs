//! Observer — a price feed that notifies subscribed observers. All methods take `&self`.
#![allow(unused_variables)]

pub mod contract;

use contract::*;
use std::rc::Rc;

pub struct PriceFeed;

impl PriceFeed {
    pub fn new() -> PriceFeed {
        todo!()
    }

    /// Listens to every symbol. 
    pub fn subscribe(&self, observer: Rc<dyn PriceObserver>) -> SubscriptionId {
        todo!()
    }

    /// Listens to one symbol (exact match). Blank symbol -> `Err(FeedError::InvalidSymbol)`.
    pub fn subscribe_to(&self, symbol: &str, observer: Rc<dyn PriceObserver>) -> Result<SubscriptionId, FeedError> {
        todo!()
    }

    /// Stops notifications for that subscription. Returns true if it was active (a second call returns false).
    pub fn unsubscribe(&self, id: SubscriptionId) -> bool {
        todo!()
    }

    /// Number of active subscriptions.
    pub fn subscriber_count(&self) -> usize {
        todo!()
    }

    /// Notifies the interested observers of a new price; returns how many observers failed.
    pub fn publish(&self, symbol: &str, price_cents: u64) -> Result<usize, FeedError> {
        todo!()
    }
}
