//! Provided: the remote API trait, its page type and the error type.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedError(pub String);

/// One page of results and whether another page follows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub items: Vec<String>,
    pub has_next: bool,
}

/// The remote API. Every call is slow and costs money.
pub trait PageFetcher {
    /// Page numbers start at 0. A page may be empty even when `has_next` is true.
    fn fetch(&self, page: usize) -> Result<Page, FeedError>;
}
