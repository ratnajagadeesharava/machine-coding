//! Thin test entry point for the saga orchestrator. Steps and the log are provided by the tests; `sleeper(ms)` is how
//! you "wait" between retries (the tests record the values instead of sleeping). The traits take `&self`: use
//! interior mutability (`Mutex`) for your state.
//!
//! Rules (the tests check all of them):
//! - Steps run in order. A step is attempted at most 3 times; before attempt 2 and 3 you call the sleeper with 100
//!   and 200. A step that fails 3 times fails the saga. `false` is the failure signal.
//! - Then the steps that completed are compensated in REVERSE order (the failed step is not compensated). A
//!   compensation is attempted at most 5 times, sleeping 100, 200, 400, 800 between attempts. If one still fails the
//!   saga ends as `CompensationFailed` and the remaining compensations are NOT run.
//! - Progress is written to the `SagaLog` so a new orchestrator on the same log continues where the old one stopped:
//!   a finished step is not executed again, a finished compensation is not run again.
//! - `run` on a finished saga returns its final status without calling any step. `run` on a saga that is running
//!   right now (another thread) returns `Running` immediately.
//! - Different sagas run concurrently; the orchestrator must be thread-safe.

use std::sync::Arc;

/// One local transaction of a saga plus its compensating action. Both must be idempotent per saga id
/// (they can be invoked again after a crash).
pub trait SagaStep: Send + Sync {
    fn name(&self) -> String;

    fn execute(&self, saga_id: &str) -> bool;

    fn compensate(&self, saga_id: &str) -> bool;
}

/// Stable storage of the orchestrator (provided by the tests; it survives a restart: a restarted orchestrator is a
/// NEW instance created with the SAME log). Appending is atomic and durable as soon as the call returns.
pub trait SagaLog: Send + Sync {
    fn append(&self, record: &str);

    /// Everything appended so far, oldest first.
    fn records(&self) -> Vec<String>;
}

pub type Log = Arc<dyn SagaLog>;
pub type Sleeper = Arc<dyn Fn(u64) + Send + Sync>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SagaStatus {
    /// never seen by this log
    NotFound,
    /// started, not finished (possibly being run right now)
    Running,
    /// every step executed
    Completed,
    /// a step failed for good and every earlier step was compensated
    Compensated,
    /// a compensation failed for good: needs a human
    CompensationFailed,
}

pub trait SagaOrchestrator: Send + Sync {
    /// Runs (or continues) the saga to its end on the calling thread and returns the final status.
    fn run(&self, saga_id: &str, steps: &[Arc<dyn SagaStep>]) -> SagaStatus;

    /// The status derived from the log (also valid right after a restart).
    fn status(&self, saga_id: &str) -> SagaStatus;
}
