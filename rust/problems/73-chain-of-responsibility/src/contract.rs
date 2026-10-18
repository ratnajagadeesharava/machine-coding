//! Provided: the request type and the decider trait.

/// An expense claim. `amount_cents` is in cents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpenseRequest {
    pub employee: String,
    pub amount_cents: i64,
    pub category: String,
}

/// One link of the chain: either DECIDES a request (`Some(decision)`) or passes it on (`None`).
pub trait Decider {
    fn decide(&self, request: &ExpenseRequest) -> Option<String>;
}
