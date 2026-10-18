#![allow(unused_imports, clippy::needless_range_loop)]
use p15_splitwise::{contract::*, create};
use playground_support::sut;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

fn after_applying(before: &BTreeMap<String, i64>, payments: &[String]) -> BTreeMap<String, i64> {
    let mut result = before.clone();
    for p in payments {
        let parts: Vec<&str> = p.split(['>', ':']).collect();
        let amount: i64 = parts[2].parse().unwrap();
        *result.entry(parts[0].to_string()).or_insert(0) += amount; // debtor pays: balance rises
        *result.entry(parts[1].to_string()).or_insert(0) -= amount; // creditor receives: balance falls
    }
    result.retain(|_, v| *v != 0);
    result
}

fn map(pairs: &[(&str, i64)]) -> BTreeMap<String, i64> {
    pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

#[test]
fn should_split_equally_between_participants() {
    let mut s = sut!(create());

    assert!(s.add_equal_expense("A", 300, &["A", "B", "C"]));

    assert_eq!(s.owed("B", "A"), 100);
    assert_eq!(s.owed("C", "A"), 100);
    assert_eq!(s.owed("A", "B"), 0);
    assert_eq!(s.balances(), map(&[("A", 200), ("B", -100), ("C", -100)]));
}

#[test]
fn should_give_equal_split_remainder_to_participants_in_name_order() {
    let mut s = sut!(create());

    s.add_equal_expense("D", 100, &["C", "A", "B"]);

    assert_eq!(s.owed("A", "D"), 34);
    assert_eq!(s.owed("B", "D"), 33);
    assert_eq!(s.owed("C", "D"), 33);
}

#[test]
fn should_allow_payer_who_did_not_participate() {
    let mut s = sut!(create());

    s.add_equal_expense("A", 90, &["B", "C"]);

    assert_eq!(s.owed("B", "A"), 45);
    assert_eq!(s.owed("C", "A"), 45);
    assert_eq!(s.balances().get("A"), Some(&90));
}

#[test]
fn should_create_no_debt_when_payer_is_the_only_participant() {
    let mut s = sut!(create());

    assert!(s.add_equal_expense("A", 50, &["A"]));

    assert!(s.balances().is_empty());
}

#[test]
fn should_split_by_exact_amounts() {
    let mut s = sut!(create());

    assert!(s.add_exact_expense("A", 100, &[("B", 30), ("C", 70)]));

    assert_eq!(s.owed("B", "A"), 30);
    assert_eq!(s.owed("C", "A"), 70);
}

#[test]
fn should_refuse_exact_splits_that_do_not_add_up() {
    let mut s = sut!(create());

    assert!(!s.add_exact_expense("A", 100, &[("B", 30), ("C", 60)]));
    assert!(!s.add_exact_expense("A", 100, &[("B", 130), ("C", -30)]));

    assert!(s.balances().is_empty());
}

#[test]
fn should_split_by_percent() {
    let mut s = sut!(create());

    assert!(s.add_percent_expense("A", 200, &[("B", 25), ("C", 75)]));

    assert_eq!(s.owed("B", "A"), 50);
    assert_eq!(s.owed("C", "A"), 150);
}

#[test]
fn should_distribute_percent_remainder_in_name_order() {
    let mut s = sut!(create());

    // floor(101*33/100)=33, floor(101*33/100)=33, floor(101*34/100)=34 -> 100, remainder 1 goes to "A"
    assert!(s.add_percent_expense("P", 101, &[("C", 34), ("A", 33), ("B", 33)]));

    assert_eq!(s.owed("A", "P"), 34);
    assert_eq!(s.owed("B", "P"), 33);
    assert_eq!(s.owed("C", "P"), 34);
}

#[test]
fn should_refuse_percents_that_do_not_sum_to_one_hundred() {
    let mut s = sut!(create());

    assert!(!s.add_percent_expense("A", 100, &[("B", 50), ("C", 49)]));
    assert!(!s.add_percent_expense("A", 100, &[("B", 60), ("C", 50)]));
    assert!(!s.add_percent_expense("A", 100, &[("B", 120), ("C", -20)]));
    assert!(s.balances().is_empty());
}

#[test]
fn should_refuse_invalid_equal_expenses() {
    let mut s = sut!(create());

    assert!(!s.add_equal_expense("A", 0, &["B"]));
    assert!(!s.add_equal_expense("A", -5, &["B"]));
    assert!(!s.add_equal_expense("A", 10, &[]));
    assert!(!s.add_equal_expense("A", 10, &["B", "B"]));
    assert!(s.balances().is_empty());
}

#[test]
fn should_net_debts_between_two_people_in_both_directions() {
    let mut s = sut!(create());
    s.add_equal_expense("A", 100, &["A", "B"]); // B owes A 50
    s.add_equal_expense("B", 30, &["A", "B"]); // A owes B 15

    assert_eq!(s.owed("B", "A"), 35);
    assert_eq!(s.owed("A", "B"), 0);
}

#[test]
fn should_reduce_debt_when_settled() {
    let mut s = sut!(create());
    s.add_equal_expense("A", 100, &["A", "B"]);

    assert!(s.settle("B", "A", 20));

    assert_eq!(s.owed("B", "A"), 30);
    assert!(s.settle("B", "A", 30));
    assert_eq!(s.owed("B", "A"), 0);
    assert!(s.balances().is_empty());
}

#[test]
fn should_refuse_invalid_settlements() {
    let mut s = sut!(create());

    assert!(!s.settle("B", "A", 0));
    assert!(!s.settle("B", "A", -5));
    assert!(!s.settle("A", "A", 5));
}

#[test]
fn should_return_zero_for_unknown_people() {
    let s = sut!(create());

    assert_eq!(s.owed("x", "y"), 0);
    assert!(s.balances().is_empty());
}

#[test]
fn should_simplify_chains_of_debts() {
    let mut s = sut!(create());
    s.add_equal_expense("A", 200, &["A", "B"]); // B owes A 100
    s.add_equal_expense("B", 200, &["B", "C"]); // C owes B 100

    assert_eq!(s.simplify(), vec!["C>A:100"]);
    assert_eq!(s.owed("B", "A"), 100); // simplify must not change the state
}

#[test]
fn should_simplify_any_scenario_to_at_most_n_minus_one_payments() {
    let mut s = sut!(create());
    let people = ["a", "b", "c", "d", "e", "f"];
    let mut seed = 1u64;
    let mut next = |bound: usize| {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((seed >> 33) as usize) % bound
    };
    for _ in 0..200 {
        let payer = people[next(people.len())];
        let n = 2 + next(4);
        let mut chosen: Vec<&str> = Vec::new();
        while chosen.len() < n {
            let p = people[next(people.len())];
            if !chosen.contains(&p) {
                chosen.push(p);
            }
        }
        let amount = 100 + next(900) as i64;
        assert!(s.add_equal_expense(payer, amount, &chosen));
    }
    let before = s.balances();
    assert_eq!(before.values().sum::<i64>(), 0);

    let payments = s.simplify();

    assert!(payments.len() <= before.len().saturating_sub(1));
    assert!(after_applying(&before, &payments).is_empty());
}

#[test]
fn should_stay_exact_and_fast_with_many_expenses() {
    let mut s = sut!(create());
    let names: Vec<String> = (0..1000).map(|i| format!("user{i}")).collect();
    let start = Instant::now();

    for i in 0..100_000usize {
        let participants = [
            names[(i + 1) % 1000].as_str(),
            names[(i + 2) % 1000].as_str(),
            names[(i + 3) % 1000].as_str(),
        ];
        s.add_equal_expense(&names[i % 1000], 100 + (i % 50) as i64, &participants);
    }
    assert_eq!(s.balances().values().sum::<i64>(), 0);
    s.owed("user1", "user2");
    s.simplify();

    assert!(start.elapsed() < Duration::from_secs(10));
}
