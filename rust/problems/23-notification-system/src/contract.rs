//! Thin test entry point. Your domain model lives behind it.

/// Provided by the caller (external system). Do not modify.
pub trait DeliveryGateway: Send + Sync {
    /// One delivery attempt. `true` when accepted, `false` when refused.
    fn deliver(&self, channel: &str, address: &str, body: &str) -> bool;
}

pub trait NotificationApi {
    fn register_user(&mut self, user_id: &str, addresses: &[(&str, &str)]);

    /// Ignored for unregistered users. Everything is enabled by default.
    fn set_preference(&mut self, user_id: &str, channel: &str, category: &str, enabled: bool);

    fn define_template(&mut self, template_id: &str, body: &str);

    /// Returns `UNKNOWN_USER`, `INVALID`, `SUPPRESSED`, `FAILED` or `DELIVERED:<channel>`.
    /// Channels are tried in the given order.
    fn send(
        &mut self,
        user_id: &str,
        category: &str,
        template_id: &str,
        params: &[(&str, &str)],
        channels: &[&str],
    ) -> String;
}
