//! Thin test entry point. Your domain model lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

use std::sync::Arc;

pub trait EventBusApi: Send + Sync {
    /// Registers the handler for the topic and returns the subscription id.
    fn subscribe(&self, topic: &str, handler: Arc<dyn Fn(&str) + Send + Sync>) -> String;

    /// `true` the first time for a known id; `false` for unknown or already removed ids.
    fn unsubscribe(&self, subscription_id: &str) -> bool;

    /// Hands the event to every current subscriber of the topic; never waits for handlers.
    fn publish(&self, topic: &str, event: &str);

    /// `true` once everything published so far (including by handlers) has been handled or discarded.
    fn await_idle(&self, timeout_millis: u64) -> bool;

    /// Stops accepting new events; events already published are still delivered. Idempotent.
    fn close(&self);
}
