//! Builder — `HttpRequest::builder()...build()`. The finished request is immutable and always valid.
#![allow(unused_variables)]

pub mod contract;

use contract::BuildError;

/// Immutable. No public fields, no public constructor: the only way to get one is the builder.
#[derive(Debug, Clone, PartialEq)]
pub struct HttpRequest;

impl HttpRequest {
    pub fn builder() -> HttpRequestBuilder {
        todo!()
    }

    /// "GET", "POST", ... (upper case).
    pub fn method(&self) -> &str {
        todo!()
    }

    /// The URL including the query string, e.g. `https://api.test/items?page=2&q=a%20b`.
    pub fn url(&self) -> &str {
        todo!()
    }

    /// (lower-case name, value) pairs in insertion order.
    pub fn headers(&self) -> &[(String, String)] {
        todo!()
    }

    /// Case-insensitive lookup.
    pub fn header(&self, name: &str) -> Option<&str> {
        todo!()
    }

    pub fn body(&self) -> Option<&str> {
        todo!()
    }

    pub fn timeout_millis(&self) -> u32 {
        todo!()
    }
}

/// Fluent, consuming builder. Defaults: method GET, timeout 5000 ms. Setters never fail: the FIRST problem is
/// remembered and reported by `build()`. `Clone` lets one half-built builder produce several requests.
#[derive(Debug, Clone)]
pub struct HttpRequestBuilder;

impl HttpRequestBuilder {
    /// GET, POST, PUT, PATCH or DELETE in any letter case, otherwise `InvalidMethod`.
    pub fn method(self, method: &str) -> Self {
        todo!()
    }

    /// Must start with "http://" or "https://" followed by something, otherwise `InvalidUrl`.
    pub fn url(self, url: &str) -> Self {
        todo!()
    }

    /// Blank name -> `InvalidName`. Names are stored lower-case. Setting a name again (any case) replaces the value
    /// in place (keeps its original position).
    pub fn header(self, name: &str, value: &str) -> Self {
        todo!()
    }

    /// Adds `name=value` to the query string, in call order. Name and value are percent-encoded: every UTF-8 byte
    /// other than A-Z a-z 0-9 '-' '_' '.' '~' becomes %XX (upper-case hex), so a space is %20. Appended with '?'
    /// (or '&' if the URL already contains '?'). Blank name -> `InvalidName`.
    pub fn query(self, name: &str, value: &str) -> Self {
        todo!()
    }

    pub fn body(self, body: &str) -> Self {
        todo!()
    }

    /// 1 to 60000 inclusive, otherwise `InvalidTimeout`.
    pub fn timeout_millis(self, millis: u32) -> Self {
        todo!()
    }

    /// Reports the first remembered problem; then `MissingUrl` if no URL was set; then `BodyNotAllowed` for a body
    /// together with GET or DELETE.
    pub fn build(self) -> Result<HttpRequest, BuildError> {
        todo!()
    }
}
