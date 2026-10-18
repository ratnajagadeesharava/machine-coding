use p33_classes_objects::{contract::MoneyError, Money};
use std::collections::HashSet;

fn usd(n: i64) -> Money {
    Money::new("USD", n).unwrap()
}

#[test]
fn should_expose_currency_and_minor_units() {
    let m = usd(1250);

    assert_eq!(m.currency(), "USD");
    assert_eq!(m.minor_units(), 1250);
    assert_eq!(Money::zero("EUR").unwrap().minor_units(), 0);
}

#[test]
fn should_treat_equal_values_as_equal_and_hash_alike() {
    let a = usd(500);
    let b = usd(500);

    assert_eq!(a, b);
    assert_ne!(a, usd(501));
    assert_ne!(a, Money::new("EUR", 500).unwrap());
    let set: HashSet<Money> = [a.clone()].into_iter().collect();
    assert!(set.contains(&b));
}

#[test]
fn should_return_new_values_and_leave_operands_untouched() {
    let a = usd(1000);
    let b = usd(250);

    assert_eq!(a.plus(&b).unwrap(), usd(1250));
    assert_eq!(a.minus(&b).unwrap(), usd(750));
    let negative = b.minus(&a).unwrap();
    assert_eq!(negative, usd(-750));
    assert!(negative.is_negative());
    assert!(!a.is_negative());
    assert_eq!(a.times(3).unwrap(), usd(3000));
    assert_eq!(a, usd(1000));
    assert_eq!(b, usd(250));
}

#[test]
fn should_print_two_decimal_places() {
    assert_eq!(usd(1250).to_string(), "USD 12.50");
    assert_eq!(Money::new("EUR", 5).unwrap().to_string(), "EUR 0.05");
    assert_eq!(Money::zero("GBP").unwrap().to_string(), "GBP 0.00");
    assert_eq!(usd(-5).to_string(), "USD -0.05");
    assert_eq!(usd(100_000).to_string(), "USD 1000.00");
}

#[test]
fn should_reject_mixing_currencies() {
    let eur = Money::new("EUR", 100).unwrap();

    assert_eq!(usd(100).plus(&eur), Err(MoneyError::CurrencyMismatch));
    assert_eq!(usd(100).minus(&eur), Err(MoneyError::CurrencyMismatch));
    assert_eq!(usd(100).partial_cmp(&eur), None);
}

#[test]
fn should_reject_invalid_currency_codes() {
    assert_eq!(Money::new("usd", 1), Err(MoneyError::InvalidCurrency));
    assert_eq!(Money::new("US", 1), Err(MoneyError::InvalidCurrency));
    assert_eq!(Money::new("", 1), Err(MoneyError::InvalidCurrency));
    assert_eq!(Money::zero("DOLLARS"), Err(MoneyError::InvalidCurrency));
}

#[test]
fn should_order_amounts_of_the_same_currency() {
    let mut list = vec![usd(300), usd(-10), usd(120)];

    list.sort_by(|a, b| a.partial_cmp(b).unwrap());

    assert_eq!(list, vec![usd(-10), usd(120), usd(300)]);
    assert!(usd(5) <= usd(5));
    assert!(usd(4) < usd(5));
}

#[test]
fn should_fail_loudly_on_overflow() {
    let big = usd(i64::MAX);

    assert_eq!(big.plus(&usd(1)), Err(MoneyError::Overflow));
    assert_eq!(big.times(2), Err(MoneyError::Overflow));
}
