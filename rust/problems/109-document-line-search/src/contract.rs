//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

pub trait DocumentSearch: Send + Sync {
    /// Ids of the live lines that contain every word of `query`, in increasing id order.
    /// Words are maximal runs of alphanumeric characters, compared case-insensitively and as whole words.
    /// A query without any word (empty, blank, punctuation only) returns an empty vector.
    fn search(&self, query: &str) -> Vec<u32>;

    /// Removes the line for good; every other line keeps its id and a deleted id is never reused.
    /// Returns true if the line was live, false if the id is unknown or was already deleted.
    fn delete_line(&self, id: u32) -> bool;
}
