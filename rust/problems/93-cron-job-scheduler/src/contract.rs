//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Time comes from the clock handed to `create`. A "minute" is epoch milliseconds / 60,000 (UTC); epoch minute 0 is
//! Thursday 1970-01-01 00:00. Nothing runs by itself: the caller drives the scheduler by calling `run_due`.

use std::sync::Arc;

/// The work a job does (provided by the tests). A job fails by returning `Err`.
pub type JobFn = Arc<dyn Fn() -> Result<(), String> + Send + Sync>;

#[derive(Debug, PartialEq, Eq)]
pub enum JobError {
    /// duplicate name, bad cron expression, max_attempts < 1, backoff of 0
    InvalidDefinition,
    UnknownJob,
}

pub type Jobs<T> = Result<T, JobError>;

pub trait JobService: Send + Sync {
    /// Defines a job. `cron` has THREE fields "minute hour weekday" (weekday 0-6, 0 = Sunday); each field is `*`,
    /// `*/n`, a number, a range `a-b` or a comma list of those. The first run is the first matching minute strictly
    /// after the minute of the definition. A failing run is retried up to `max_attempts` attempts in total; after
    /// failure number k (k = 1, 2, ...) the next attempt is due `min(initial_backoff_ms * 2^(k-1), max_backoff_ms)`
    /// later.
    fn define(&self, name: &str, cron: &str, job: JobFn, max_attempts: u32, initial_backoff_ms: u64, max_backoff_ms: u64) -> Jobs<()>;

    /// Runs, on the calling thread, everything that is due now and returns the number of executions (attempts) made.
    /// Order: by due time, then by definition order. Per job at most ONE execution per call, and
    /// * a retry that is due runs (attempt k+1 of the same slot);
    /// * otherwise if the next cron slot is due it runs, for the LATEST matching minute not after now (missed earlier
    ///   minutes are not run and not recorded);
    /// * if a cron slot is due while the job is still waiting for a retry, the slot is SKIPPED (recorded, not run);
    ///   no two slots overlap.
    ///
    /// After a slot ran or was skipped, the next slot is the first matching minute after the current minute.
    /// Failures never escape and never stop other jobs. Concurrent callers must never execute the same attempt twice.
    fn run_due(&self) -> usize;

    /// Paused jobs neither run nor retry.
    fn pause(&self, name: &str) -> Jobs<()>;

    /// The next slot is recomputed from now (first matching minute strictly after the current minute).
    fn resume(&self, name: &str) -> Jobs<()>;

    fn remove(&self, name: &str) -> Jobs<()>;

    /// Epoch minute of the next cron slot; `None` if the job is paused.
    fn next_slot(&self, name: &str) -> Jobs<Option<u64>>;

    /// Everything that happened for the job, oldest first: "slot:attempt:OUTCOME" with OUTCOME one of OK, FAILED
    /// (will be retried), DEAD (last attempt failed), SKIPPED (attempt is 0).
    fn history(&self, name: &str) -> Jobs<Vec<String>>;

    /// Jobs that ran out of attempts, as "name:slot", in the order they died.
    fn dead_letters(&self) -> Vec<String>;
}
