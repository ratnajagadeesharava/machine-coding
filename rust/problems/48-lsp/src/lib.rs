//! Coupons and checkout.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// Percentage coupon.
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

/// Fixed-amount coupon.
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

/// Minimum-spend coupon.
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

/// Computes a basket total with an optional coupon.
pub fn checkout_total(prices_cents: &[i64], coupon: Option<&dyn Coupon>) -> Result<i64, CouponError> {
    todo!()
}
