//! Thin test entry point. Your domain model lives behind it.

pub trait KeyValueStoreApi {
    fn put(&mut self, key: &str, value: &str);

    /// Refused (`false`, no change) when `ttl_millis <= 0`. Expired once `now >= set_at + ttl_millis`.
    fn put_with_ttl(&mut self, key: &str, value: &str, ttl_millis: i64) -> bool;

    fn get(&self, key: &str) -> Option<String>;

    /// True only if a live key was removed.
    fn delete(&mut self, key: &str) -> bool;

    /// Live keys starting with the prefix, ascending. Empty prefix = all.
    fn keys_with_prefix(&self, prefix: &str) -> Vec<String>;

    /// Number of live keys.
    fn size(&self) -> usize;

    fn begin(&mut self);

    /// Keeps the innermost transaction's changes; `false` if none is open.
    fn commit(&mut self) -> bool;

    /// Discards the innermost transaction's changes; `false` if none is open.
    fn rollback(&mut self) -> bool;
}
