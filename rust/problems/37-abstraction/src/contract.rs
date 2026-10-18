//! Provided: the error type used by accounts.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountError {
    /// Amount was zero or negative.
    InvalidAmount,
    /// `can_withdraw` said no.
    Declined,
}
