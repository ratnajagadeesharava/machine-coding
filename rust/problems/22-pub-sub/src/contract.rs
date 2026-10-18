//! Thin test entry point. Your domain model lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

pub trait PubSubApi: Send + Sync {
    fn create_topic(&self, topic: &str) -> bool;

    fn subscribe(&self, topic: &str, subscriber: &str) -> bool;

    fn unsubscribe(&self, topic: &str, subscriber: &str) -> bool;

    /// Number of subscribers the message was delivered to; -1 for an unknown topic.
    fn publish(&self, topic: &str, message: &str) -> i32;

    /// Up to `max` pending messages, oldest first, consumed by the call.
    fn poll(&self, subscriber: &str, topic: &str, max: i32) -> Vec<String>;
}
