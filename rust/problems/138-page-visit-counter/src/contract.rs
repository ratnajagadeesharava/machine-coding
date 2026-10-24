//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

#[derive(Debug, PartialEq, Eq)]
pub enum CounterError {
    /// blank url, url with an empty host after normalisation, blank visitor id, or `k == 0`
    InvalidArgument,
}

/// Every `url` argument is normalised first (see the README). Visitor ids are compared exactly as given
/// (case-sensitive, not trimmed).
pub trait VisitCounter: Send + Sync {
    /// Records one visit of `visitor_id` to the page.
    fn visit(&self, url: &str, visitor_id: &str) -> Result<(), CounterError>;

    /// Number of visits to the page, 0 if it was never visited (or was reset).
    fn total_visits(&self, url: &str) -> Result<u64, CounterError>;

    /// Number of distinct visitor ids that visited the page. Exact. 0 for an unknown page.
    fn unique_visitors(&self, url: &str) -> Result<u64, CounterError>;

    /// Number of visits to the page by this visitor, 0 if none.
    fn visits_by(&self, url: &str, visitor_id: &str) -> Result<u64, CounterError>;

    /// The `k` most visited pages as `"url:total"` (normalised url), ordered by total descending then url
    /// ascending (plain string order). Fewer entries if there are fewer pages. `k == 0` is an error.
    fn top_pages(&self, k: usize) -> Result<Vec<String>, CounterError>;

    /// The `k` most frequent visitors of the page as `"visitorId:count"`, ordered by count descending then
    /// visitor id ascending. Empty for an unknown page. `k == 0` is an error.
    fn top_visitors(&self, url: &str, k: usize) -> Result<Vec<String>, CounterError>;

    /// Total number of visits over all pages.
    fn grand_total(&self) -> u64;

    /// Number of pages that currently have at least one recorded visit.
    fn page_count(&self) -> usize;

    /// Forgets the page and all its counts. Returns whether the page existed.
    fn reset(&self, url: &str) -> Result<bool, CounterError>;

    /// Forgets every page.
    fn reset_all(&self);

    /// A point-in-time view of all pages as `"url:total:unique"` lines sorted by url ascending. It is an owned copy:
    /// later visits do not change it, and it equals the state after some prefix of the serial history of all
    /// visits (never a page counted "ahead of" another page that was updated earlier).
    fn snapshot(&self) -> Vec<String>;

    /// Like `snapshot`, but per visitor: `"url:visitorId:count"` lines sorted by url then visitor id (plain string
    /// order). Taken at a single point in time as well.
    fn snapshot_detail(&self) -> Vec<String>;
}
