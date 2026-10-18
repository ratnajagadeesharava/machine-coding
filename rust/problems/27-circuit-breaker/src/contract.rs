//! Thin test entry point. Your domain model lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

pub trait CircuitBreakerApi: Send + Sync {
    /// Runs the operation unless refused. The operation returns `true` for success.
    /// Returns "SUCCESS", "FAILURE" or "REJECTED" (refused, operation not run).
    fn call(&self, operation: &dyn Fn() -> bool) -> String;

    /// "CLOSED", "OPEN" or "HALF_OPEN", at the current time.
    fn state(&self) -> String;
}
