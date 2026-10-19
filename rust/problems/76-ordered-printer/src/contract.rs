//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

/// The worker index was outside `0..workers`.
#[derive(Debug, PartialEq, Eq)]
pub struct UnknownWorker;

pub trait OrderedPrinter: Send + Sync {
    /// Blocks until it is `worker`'s turn, runs `action` on the calling thread, then hands the turn to
    /// worker `(worker + 1) % workers`. The turn is handed over even if `action` panics (the panic
    /// propagates to the caller). `Err(UnknownWorker)` if `worker` is outside `0..workers`.
    fn take_turn(&self, worker: usize, action: &mut dyn FnMut()) -> Result<(), UnknownWorker>;
}
