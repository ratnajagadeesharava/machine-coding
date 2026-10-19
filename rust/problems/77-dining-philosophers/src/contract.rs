//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Philosophers sit in a circle, numbered `0..seats`; there is one fork between every two neighbours, so
//! philosopher `p` shares a fork with `(p + 1) % seats` and with `(p + seats - 1) % seats`.

/// The philosopher index was outside `0..seats`.
#[derive(Debug, PartialEq, Eq)]
pub struct UnknownPhilosopher;

pub trait Table: Send + Sync {
    /// Blocks until philosopher `p` holds both of its forks, runs `eat` on the calling thread, then puts the
    /// forks down (also when `eat` panics; the panic propagates). `Err(UnknownPhilosopher)` if `p >= seats`.
    fn dine(&self, philosopher: usize, eat: &mut dyn FnMut()) -> Result<(), UnknownPhilosopher>;
}
