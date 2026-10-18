//! Provided: the error type for stock operations.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StockError {
    /// Blank sku, zero capacity or a zero amount.
    InvalidInput,
    /// The change would take the quantity below 0 or above the capacity.
    InvariantViolation,
}
