//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

#[derive(Debug, PartialEq, Eq)]
pub enum ShortenError {
    /// null/blank/not an http:// or https:// URL
    InvalidUrl,
    /// alias does not match [A-Za-z0-9_-]{3,20}
    InvalidAlias,
    /// alias already means a different URL (or is an existing generated code)
    AliasTaken,
}

pub trait UrlShortener: Send + Sync {
    /// Short code for an http(s) URL. Shortening the same URL again returns the same code.
    fn shorten(&self, long_url: &str) -> Result<String, ShortenError>;

    /// Like `shorten` but with a caller-chosen code. Re-registering the same alias for the same URL returns it again.
    fn shorten_with_alias(&self, long_url: &str, alias: &str) -> Result<String, ShortenError>;

    /// The URL behind a code, counting one click. `None` (and no click) for an unknown or deleted code.
    fn resolve(&self, code: &str) -> Option<String>;

    /// Number of successful resolves; `None` for an unknown or deleted code.
    fn clicks(&self, code: &str) -> Option<u64>;

    /// Removes a code. Returns false if it did not exist. A deleted code is never handed out again.
    fn delete(&self, code: &str) -> bool;
}
