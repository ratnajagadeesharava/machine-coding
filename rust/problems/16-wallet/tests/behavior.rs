#![allow(unused_imports, clippy::needless_range_loop)]
use p16_wallet::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::sync::Arc;

fn service(wallets: &[&str]) -> Option<Arc<dyn WalletApi>> {
    let api = create()?;
    for w in wallets {
        assert!(api.create_wallet(w));
    }
    Some(api)
}

fn count(results: &[String], what: &str) -> usize {
    results.iter().filter(|r| r.as_str() == what).count()
}

#[test]
fn should_start_with_an_empty_wallet_and_refuse_duplicate_creation() {
    let api = sut!(service(&["alice"]));

    assert_eq!(api.balance("alice"), Some(0));
    assert_eq!(api.credit("k1", "alice", 50), "APPLIED");
    assert!(!api.create_wallet("alice"));
    assert!(!api.create_wallet(" "));
    assert!(!api.create_wallet(""));
    assert_eq!(api.balance("alice"), Some(50));
    assert_eq!(api.balance("nobody"), None);
}

#[test]
fn should_credit_and_debit() {
    let api = sut!(service(&["alice"]));

    assert_eq!(api.credit("k1", "alice", 1000), "APPLIED");
    assert_eq!(api.debit("k2", "alice", 400), "APPLIED");

    assert_eq!(api.balance("alice"), Some(600));
}

#[test]
fn should_allow_debit_down_to_exactly_zero_but_not_below() {
    let api = sut!(service(&["alice"]));
    api.credit("k1", "alice", 100);

    assert_eq!(api.debit("k2", "alice", 101), "INSUFFICIENT_FUNDS");
    assert_eq!(api.balance("alice"), Some(100));
    assert_eq!(api.debit("k3", "alice", 100), "APPLIED");
    assert_eq!(api.balance("alice"), Some(0));
    assert_eq!(api.debit("k4", "alice", 1), "INSUFFICIENT_FUNDS");
}

#[test]
fn should_refuse_non_positive_amounts() {
    let api = sut!(service(&["alice", "bob"]));
    api.credit("k0", "alice", 100);

    assert_eq!(api.credit("k1", "alice", 0), "INVALID_AMOUNT");
    assert_eq!(api.credit("k2", "alice", -5), "INVALID_AMOUNT");
    assert_eq!(api.debit("k3", "alice", 0), "INVALID_AMOUNT");
    assert_eq!(api.debit("k4", "alice", -1), "INVALID_AMOUNT");
    assert_eq!(api.transfer("k5", "alice", "bob", 0), "INVALID_AMOUNT");
    assert_eq!(api.transfer("k6", "alice", "bob", -10), "INVALID_AMOUNT");

    assert_eq!(api.balance("alice"), Some(100));
    assert_eq!(api.balance("bob"), Some(0));
}

#[test]
fn should_refuse_unknown_wallets_without_side_effects() {
    let api = sut!(service(&["alice"]));
    api.credit("k0", "alice", 100);

    assert_eq!(api.credit("k1", "ghost", 10), "UNKNOWN_WALLET");
    assert_eq!(api.debit("k2", "ghost", 10), "UNKNOWN_WALLET");
    assert_eq!(api.transfer("k3", "alice", "ghost", 10), "UNKNOWN_WALLET");
    assert_eq!(api.transfer("k4", "ghost", "alice", 10), "UNKNOWN_WALLET");

    assert_eq!(api.balance("alice"), Some(100));
    assert_eq!(api.statement("alice"), vec!["+100"]);
}

#[test]
fn should_move_money_between_wallets() {
    let api = sut!(service(&["alice", "bob"]));
    api.credit("k0", "alice", 1000);

    assert_eq!(api.transfer("k1", "alice", "bob", 300), "APPLIED");

    assert_eq!(api.balance("alice"), Some(700));
    assert_eq!(api.balance("bob"), Some(300));
}

#[test]
fn should_refuse_transfer_beyond_balance_and_change_nothing() {
    let api = sut!(service(&["alice", "bob"]));
    api.credit("k0", "alice", 100);

    assert_eq!(
        api.transfer("k1", "alice", "bob", 101),
        "INSUFFICIENT_FUNDS"
    );

    assert_eq!(api.balance("alice"), Some(100));
    assert_eq!(api.balance("bob"), Some(0));
    assert!(api.statement("bob").is_empty());
}

#[test]
fn should_not_apply_the_same_key_twice() {
    let api = sut!(service(&["alice"]));

    assert_eq!(api.credit("k1", "alice", 1000), "APPLIED");
    assert_eq!(api.credit("k1", "alice", 1000), "DUPLICATE");

    assert_eq!(api.balance("alice"), Some(1000));
    assert_eq!(api.statement("alice"), vec!["+1000"]);
}

#[test]
fn should_treat_a_key_as_duplicate_even_with_different_parameters() {
    let api = sut!(service(&["alice", "bob"]));
    api.credit("k1", "alice", 100);

    assert_eq!(api.credit("k1", "alice", 999), "DUPLICATE");
    assert_eq!(api.credit("k1", "bob", 100), "DUPLICATE");
    assert_eq!(api.debit("k1", "alice", 10), "DUPLICATE");
    assert_eq!(api.transfer("k1", "alice", "bob", 10), "DUPLICATE");

    assert_eq!(api.balance("alice"), Some(100));
    assert_eq!(api.balance("bob"), Some(0));
}

