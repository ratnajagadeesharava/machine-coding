use p48_lsp::contract::*;
use p48_lsp::{checkout_total, FlatCoupon, MinSpendCoupon, PercentCoupon};

const AMOUNTS: [i64; 10] = [0, 1, 99, 100, 999, 1000, 1001, 10_000, 123_456, 1_000_000_000];

/// The shared suite: every coupon must pass ALL of it. A type that cannot is not substitutable for `Coupon`.
fn contract_suite(c: &dyn Coupon) {
    assert!(!c.code().trim().is_empty(), "code must be non-blank");
    assert_eq!(c.apply(0), Ok(0), "apply(0) must be 0");
    for amount in AMOUNTS {
        let price = c.apply(amount).unwrap_or_else(|e| panic!("apply({amount}) must not fail, got {e:?}"));
        assert!(price >= 0, "apply({amount}) = {price} is negative");
        assert!(price <= amount, "apply({amount}) = {price} increased the price");
        assert_eq!(c.apply(amount), Ok(price), "apply({amount}) must be deterministic");
    }
    assert_eq!(c.apply(-1), Err(CouponError::NegativeAmount));
    assert_eq!(c.apply(i64::MIN), Err(CouponError::NegativeAmount));
}

/// A new type invented in the test: it applies one coupon and then another. It only passes the shared suite if
/// the coupons it is built from honour the contract, and `checkout_total` must accept it unchanged.
struct Stacked(Box<dyn Coupon>, Box<dyn Coupon>);

impl Coupon for Stacked {
    fn code(&self) -> &str {
        "STACK"
    }
    fn apply(&self, amount: i64) -> Result<i64, CouponError> {
        self.0.apply(amount).and_then(|reduced| self.1.apply(reduced))
    }
}

fn stacked() -> Stacked {
    Stacked(Box::new(PercentCoupon::new("P", 10).unwrap()), Box::new(FlatCoupon::new("F", 100).unwrap()))
}

#[test]
fn percent_coupon_should_honour_the_contract() {
    contract_suite(&PercentCoupon::new("SAVE25", 25).unwrap());
    contract_suite(&PercentCoupon::new("ALL", 100).unwrap());
}

#[test]
fn flat_coupon_should_honour_the_contract() {
    contract_suite(&FlatCoupon::new("FIVE", 500).unwrap());
    contract_suite(&FlatCoupon::new("HUGE", i64::MAX).unwrap());
}

#[test]
fn min_spend_coupon_should_honour_the_contract() {
    contract_suite(&MinSpendCoupon::new("BIG", 1000, 300).unwrap());
}

#[test]
fn a_coupon_defined_in_the_test_should_honour_the_contract_too() {
    contract_suite(&stacked());
    assert_eq!(checkout_total(&[600, 400], Some(&stacked())), Ok(800));
}

#[test]
fn should_compute_each_coupons_own_discount() {
    assert_eq!(PercentCoupon::new("A", 25).unwrap().apply(1001), Ok(751));
    assert_eq!(PercentCoupon::new("B", 10).unwrap().apply(1000), Ok(900));
    assert_eq!(FlatCoupon::new("A", 500).unwrap().apply(1000), Ok(500));
    assert_eq!(FlatCoupon::new("A", 500).unwrap().apply(120), Ok(0));

    let min = MinSpendCoupon::new("BIG", 1000, 300).unwrap();
    assert_eq!(min.apply(999), Ok(999));
    assert_eq!(min.apply(1000), Ok(700));
    assert_eq!(min.apply(5000), Ok(4700));
}

#[test]
fn should_reject_bad_constructor_arguments() {
    assert!(matches!(PercentCoupon::new("X", 0), Err(CouponError::InvalidArgument)));
    assert!(matches!(PercentCoupon::new("X", 101), Err(CouponError::InvalidArgument)));
    assert!(matches!(FlatCoupon::new("X", -1), Err(CouponError::InvalidArgument)));
}

#[test]
fn should_sum_the_prices_and_apply_the_coupon_once() {
    let flat = FlatCoupon::new("F", 500).unwrap();
    let pct = PercentCoupon::new("P", 10).unwrap();

    assert_eq!(checkout_total(&[600, 600], Some(&flat)), Ok(700));
    assert_eq!(checkout_total(&[500, 500], Some(&pct)), Ok(900));
}

#[test]
fn should_treat_every_coupon_type_the_same_way() {
    let coupons: Vec<Box<dyn Coupon>> = vec![
        Box::new(PercentCoupon::new("P", 50).unwrap()),
        Box::new(FlatCoupon::new("F", 100).unwrap()),
        Box::new(MinSpendCoupon::new("M", 5000, 100).unwrap()),
    ];

    let totals: Vec<i64> = coupons.iter().map(|c| checkout_total(&[300, 200], Some(c.as_ref())).unwrap()).collect();

    assert_eq!(totals, vec![250, 400, 500]);
}

#[test]
fn should_handle_no_coupon_empty_baskets_and_negative_prices() {
    assert_eq!(checkout_total(&[300, 200], None), Ok(500));
    assert_eq!(checkout_total(&[], Some(&FlatCoupon::new("F", 100).unwrap())), Ok(0));
    assert_eq!(checkout_total(&[100, -1], None), Err(CouponError::NegativeAmount));
}
