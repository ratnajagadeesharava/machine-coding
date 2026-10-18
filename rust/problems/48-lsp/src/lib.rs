//! LSP — three coupons that all honour the `Coupon` contract, and a `checkout` that trusts it.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// percent in 1..=100 (else `InvalidArgument`). Discount = amount * percent / 100, rounded down.
pub struct PercentCoupon;

impl PercentCoupon {
    pub fn new(code: &str, percent: u32) -> Result<PercentCoupon, CouponError> {
        todo!()
    }
}

impl Coupon for PercentCoupon {
    fn code(&self) -> &str {
        todo!()
    }
    fn apply(&self, amount_cents: i64) -> Result<i64, CouponError> {
        todo!()
    }
}

/// Takes a fixed number of cents off (`off >= 0`, else `InvalidArgument`). The price never goes below 0.
pub struct FlatCoupon;

impl FlatCoupon {
    pub fn new(code: &str, off_cents: i64) -> Result<FlatCoupon, CouponError> {
        todo!()
    }
}

impl Coupon for FlatCoupon {
    fn code(&self) -> &str {
        todo!()
    }
    fn apply(&self, amount_cents: i64) -> Result<i64, CouponError> {
        todo!()
    }
}

/// Takes `off` off, but only when the amount is at least `min_spend`. Below the minimum the amount is returned
/// unchanged (it must NOT be an error). The price never goes below 0.
pub struct MinSpendCoupon;

impl MinSpendCoupon {
    pub fn new(code: &str, min_spend_cents: i64, off_cents: i64) -> Result<MinSpendCoupon, CouponError> {
        todo!()
    }
}

impl Coupon for MinSpendCoupon {
    fn code(&self) -> &str {
        todo!()
    }
    fn apply(&self, amount_cents: i64) -> Result<i64, CouponError> {
        todo!()
    }
}

/// Sum of the prices, then the coupon is applied ONCE to that sum. `None` coupon = no discount. Empty list = 0.
/// A negative price -> `Err(CouponError::NegativeAmount)`. Works with ANY coupon: no special cases per type.
pub fn checkout_total(prices_cents: &[i64], coupon: Option<&dyn Coupon>) -> Result<i64, CouponError> {
    todo!()
}
