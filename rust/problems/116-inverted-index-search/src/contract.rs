//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

#[derive(Debug, PartialEq, Eq)]
pub enum SearchError {
    /// blank doc id / mode other than "AND" or "OR"
    InvalidArgument,
    /// `add_document` with the id of a live document
    DuplicateDocument,
    /// `update_document` / `delete_document` / `score_of` with an id that is not live
    UnknownDocument,
}

pub trait SearchEngine: Send + Sync {
    /// Adds a document. `InvalidArgument` if `doc_id` is blank, `DuplicateDocument` if a live document has that id.
    /// Empty text (or text without any token) is allowed.
    fn add_document(&self, doc_id: &str, text: &str) -> Result<(), SearchError>;

    /// Replaces the text of a live document; visible to the very next search. `UnknownDocument` if it is not live.
    fn update_document(&self, doc_id: &str, text: &str) -> Result<(), SearchError>;

    /// Removes a live document; visible to the very next search. `UnknownDocument` if it is not live.
    /// The id may be added again later.
    fn delete_document(&self, doc_id: &str) -> Result<(), SearchError>;

    /// Same as `search_page(query, mode, 0, k)`: the `k` best documents. `mode` is exactly "AND" or "OR"
    /// (`InvalidArgument` otherwise).
    fn search(&self, query: &str, mode: &str, k: usize) -> Result<Vec<String>, SearchError>;

    /// The documents ranked `offset ..= offset + limit - 1` of the full ranking (best first), as doc ids.
    /// Ranking: TF-IDF score descending, ties by doc id ascending (formula in the README). Fewer (or none)
    /// are returned when the ranking is shorter. `InvalidArgument` for a mode other than "AND"/"OR".
    fn search_page(
        &self,
        query: &str,
        mode: &str,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<String>, SearchError>;

    /// TF-IDF score of a live document for the query: the sum over the distinct query terms that occur in the
    /// document, independent of the mode. 0.0 when no query term occurs in it. `UnknownDocument` if not live.
    fn score_of(&self, query: &str, doc_id: &str) -> Result<f64, SearchError>;

    /// Number of live documents.
    fn document_count(&self) -> usize;
}
