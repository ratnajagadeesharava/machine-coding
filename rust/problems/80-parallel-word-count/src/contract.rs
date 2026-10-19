//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

use std::collections::HashMap;

/// `parallelism` was 0.
#[derive(Debug, PartialEq, Eq)]
pub struct InvalidParallelism;

pub trait WordCounter: Send + Sync {
    /// Counts word occurrences over all documents using at most `parallelism` worker threads.
    ///
    /// A word is a maximal run of ASCII letters (a-z, A-Z); every other character (digits, punctuation, spaces,
    /// newlines, non-ASCII) separates words. Words are compared case-insensitively and reported in lower case.
    /// The result has no zero entries. `Err(InvalidParallelism)` if `parallelism == 0`.
    fn count(
        &self,
        documents: &[String],
        parallelism: usize,
    ) -> Result<HashMap<String, u64>, InvalidParallelism>;
}
