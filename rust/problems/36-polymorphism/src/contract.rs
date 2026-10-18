//! Provided: the `Vehicle` trait and the error type.

pub trait Vehicle {
    fn plate(&self) -> &str;
    /// Toll in cents. `peak` is true during rush hour.
    fn toll(&self, peak: bool) -> u64;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TollError {
    /// A truck needs at least 2 axles.
    InvalidAxles,
}
