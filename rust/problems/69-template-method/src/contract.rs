//! Provided: the report and error types.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportError(pub String);

/// What an import run did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportReport {
    pub read: usize,
    pub saved: usize,
    pub skipped: usize,
    pub failed: usize,
}
