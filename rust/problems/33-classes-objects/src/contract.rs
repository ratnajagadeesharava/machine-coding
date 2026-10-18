//! Provided: the error type your `Money` returns.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoneyError {
    /// Currency is not exactly three upper-case ASCII letters.
    InvalidCurrency,
    /// Operation between two different currencies.
    CurrencyMismatch,
    /// The result does not fit in an `i64`.
    Overflow,
}
