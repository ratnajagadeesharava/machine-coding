//! Provided: the `Coupon` contract and the error type.
//!
//! EVERY coupon must obey ALL of this, so that code written against `Coupon` works with any of them:
//! - `code()` is non-blank.
//! - `apply(amount)` for any `amount >= 0` returns `Ok(price)` with `0 <= price <= amount`. It never returns
//!   `Err` for such an amount (even if the coupon "does not apply").
//! - `apply(0) == Ok(0)`.
//! - a negative amount gives `Err(CouponError::NegativeAmount)` from every coupon.
//! - it is deterministic.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CouponError {
    NegativeAmount,
    /// Bad constructor argument (percent outside 1..=100, negative discount ...).
    InvalidArgument,
}

pub trait Coupon {
    fn code(&self) -> &str;
    fn apply(&self, amount_cents: i64) -> Result<i64, CouponError>;
}
