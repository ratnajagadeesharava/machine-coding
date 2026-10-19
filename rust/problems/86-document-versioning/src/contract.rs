//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! A document is text; its lines are `content.split('\n')` and are joined again with "\n".
//! Versions are numbered 1, 2, 3 ... and are never rewritten (history is append-only).

#[derive(Debug, PartialEq, Eq)]
pub enum DocError {
    AlreadyExists,
    UnknownDocument,
    /// version outside 1..=latest
    UnknownVersion,
    /// a stale edit overlaps a change made since its base version
    Conflict,
}

pub type Doc<T> = Result<T, DocError>;

pub trait DocStore: Send + Sync {
    /// Creates version 1.
    fn create(&self, doc_id: &str, author: &str, content: &str) -> Doc<()>;

    /// Number of the newest version.
    fn latest(&self, doc_id: &str) -> Doc<usize>;

    /// Content of a version.
    fn content(&self, doc_id: &str, version: usize) -> Doc<String>;

    /// Saves `new_content`, which the author edited starting from `base_version`, and returns the version that now
    /// holds the result.
    /// * base == latest: if the content is unchanged nothing is created and latest is returned; else a new version.
    /// * base < latest (stale): three-way merge of base, latest and `new_content` (see the README for the exact
    ///   rules). A clean merge creates a new version with the merged content (or returns latest if the merge equals
    ///   latest); an overlapping change returns `Err(Conflict)` and creates nothing.
    /// * `Err(UnknownVersion)` if base is not in 1..=latest.
    fn edit(
        &self,
        doc_id: &str,
        author: &str,
        base_version: usize,
        new_content: &str,
    ) -> Doc<usize>;

    /// Creates a new version holding the content of an old one (returns latest, creating nothing, if equal).
    fn restore(&self, doc_id: &str, author: &str, version: usize) -> Doc<usize>;

    /// One entry per version, oldest first, formatted "version:author".
    fn history(&self, doc_id: &str) -> Doc<Vec<String>>;

    /// Line diff between two versions, one entry per output line: "  text" (unchanged), "- text" (removed),
    /// "+ text" (added). Exact algorithm in the README.
    fn diff(&self, doc_id: &str, from_version: usize, to_version: usize) -> Doc<Vec<String>>;
}
