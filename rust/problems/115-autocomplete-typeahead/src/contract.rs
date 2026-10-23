//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Terms are stored trimmed and lower-cased; internal whitespace is kept. Every method that takes a term, prefix or
//! user id returns `InvalidArgument` for a blank term or user id (a blank prefix is valid and matches everything).
//! Time comes from the clock handed to `create` (epoch milliseconds).

#[derive(Debug, PartialEq, Eq)]
pub enum TypeaheadError {
    /// blank term / blank user id / zero count / `k == 0`
    InvalidArgument,
}

/// Rule used to order suggestions that start with the same prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ranking {
    /// Higher recorded count first.
    Frequency,
    /// Most recently recorded first (time comes from the clock handed to `create`).
    Recency,
}

pub trait Typeahead: Send + Sync {
    /// Adds 1 to the count of `term` and marks it as recorded now. Same as `record_count(term, 1)`.
    fn record(&self, term: &str) -> Result<(), TypeaheadError>;

    /// Adds `count` (must be positive) to the count of `term` and marks it as recorded now. The new count is visible
    /// to the very next call, from any thread. A rejected call changes nothing.
    fn record_count(&self, term: &str, count: u64) -> Result<(), TypeaheadError>;

    /// Total count of `term` (0 if never recorded). Term is normalised like in `record`.
    fn count(&self, term: &str) -> Result<u64, TypeaheadError>;

    /// Switches the active ranking rule; counts and recency are kept.
    fn set_ranking(&self, ranking: Ranking);

    /// Up to `k` terms that start with `prefix` (a term equal to the prefix qualifies), best first under the active
    /// ranking: Frequency = count descending; Recency = most recently recorded first (terms recorded at the same
    /// clock instant are tied). Ties are always broken alphabetically (ascending, plain character order).
    /// The prefix is trimmed and lower-cased; an empty or blank prefix matches every term (top-k overall).
    /// Unmatched prefix -> empty list. `k == 0` -> `InvalidArgument`.
    fn suggest(&self, prefix: &str, k: usize) -> Result<Vec<String>, TypeaheadError>;

    /// Like `record` and additionally adds 1 to `user_id`'s own count of the term
    /// (the global count and recency are updated too).
    fn record_for(&self, user_id: &str, term: &str) -> Result<(), TypeaheadError>;

    /// Personalised suggestions: same candidates as `suggest`, but ordered by `user_id`'s own count of the term
    /// (descending; terms the user never recorded count 0), then by the active ranking, then alphabetically.
    /// A user with no history gets exactly the `suggest` result.
    fn suggest_for(
        &self,
        user_id: &str,
        prefix: &str,
        k: usize,
    ) -> Result<Vec<String>, TypeaheadError>;
}
