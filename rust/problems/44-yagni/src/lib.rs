//! Does exactly what the README asks and nothing more.
#![allow(unused_variables)]

pub mod contract;

use contract::ShortenError;

pub struct UrlShortener;

impl UrlShortener {
    pub fn new() -> UrlShortener {
        todo!()
    }

    /// Codes are the sequence numbers 1, 2, 3, ... in lower-case base 36 ("1".."9", "a".."z", "10", ...).
    /// The same URL always gets the same code. The URL is trimmed first. Anything that is not `http://` or
    /// `https://` followed by at least one character -> `Err(ShortenError::InvalidUrl)`.
    pub fn shorten(&mut self, url: &str) -> Result<String, ShortenError> {
        todo!()
    }

    /// The original (trimmed) URL for a code, `None` if unknown.
    pub fn resolve(&self, code: &str) -> Option<String> {
        todo!()
    }
}
