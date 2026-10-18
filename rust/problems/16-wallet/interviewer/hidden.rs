#![allow(unused_imports, clippy::needless_range_loop)]
use p16_wallet::{contract::*, create};
use playground_support::sut;
use std::sync::Arc;

fn service(wallets: &[&str]) -> Option<Arc<dyn WalletApi>> {
    let api = create()?;
    for w in wallets {
        api.create_wallet(w);
    }
    Some(api)
}

#[test]
fn transfer_to_the_same_wallet_should_be_refused_as_invalid() {
    let api = sut!(service(&["alice"]));
    api.credit("k0", "alice", 100);

    assert_eq!(api.transfer("k1", "alice", "alice", 10), "INVALID_AMOUNT");

    assert_eq!(api.balance("alice"), Some(100));
    assert_eq!(api.statement("alice"), vec!["+100"]);
}

#[test]
fn duplicate_should_win_over_any_other_check() {
    let api = sut!(service(&["alice"]));
    api.credit("k1", "alice", 100);

    assert_eq!(api.credit("k1", "alice", 0), "DUPLICATE");
    assert_eq!(api.debit("k1", "ghost", 5), "DUPLICATE");
    assert_eq!(api.debit("k1", "alice", 1_000_000), "DUPLICATE");
}

#[test]
fn invalid_amount_should_win_over_unknown_wallet_and_unknown_wallet_over_funds() {
    let api = sut!(service(&["alice"]));

    assert_eq!(api.debit("k1", "ghost", 0), "INVALID_AMOUNT");
    assert_eq!(api.transfer("k2", "ghost", "alice", -1), "INVALID_AMOUNT");
    assert_eq!(api.debit("k3", "ghost", 10), "UNKNOWN_WALLET");
    assert_eq!(api.transfer("k4", "alice", "ghost", 10), "UNKNOWN_WALLET");
    assert_eq!(api.debit("k5", "alice", 10), "INSUFFICIENT_FUNDS");
}

#[test]
fn an_unknown_wallet_refusal_should_not_burn_the_key() {
    let api = sut!(service(&["alice"]));
    api.credit("k0", "alice", 100);

    assert_eq!(api.transfer("k1", "alice", "bob", 40), "UNKNOWN_WALLET");
    api.create_wallet("bob");

    assert_eq!(api.transfer("k1", "alice", "bob", 40), "APPLIED");
    assert_eq!(api.balance("alice"), Some(60));
    assert_eq!(api.balance("bob"), Some(40));
}

#[test]
fn should_transfer_the_whole_balance_and_keep_an_independent_statement_snapshot() {
    let api = sut!(service(&["alice", "bob"]));
    api.credit("k0", "alice", 70);

    let before = api.statement("alice");
    assert_eq!(api.transfer("k1", "alice", "bob", 70), "APPLIED");

    assert_eq!(api.balance("alice"), Some(0));
    assert_eq!(before, vec!["+70"]);
    assert_eq!(api.statement("alice"), vec!["+70", "-70"]);
}
