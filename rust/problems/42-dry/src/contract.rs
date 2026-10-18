//! Provided: the pricing decisions and the error type. Amounts are in cents.

pub trait PricingRules {
    /// Discount (>= 0) granted on a net amount.
    fn discount(&self, net: i64) -> i64;
    /// Tax (>= 0) due on an already-discounted amount.
    fn tax(&self, taxable: i64) -> i64;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocError {
    /// Empty lines, or a negative line.
    InvalidLines,
    /// Blank invoice number.
    InvalidNumber,
}
