//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Time comes from the clock handed to `create`.

/// The business operation behind an idempotency key (provided by the tests). It fails by returning `Err`.
pub type HandlerFn<'a> = &'a (dyn Fn(&str) -> Result<String, String> + Sync);

#[derive(Debug, PartialEq, Eq)]
pub enum IdemError {
    /// the same key was reused for a different request body
    KeyConflict,
    /// blank key
    InvalidInput,
    /// the handler failed (nothing was stored); carries its error
    HandlerFailed(String),
}

pub trait IdempotentProcessor: Send + Sync {
    /// Runs `handler` for the request at most once per idempotency key and returns its response; every other call
    /// with the same key and the same body returns the stored response without running the handler.
    ///
    /// * Same key, DIFFERENT body (also while the first call is still running): `Err(KeyConflict)`.
    /// * Concurrent calls with the same key and body: one runs the handler, the others wait for it and return the same
    ///   response. Calls with different keys never wait for each other.
    /// * If the handler fails, nothing is stored: the error reaches the caller that ran it (`HandlerFailed`), and the
    ///   key is free again. Callers that were waiting for it do not fail; one of them runs the handler next.
    /// * A stored response expires 24 hours after it was stored (then the key can be used again).
    fn process(&self, idempotency_key: &str, request_body: &str, handler: HandlerFn<'_>) -> Result<String, IdemError>;
}
