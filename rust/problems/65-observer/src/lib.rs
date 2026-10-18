//! Observer — a subject that notifies loosely-coupled observers. All methods take `&self`, so observers can
//! subscribe and unsubscribe while a notification is in progress (use `RefCell` inside).
#![allow(unused_variables)]

pub mod contract;

use contract::*;
use std::rc::Rc;

pub struct PriceFeed;

impl PriceFeed {
    pub fn new() -> PriceFeed {
        todo!()
    }

    /// Listens to every symbol. Ids are unique.
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

    /// Tells the interested observers about a new price. Blank symbol -> `Err(InvalidSymbol)`, price 0 ->
    /// `Err(InvalidPrice)` (nobody is notified). Rules:
    /// - observers are notified in the order they subscribed;
    /// - the set of observers is fixed when `publish` starts: one that subscribes during the notification does NOT
    ///   get this price;
    /// - an observer unsubscribed before its turn (by anyone, even during this publish) is skipped;
    /// - an observer that returns `Err` does not stop the others.
    ///
    /// Returns how many observers failed.
    pub fn publish(&self, symbol: &str, price_cents: u64) -> Result<usize, FeedError> {
        todo!()
    }
}
