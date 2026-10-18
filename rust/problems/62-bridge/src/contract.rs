//! Provided: the implementor trait.

/// HOW a message is delivered.
pub trait Channel {
    fn deliver(&self, recipient: &str, title: &str, body: &str);
}
