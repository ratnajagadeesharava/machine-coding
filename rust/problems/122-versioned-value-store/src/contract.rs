//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

#[derive(Debug, PartialEq, Eq)]
pub enum StoreError {
    /// blank key, empty batch / duplicate key in a batch, version outside 1..=current, compaction floor outside
    /// 1..=current
    InvalidArgument,
    /// the snapshot id was never issued or was already released
    UnknownSnapshot,
    /// `get_at` for a version below the compaction floor
    Compacted,
}

pub trait VersionedStore: Send + Sync {
    /// Writes `value` for `key` and returns the new store-wide version (1, 2, 3, ... without gaps).
    /// A blank key is `InvalidArgument` (no version is consumed).
    fn put(&self, key: &str, value: &str) -> Result<u64, StoreError>;

    /// Writes all entries as ONE version and returns it: a reader sees all of them or none of them.
    /// An empty batch, a blank key or a key listed twice is `InvalidArgument`; nothing is applied and no version
    /// is consumed.
    fn put_all(&self, entries: &[(&str, &str)]) -> Result<u64, StoreError>;

    /// Deletes the key by writing a tombstone and returns `Some(version)`, or `None` if the key has no live value
    /// (never written, or already deleted); in that case nothing changes and no version is consumed.
    fn delete(&self, key: &str) -> Result<Option<u64>, StoreError>;

    /// The latest value, `None` if the key is deleted or was never written.
    fn get(&self, key: &str) -> Result<Option<String>, StoreError>;

    /// The value of `key` as of `version`: the newest write with version <= `version` (`None` if there is none or
    /// it is a tombstone). `InvalidArgument` if `version` is 0 or above `current_version()`; `Compacted` if it is
    /// below the compaction floor (see `compact`).
    fn get_at(&self, key: &str, version: u64) -> Result<Option<String>, StoreError>;

    /// Version of the latest successful write; 0 for a store that has never been written.
    fn current_version(&self) -> u64;

    /// Retained history of the key, oldest first, one string per entry: `"<version>:<value>"`, and
    /// `"<version>:<deleted>"` for a tombstone. Empty for an unknown key.
    fn history(&self, key: &str) -> Result<Vec<String>, StoreError>;

    /// Compare-and-set. Succeeds only if `expected_key_version` equals the version of the key's current live value,
    /// where 0 means "the key has no live value" (never written, or deleted). On success returns
    /// `Some(new store-wide version)`; on mismatch returns `None`, changes nothing and consumes no version.
    fn put_if_version(
        &self,
        key: &str,
        value: &str,
        expected_key_version: u64,
    ) -> Result<Option<u64>, StoreError>;

    /// Opens a read view fixed at `current_version()` right now and returns its id (a positive number, unique).
    fn snapshot(&self) -> u64;

    /// The value of the key in that snapshot's view. `UnknownSnapshot` for an unknown or released id.
    fn snapshot_get(&self, snapshot_id: u64, key: &str) -> Result<Option<String>, StoreError>;

    /// Closes the snapshot. `UnknownSnapshot` for an unknown or already released id.
    fn release_snapshot(&self, snapshot_id: u64) -> Result<(), StoreError>;

    /// Drops history that no read can need any more and returns how many entries were removed. Reads that stay
    /// answerable: `get_at` for every version >= `keep_versions_at_or_after`, `get`, and every open snapshot.
    /// Afterwards `get_at` for a version below the floor (the highest value passed so far) is `Compacted`.
    /// `InvalidArgument` unless 1 <= `keep_versions_at_or_after` <= `current_version()`.
    fn compact(&self, keep_versions_at_or_after: u64) -> Result<u64, StoreError>;
}
