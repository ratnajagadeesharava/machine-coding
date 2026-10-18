//! Provided: the subject trait, the access policy and the error type.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    /// The policy refused this user/document.
    AccessDenied,
    /// Blank document id.
    InvalidId,
}

/// The subject interface: something that can fetch documents. The real store and every proxy implement it.
pub trait DocumentStore {
    /// The document text, or `Ok(None)` if there is no such document.
    fn fetch(&self, id: &str) -> Result<Option<String>, StoreError>;
}

/// Decides who may read what.
pub trait AccessPolicy {
    fn can_read(&self, user: &str, document_id: &str) -> bool;
}
