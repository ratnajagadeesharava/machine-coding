//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Time comes from the clock handed to `create`.

use std::sync::Arc;

/// A service behind the gateway (provided by the tests). Thread-safe; fails by returning `Err`.
/// Any `Fn(&str) -> Result<String, String>` closure is a backend.
pub trait Backend: Send + Sync {
    fn handle(&self, path: &str) -> Result<String, String>;
}

impl<F> Backend for F
where
    F: Fn(&str) -> Result<String, String> + Send + Sync,
{
    fn handle(&self, path: &str) -> Result<String, String> {
        self(path)
    }
}

/// What the gateway answers. Provided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub body: String,
}

impl Response {
    pub fn new(status: u16, body: &str) -> Response {
        Response { status, body: body.to_string() }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum GatewayError {
    /// unknown backend id, empty backend list
    InvalidRoute,
    UnknownName,
}

pub trait Gateway: Send + Sync {
    fn register_backend(&self, id: &str, backend: Arc<dyn Backend>);

    /// Routes every path that starts with `path_prefix` to the backends (in the given order). When several prefixes
    /// match, the longest one wins.
    fn add_route(&self, path_prefix: &str, backend_ids: &[&str]) -> Result<(), GatewayError>;

    /// Registers an API key. The client may make `requests_per_minute` requests per minute: a token bucket with that
    /// capacity, full at registration, refilled continuously at requests_per_minute / 60 tokens per second
    /// (fractions accumulate). Each request that passes authentication and routing costs one token.
    fn add_client(&self, api_key: &str, client_name: &str, requests_per_minute: u32);

    /// Handles a request, checking in this order:
    /// 1. unknown API key: 401 "unauthorized";
    /// 2. no route: 404 "no route" (costs no token);
    /// 3. no token left: 429 "rate limited";
    /// 4. pick backends: the route's backends whose circuit is not open, starting with the next one in round-robin
    ///    order (one shared counter per route) and failing over to the following ones when a backend fails. First
    ///    success: 200 with the backend's body. If every candidate failed: 502 "bad gateway". If every backend's
    ///    circuit is open: 503 "service unavailable" (nothing is called).
    ///
    /// Circuit breaker per backend: 3 consecutive failures open it for 30 seconds; after that one trial request is
    /// let through (half-open): success closes it, failure opens it for another 30 seconds. Any success resets the
    /// consecutive-failure count.
    fn handle(&self, api_key: &str, path: &str) -> Response;

    /// Number of requests answered 200 for the client.
    fn success_count(&self, client_name: &str) -> Result<usize, GatewayError>;

    /// Number of times the backend was called (successful or not).
    fn call_count(&self, backend_id: &str) -> Result<usize, GatewayError>;
}