#[test]
fn should_not_consume_the_key_of_a_refused_request() {
    let api = sut!(service(&["alice"]));

    assert_eq!(api.debit("k1", "alice", 50), "INSUFFICIENT_FUNDS");
    api.credit("k2", "alice", 100);

    assert_eq!(api.debit("k1", "alice", 50), "APPLIED");
    assert_eq!(api.balance("alice"), Some(50));
}

#[test]
fn should_move_money_only_once_when_a_transfer_is_replayed() {
    let api = sut!(service(&["alice", "bob"]));
    api.credit("k0", "alice", 1000);

    assert_eq!(api.transfer("t1", "alice", "bob", 250), "APPLIED");
    assert_eq!(api.transfer("t1", "alice", "bob", 250), "DUPLICATE");

    assert_eq!(api.balance("alice"), Some(750));
    assert_eq!(api.balance("bob"), Some(250));
}

#[test]
fn should_record_signed_statement_entries_in_order() {
    let api = sut!(service(&["alice", "bob"]));
    api.credit("k1", "alice", 1000);
    api.transfer("k2", "alice", "bob", 300);
    api.debit("k3", "alice", 50);
    api.transfer("k4", "bob", "alice", 100);
    api.debit("k5", "bob", 9999); // refused
    api.credit("k1", "alice", 5); // duplicate

    assert_eq!(api.statement("alice"), vec!["+1000", "-300", "-50", "+100"]);
    assert_eq!(api.statement("bob"), vec!["+300", "-100"]);
    assert!(api.statement("ghost").is_empty());
}

#[test]
fn should_keep_total_money_constant_when_opposite_transfers_race() {
    let api = sut!(service(&["a", "b"]));
    api.credit("seed-a", "a", 1000);
    api.credit("seed-b", "b", 1000);

    run_concurrently(16, |t| {
        for i in 0..300usize {
            let key = format!("t{t}-{i}");
            let amount = 1 + (i % 3) as i64;
            if t % 2 == 0 {
                api.transfer(&key, "a", "b", amount);
            } else {
                api.transfer(&key, "b", "a", amount);
            }
        }
    });

    let a = api.balance("a").unwrap();
    let b = api.balance("b").unwrap();
    assert!(a >= 0);
    assert!(b >= 0);
    assert_eq!(a + b, 2000);
}

#[test]
fn should_never_overdraw_when_many_debits_race() {
    let api = sut!(service(&["alice"]));
    api.credit("seed", "alice", 100);

    let results = collect_concurrently(100, |i| api.debit(&format!("d{i}"), "alice", 3));

    assert_eq!(count(&results, "APPLIED"), 33);
    assert_eq!(count(&results, "INSUFFICIENT_FUNDS"), 67);
    assert_eq!(api.balance("alice"), Some(1));
    assert_eq!(api.statement("alice").len(), 34);
}

#[test]
fn should_apply_a_racing_key_exactly_once() {
    let api = sut!(service(&["alice"]));

    let results = collect_concurrently(32, |_| api.credit("same-key", "alice", 10));

    assert_eq!(count(&results, "APPLIED"), 1);
    assert_eq!(count(&results, "DUPLICATE"), 31);
    assert_eq!(api.balance("alice"), Some(10));
    assert_eq!(api.statement("alice"), vec!["+10"]);
}

#[test]
fn should_conserve_money_across_a_ring_of_wallets_under_contention() {
    let api = sut!(service(&["w0", "w1", "w2", "w3"]));
    for i in 0..4 {
        api.credit(&format!("seed{i}"), &format!("w{i}"), 500);
    }

    run_concurrently(16, |t| {
        for i in 0..250usize {
            let from = (t + i) % 4;
            let mut to = (t + 2 * i + 1) % 4;
            if from == to {
                to = (to + 1) % 4;
            }
            api.transfer(
                &format!("r{t}-{i}"),
                &format!("w{from}"),
                &format!("w{to}"),
                1 + ((i * 7) % 40) as i64,
            );
        }
    });

    let mut total = 0;
    for i in 0..4 {
        let balance = api.balance(&format!("w{i}")).unwrap();
        assert!(balance >= 0);
        total += balance;
    }
    assert_eq!(total, 2000);
}

#[test]
fn should_keep_statements_consistent_with_balances_after_races() {
    let api = sut!(service(&["a", "b"]));
    api.credit("seed", "a", 500);

    run_concurrently(8, |t| {
        for i in 0..100usize {
            let (from, to) = if t % 2 == 0 { ("a", "b") } else { ("b", "a") };
            api.transfer(&format!("x{t}-{i}"), from, to, 5);
        }
    });

    for w in ["a", "b"] {
        let sum: i64 = api
            .statement(w)
            .iter()
            .map(|e| e.parse::<i64>().unwrap())
            .sum();
        assert_eq!(Some(sum), api.balance(w));
    }
}
