//! Thin test entry point. Your domain model lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

pub trait MetricsApi: Send + Sync {
    /// False (nothing recorded) if `delta <= 0` or the name is blank.
    fn increment(&self, name: &str, delta: i64, tags: &[(&str, &str)]) -> bool;

    /// Sum over exactly this tag set; 0 if unknown.
    fn counter(&self, name: &str, tags: &[(&str, &str)]) -> i64;

    /// Sum over all tag sets of the counter name.
    fn counter_total(&self, name: &str) -> i64;

    /// False if the name is blank.
    fn observe(&self, name: &str, value: i64) -> bool;

    /// How many values were observed for the histogram.
    fn observation_count(&self, name: &str) -> u64;

    /// Nearest-rank percentile, `p` in 1..=100; `None` if unknown/empty or `p` out of range.
    fn percentile(&self, name: &str, p: u32) -> Option<i64>;

    /// Sum of deltas (any tags) recorded in `(now - window_millis, now]`.
    fn count_in_window(&self, name: &str, window_millis: u64) -> i64;
}
