#![allow(unused_imports, clippy::needless_range_loop)]
use p06_atm::{contract::*, create};
use playground_support::sut;
use std::collections::BTreeMap;

fn atm() -> Option<Box<dyn AtmApi>> {
    let mut atm = create()?;
    atm.add_account("card-1", "1234", 2000);
    atm.load_cash(100, 10);
    atm.load_cash(50, 10);
    atm.load_cash(20, 10);
    atm.load_cash(10, 10);
    Some(atm)
}

fn total(notes: &BTreeMap<u32, u32>) -> u32 {
    notes.iter().map(|(d, c)| d * c).sum()
}

#[test]
fn should_authenticate_with_correct_pin() {
    let mut atm = sut!(atm());

    assert!(atm.authenticate("card-1", "1234").is_some());
}

#[test]
fn should_reject_wrong_pin_and_unknown_card() {
    let mut atm = sut!(atm());

    assert!(atm.authenticate("card-1", "0000").is_none());
    assert!(atm.authenticate("nope", "1234").is_none());
}

#[test]
fn should_lock_card_after_three_consecutive_wrong_pins() {
    let mut atm = sut!(atm());

    atm.authenticate("card-1", "1");
    atm.authenticate("card-1", "2");
    atm.authenticate("card-1", "3");

    assert!(atm.authenticate("card-1", "1234").is_none());
}

#[test]
fn should_reset_failure_count_after_successful_login() {
    let mut atm = sut!(atm());

    atm.authenticate("card-1", "1");
    atm.authenticate("card-1", "2");
    assert!(atm.authenticate("card-1", "1234").is_some());
    atm.authenticate("card-1", "3");
    atm.authenticate("card-1", "4");

    assert!(atm.authenticate("card-1", "1234").is_some());
}

#[test]
fn should_report_balance() {
    let mut atm = sut!(atm());
    let s = atm.authenticate("card-1", "1234").unwrap();

    assert_eq!(atm.balance(&s), Some(2000));
}

#[test]
fn should_dispense_notes_and_debit_account() {
    let mut atm = sut!(atm());
    let s = atm.authenticate("card-1", "1234").unwrap();

    let notes = atm.withdraw(&s, 180).unwrap();

    assert_eq!(total(&notes), 180);
    assert_eq!(notes.values().sum::<u32>(), 4); // 100+50+20+10
    assert_eq!(atm.balance(&s), Some(1820));
    assert_eq!(atm.cash_in_machine(), 1000 + 500 + 200 + 100 - 180);
}

#[test]
fn should_refuse_amounts_that_are_not_positive_multiples_of_ten() {
    let mut atm = sut!(atm());
    let s = atm.authenticate("card-1", "1234").unwrap();

    assert!(atm.withdraw(&s, 0).is_none());
    assert!(atm.withdraw(&s, 15).is_none());
    assert_eq!(atm.balance(&s), Some(2000));
}

#[test]
fn should_refuse_when_account_balance_is_insufficient() {
    let mut atm = sut!(create());
    atm.add_account("poor", "1111", 50);
    atm.load_cash(100, 10);
    atm.load_cash(10, 10);
    let s = atm.authenticate("poor", "1111").unwrap();

    assert!(atm.withdraw(&s, 60).is_none());
    assert_eq!(atm.balance(&s), Some(50));
}

#[test]
fn should_refuse_when_machine_cannot_pay_exactly_and_leave_everything_untouched() {
    let mut atm = sut!(create());
    atm.add_account("card-1", "1234", 2000);
    atm.load_cash(50, 10);
    let s = atm.authenticate("card-1", "1234").unwrap();

    assert!(atm.withdraw(&s, 30).is_none());

    assert_eq!(atm.balance(&s), Some(2000));
    assert_eq!(atm.cash_in_machine(), 500);
    assert_eq!(atm.withdraw(&s, 100), Some(BTreeMap::from([(50, 2)])));
}

#[test]
fn should_find_valid_combination_when_greedy_choice_would_fail() {
    let mut atm = sut!(create());
    atm.add_account("card-1", "1234", 2000);
    atm.load_cash(50, 1);
    atm.load_cash(20, 3);
    let s = atm.authenticate("card-1", "1234").unwrap();

    assert_eq!(atm.withdraw(&s, 60), Some(BTreeMap::from([(20, 3)])));
}

#[test]
fn should_enforce_session_withdrawal_limit() {
    let mut atm = sut!(atm());
    let s = atm.authenticate("card-1", "1234").unwrap();

    assert!(atm.withdraw(&s, 300).is_some());
    assert!(atm.withdraw(&s, 300).is_none());
    assert!(atm.withdraw(&s, 200).is_some());
    assert!(atm.withdraw(&s, 10).is_none());
}

#[test]
fn should_start_fresh_limit_in_new_session() {
    let mut atm = sut!(atm());
    let first = atm.authenticate("card-1", "1234").unwrap();
    atm.withdraw(&first, 500);
    atm.end_session(&first);

    let second = atm.authenticate("card-1", "1234").unwrap();

    assert!(atm.withdraw(&second, 500).is_some());
}

#[test]
fn should_invalidate_ended_session() {
    let mut atm = sut!(atm());
    let s = atm.authenticate("card-1", "1234").unwrap();

    atm.end_session(&s);

    assert_eq!(atm.balance(&s), None);
    assert!(atm.withdraw(&s, 10).is_none());
    assert!(!atm.deposit(&s, 10));
    assert_eq!(atm.balance("bogus"), None);
}

#[test]
fn should_credit_deposits() {
    let mut atm = sut!(atm());
    let s = atm.authenticate("card-1", "1234").unwrap();

    assert!(atm.deposit(&s, 250));

    assert_eq!(atm.balance(&s), Some(2250));
}

#[test]
fn should_reject_non_positive_deposits() {
    let mut atm = sut!(atm());
    let s = atm.authenticate("card-1", "1234").unwrap();

    assert!(!atm.deposit(&s, 0));
    assert_eq!(atm.balance(&s), Some(2000));
}
