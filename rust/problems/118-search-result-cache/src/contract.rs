//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

/// A failure reported by the search backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchError(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheError {
    /// null-equivalent / blank query
    InvalidQuery,
    /// the backend call failed; every caller that waited for that call gets an equal error
    Backend(SearchError),
}

/// The search service behind the cache. PROVIDED: the tests implement it (a counting fake). Not part of your design.
pub trait SearchBackend: Send + Sync {
    /// Runs the query and returns the matching document ids, best first. The argument is already normalised.
    /// The call may be slow and may fail.
    fn search(&self, normalised_query: &str) -> Result<Vec<String>, SearchError>;

    /// The current version of the index. It changes (it only ever goes up) whenever documents are added, changed
    /// or removed. Cheap to call.
    fn index_version(&self) -> u64;
}

pub trait SearchCache: Send + Sync {
    /// Returns the results for the query, from the cache when possible, otherwise from the backend.
    ///
    /// Normalisation: trim, lower-case (Unicode) and collapse every run of whitespace into one space. Word order
    /// is significant ("red shoes" and "shoes red" are different queries). The normalised text is both the cache
    /// key and the argument given to the backend.
    ///
    /// A stored entry is served only if it is younger than the TTL (it expires exactly `ttl` ms after it was
    /// stored; hits do not extend it) and the backend's `index_version()` still equals the version that was read
    /// just before the backend call that produced it. Otherwise the entry is dropped and the backend is called.
    /// If other threads are already waiting for a backend call for the same normalised query, this call joins it
    /// instead of making another one. A failed call is returned (an equal `CacheError::Backend`) to every thread
    /// that waited for it and is not cached.
    ///
    /// A blank query is `Err(CacheError::InvalidQuery)` and nothing is counted.
    fn get(&self, raw_query: &str) -> Result<Vec<String>, CacheError>;

    /// Drops the entry for this query (normalised like `get`). `InvalidQuery` for a blank query.
    fn invalidate(&self, raw_query: &str) -> Result<(), CacheError>;

    /// Drops every entry.
    fn invalidate_all(&self);

    /// Number of gets served from a stored entry.
    fn hits(&self) -> u64;

    /// Number of gets that were not served from a stored entry (including the ones that joined a call already in
    /// flight, and the ones whose backend call failed). A get is counted as soon as it has decided it cannot be
    /// served from the cache, before it waits for or makes the backend call.
    fn misses(&self) -> u64;

    /// Number of times the backend's `search` was invoked (failed calls included).
    fn backend_calls(&self) -> u64;

    /// Number of entries a get would serve right now (stored, not expired, index version unchanged).
    fn size(&self) -> usize;
}
