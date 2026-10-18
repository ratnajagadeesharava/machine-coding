//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

use std::sync::mpsc::Receiver;

/// Returned by `submit` when the pool cannot take the job (queue full while all workers are busy, or shut down).
#[derive(Debug, PartialEq, Eq)]
pub struct Rejected;

/// A job: yields a number. May panic.
pub type Job = Box<dyn FnOnce() -> i64 + Send>;

/// The single outcome of a job: its value, or the panic message.
pub type JobOutcome = Result<i64, String>;

pub trait WorkerPoolApi: Send + Sync {
    /// Never blocks. `Err(Rejected)` when full or shut down. The receiver yields exactly one outcome.
    fn submit(&self, job: Job) -> Result<Receiver<JobOutcome>, Rejected>;

    /// Stops accepting jobs; accepted jobs still complete. Returns immediately; idempotent.
    fn shutdown(&self);

    /// True iff `shutdown` was called and all accepted jobs finished within the timeout.
    fn await_termination(&self, timeout_ms: u64) -> bool;
}
