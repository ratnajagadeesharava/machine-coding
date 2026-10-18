#![allow(unused_imports, clippy::needless_range_loop)]
use p06_atm::{contract::*, create};
use playground_support::sut;
use std::collections::BTreeMap;

fn atm() -> Option<Box<dyn AtmApi>> {
    let mut atm = create()?;
    atm.add_account("c", "1234", 5000);
    atm.load_cash(100, 20);
    atm.load_cash(10, 20);
    Some(atm)
}

#[test]
fn wrong_pins_must_be_consecutive_to_lock() {
    let mut atm = sut!(atm());
    atm.authenticate("c", "x");
    atm.authenticate("c", "x");
    atm.authenticate("c", "1234");
    atm.authenticate("c", "x");
    atm.authenticate("c", "x");
    atm.authenticate("c", "x");
    assert!(atm.authenticate("c", "1234").is_none());
}

#[test]
fn refused_withdrawals_should_not_consume_session_limit() {
    let mut atm = sut!(atm());
    let s = atm.authenticate("c", "1234").unwrap();
    assert!(atm.withdraw(&s, 505).is_none());
    assert!(atm.withdraw(&s, 600).is_none());
    assert!(atm.withdraw(&s, 500).is_some());
}

#[test]
fn limit_is_inclusive_at_five_hundred() {
    let mut atm = sut!(atm());
    let s = atm.authenticate("c", "1234").unwrap();
    assert!(atm.withdraw(&s, 490).is_some());
    assert!(atm.withdraw(&s, 20).is_none());
    assert!(atm.withdraw(&s, 10).is_some());
}

#[test]
fn should_prefer_fewest_notes() {
    let mut atm = sut!(atm());
    let s = atm.authenticate("c", "1234").unwrap();
    assert_eq!(
        atm.withdraw(&s, 130),
        Some(BTreeMap::from([(100, 1), (10, 3)]))
    );
}
