//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

#[derive(Debug, PartialEq, Eq)]
pub enum CommentError {
    /// non-blank document id the service does not know
    UnknownDocument,
    /// blank (empty / whitespace-only) document id, reviewer id or text
    InvalidArgument,
}

pub trait CommentService: Send + Sync {
    /// Appends a comment to an existing document and returns its id: "1", "2", ... (one sequence for the whole
    /// service, in accepted order; a rejected call uses no id).
    /// Blank arguments are checked first (`InvalidArgument`), then the document (`UnknownDocument`).
    fn add_comment(
        &self,
        document_id: &str,
        reviewer_id: &str,
        text: &str,
    ) -> Result<String, CommentError>;

    /// All comments of the document, oldest first, each as `reviewer_id: text` (verbatim). Empty if there are none.
    /// The result is a copy. `InvalidArgument` for a blank id, `UnknownDocument` for an unknown one.
    fn comments(&self, document_id: &str) -> Result<Vec<String>, CommentError>;
}
