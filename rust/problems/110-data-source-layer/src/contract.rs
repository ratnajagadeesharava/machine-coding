//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

#[derive(Debug, PartialEq, Eq)]
pub enum DataError {
    /// blank key
    InvalidKey,
    /// the current source refuses this value (API: longer than 32 characters)
    ValueTooLong,
    /// not exactly "DATABASE", "API" or "FILE"
    UnknownSource,
}

pub trait DataAccess: Send + Sync {
    /// Value stored under `key` in the current source.
    fn get(&self, key: &str) -> Result<Option<String>, DataError>;

    /// Stores or overwrites a record in the current source only. A refused put changes nothing.
    fn put(&self, key: &str, value: &str) -> Result<(), DataError>;

    /// Removes the record from the current source; `Ok(true)` if there was one.
    fn remove(&self, key: &str) -> Result<bool, DataError>;

    /// Number of records in the current source.
    fn size(&self) -> usize;

    /// Makes `source_name` (exactly "DATABASE", "API" or "FILE") the current source for every later call.
    /// `Err(UnknownSource)` for any other name; the current source is then unchanged.
    fn switch_source(&self, source_name: &str) -> Result<(), DataError>;

    /// Name of the current source. Before any switch it is "DATABASE".
    fn current_source(&self) -> String;
}
