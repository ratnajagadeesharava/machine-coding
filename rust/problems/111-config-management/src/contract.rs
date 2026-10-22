//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! A config record is reported as a single string (a "view"):
//! `<namespace>/<key>=<value> v<version> t<updated_at>`, where `updated_at` is the injected clock's epoch
//! milliseconds at the moment that version was written. Example: `app/db.host=localhost v2 t1500`.
//!
//! Argument validation always happens first, before existence and version checks, and a failed call changes nothing.

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    /// Blank namespace; empty key or key with a character outside `[a-z0-9._-]`; value longer than 256
    /// characters (an empty value is allowed); rollback to a version that never existed for the config
    /// (below 1 or above the current version).
    InvalidArgument,
    /// `create` of a (namespace, key) that already exists.
    AlreadyExists,
    /// `update` or `rollback` of a config that does not exist.
    NotFound,
}

pub trait ConfigService: Send + Sync {
    /// Creates a config at version 1 with `updated_at` = now.
    fn create(&self, namespace: &str, key: &str, value: &str) -> Result<(), ConfigError>;

    /// Replaces the value if and only if the current version equals `expected_version`; the version then goes up
    /// by exactly 1 (even when the new value equals the old one) and `updated_at` = now. `Ok(true)` on success.
    /// `Ok(false)`, changing nothing, when the current version differs from `expected_version`.
    fn update(
        &self,
        namespace: &str,
        key: &str,
        value: &str,
        expected_version: u64,
    ) -> Result<bool, ConfigError>;

    /// The view of the current version, or `None` if the config does not exist.
    fn get(&self, namespace: &str, key: &str) -> Result<Option<String>, ConfigError>;

    /// Removes the config together with its history; `Ok(true)` if it existed. Creating it again starts over at
    /// version 1.
    fn delete(&self, namespace: &str, key: &str) -> Result<bool, ConfigError>;

    /// Views of the current version of every config in the namespace, ascending by key (plain string order).
    /// Empty for an unknown namespace.
    fn list(&self, namespace: &str) -> Result<Vec<String>, ConfigError>;

    /// Views of every version the config has had, oldest first, including the current one (the last element equals
    /// what `get` returns). Empty if the config does not exist. The result is a snapshot.
    fn history(&self, namespace: &str, key: &str) -> Result<Vec<String>, ConfigError>;

    /// Creates a NEW version (current version + 1, `updated_at` = now) whose value is the value that `to_version`
    /// had. Earlier history is untouched. Rolling back to the current version is allowed and still creates a new
    /// version. Returns the new version number.
    fn rollback(&self, namespace: &str, key: &str, to_version: u64) -> Result<u64, ConfigError>;
}
