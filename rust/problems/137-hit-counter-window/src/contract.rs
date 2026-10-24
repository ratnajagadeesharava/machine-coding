//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

#[derive(Debug, PartialEq, Eq)]
pub enum CounterError {
    /// blank key, hit count 0, duration outside `1..=window`, window 0, `k` of 0
    InvalidArgument,
}

/// Time comes from the `Clock` given to `create` (`now_millis()`).
///
/// Every operation, reads included, takes ONE clock reading and uses `now = max(reading, the largest now this
/// counter has used before)`; a clock that moves backwards is therefore never seen going back. A hit is recorded
/// at that `now`. A hit recorded at time `t` is inside a window of length `w` exactly when `now - w < t <= now`
/// (a hit that is exactly `w` ms old has expired).
///
/// A blank key is `InvalidArgument` everywhere. An unknown (but valid) key counts as 0.
pub trait HitCounter: Send + Sync {
    /// Records one hit for the key at the current time.
    fn hit(&self, key: &str) -> Result<(), CounterError>;

    /// Records `count` hits at the current time. `count == 0` is `InvalidArgument`.
    fn hit_n(&self, key: &str, count: u64) -> Result<(), CounterError>;

    /// Hits of the key inside the configured window.
    fn count(&self, key: &str) -> Result<u64, CounterError>;

    /// Hits of the key in `(now - duration_ms, now]`. `InvalidArgument` unless `0 < duration_ms <= current window`;
    /// the duration is validated even for unknown keys.
    fn count_in_last(&self, key: &str, duration_ms: u64) -> Result<u64, CounterError>;

    /// Hits across all keys inside the configured window.
    fn total_in_window(&self) -> u64;

    /// Changes the window (`new_window_ms == 0` is `InvalidArgument`). First every hit that is already outside the
    /// OLD window (at the current time) is gone for good; then the new window applies. Shrinking therefore drops the
    /// hits that are not inside the new window permanently, and growing never brings back a hit that was already
    /// outside the old window.
    fn set_window(&self, new_window_ms: u64) -> Result<(), CounterError>;

    /// The configured window in milliseconds.
    fn window(&self) -> u64;

    /// Keys with at least one hit inside the window, ordered by windowed count descending then key ascending
    /// (byte-wise `str` order), at most `k` of them. `k == 0` is `InvalidArgument`.
    fn top_keys(&self, k: usize) -> Result<Vec<String>, CounterError>;

    /// Approximate memory footprint: the number of timestamp/bucket records this counter currently keeps in memory,
    /// summed over all keys (a fixed-size array of slots kept for a key counts every slot of it; a key with nothing
    /// stored counts 0). Tests only check upper bounds, never an exact value. May drop expired records first.
    fn memory_probe(&self) -> usize;
}
