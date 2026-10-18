//! Thin test entry point. Your domain model lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

pub trait TaskSchedulerApi: Send + Sync {
    /// Runs the task once, when its due time (now + delay) has been reached by a `run_due` call.
    fn schedule_once(&self, delay_millis: u64, task: Box<dyn FnOnce() + Send>) -> String;

    /// `None` if `period_millis == 0`. Fixed-rate: occurrence k is due at now + initial_delay + k * period.
    fn schedule_recurring(
        &self,
        initial_delay_millis: u64,
        period_millis: u64,
        task: Box<dyn FnMut() + Send>,
    ) -> Option<String>;

    /// True if it stopped a task that was still going to run.
    fn cancel(&self, id: &str) -> bool;

    /// Runs everything due on the calling thread; returns the number of executions performed.
    fn run_due(&self) -> usize;
}
