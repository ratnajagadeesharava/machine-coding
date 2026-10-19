//! Builder.
#![allow(unused_variables)]

pub mod contract;

use contract::BuildError;

/// An HTTP request description.
#[derive(Debug, Clone, PartialEq)]
pub struct HttpRequest;

impl HttpRequest {
    pub fn builder() -> HttpRequestBuilder {
        todo!()
    }

    pub fn method(&self) -> &str {
        todo!()
    }

    pub fn url(&self) -> &str {
        todo!()
    }

    pub fn headers(&self) -> &[(String, String)] {
        todo!()
    }

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

/// Builder for `HttpRequest`.
#[derive(Debug, Clone)]
pub struct HttpRequestBuilder;

impl HttpRequestBuilder {
    pub fn method(self, method: &str) -> Self {
        todo!()
    }

    pub fn url(self, url: &str) -> Self {
        todo!()
    }

    pub fn header(self, name: &str, value: &str) -> Self {
        todo!()
    }

    pub fn query(self, name: &str, value: &str) -> Self {
        todo!()
    }

    pub fn body(self, body: &str) -> Self {
        todo!()
    }

    pub fn timeout_millis(self, millis: u32) -> Self {
        todo!()
    }

    pub fn build(self) -> Result<HttpRequest, BuildError> {
        todo!()
    }
}
