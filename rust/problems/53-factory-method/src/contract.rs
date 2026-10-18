//! Provided: the product trait and the error type.

pub trait PaymentGateway {
    /// Short name, e.g. "card" or "bank".
    fn name(&self) -> &str;
    /// True if the charge was accepted.
    fn charge(&self, customer_id: &str, amount_cents: u64) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckoutError {
    InvalidCustomer,
    InvalidAmount,
}
