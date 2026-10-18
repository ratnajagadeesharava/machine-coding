//! Thin test entry point. Your domain model lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

pub trait CacheApi: Send + Sync {
    /// The live value for the key, if any.
    fn get(&self, key: &str) -> Option<String>;

    fn put(&self, key: &str, value: &str);

    /// Cached value, or the result of the loader (computed once for concurrent callers of the same
    /// missing key). A loader `Err` is returned unchanged to every caller waiting for that computation.
    fn get_or_compute(
        &self,
        key: &str,
        loader: &(dyn Fn(&str) -> Result<String, String> + Sync),
    ) -> Result<String, String>;

    /// Number of live (non-expired) entries.
    fn size(&self) -> usize;
}
