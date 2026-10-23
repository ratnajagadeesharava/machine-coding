//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

#[derive(Debug, PartialEq, Eq)]
pub enum AutofillError {
    /// the seed list is empty
    InvalidArgument,
    /// the seeds are not one recognised series (mixed kinds, not arithmetic, malformed, mixed case, blank)
    UnrecognizedSeries,
}

pub trait AutofillService: Send + Sync {
    /// Recognises the series formed by `seeds` (cell texts, top to bottom) and returns the next `count` values of
    /// that series, in order. The seeds themselves are not part of the result.
    ///
    /// Numeric series: seeds are plain decimal numbers (optional '-', digits, optional '.digits', at most 15 digits).
    /// Letter series: seeds are 1 to 10 ASCII letters, all of one case, ordered like spreadsheet column names
    /// (A..Z, AA..AZ, BA..). Both are arithmetic: the step is the difference between the first two seeds, every
    /// consecutive pair must have that step, and a single seed has step 1.
    ///
    /// A letter series that decreases stops before "A"/"a", so the result can be shorter than `count`.
    /// `count == 0` returns an empty vector. Seeds are validated even when `count` is 0.
    ///
    /// Errors: `InvalidArgument` for an empty seed list; `UnrecognizedSeries` for any seeds that are not one
    /// recognised series.
    fn autofill(&self, seeds: &[&str], count: usize) -> Result<Vec<String>, AutofillError>;
}
