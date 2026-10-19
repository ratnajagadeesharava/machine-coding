//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

use std::collections::HashSet;
use std::sync::Arc;

/// A broken page.
#[derive(Debug, PartialEq, Eq)]
pub struct FetchError;

/// Provided by the tests (the "internet"). Thread-safe, may be slow, may contain duplicate and self links.
pub trait Fetcher: Send + Sync {
    /// The outgoing links of a page, or `Err(FetchError)` for a broken page.
    fn links(&self, url: &str) -> Result<Vec<String>, FetchError>;
}

/// `workers` was 0.
#[derive(Debug, PartialEq, Eq)]
pub struct InvalidWorkers;

pub trait Crawler: Send + Sync {
    /// Crawls every page reachable from `start` and returns the set of all URLs reached (including `start` and
    /// broken pages). Rules:
    /// * every URL is fetched exactly once, however many pages link to it;
    /// * at most `workers` fetches run at the same time;
    /// * a page whose fetch fails counts as reached but contributes no links; the crawl goes on;
    /// * returns only when no work is left, and leaves no thread running behind it.
    ///
    /// `Err(InvalidWorkers)` if `workers == 0`.
    fn crawl(
        &self,
        start: &str,
        workers: usize,
        fetcher: Arc<dyn Fetcher>,
    ) -> Result<HashSet<String>, InvalidWorkers>;
}
