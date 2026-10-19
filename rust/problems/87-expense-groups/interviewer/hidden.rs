#![allow(unused_imports, clippy::needless_range_loop)]
use p87_expense_groups::{contract::*, create};
use playground_support::sut;
use std::time::Instant;

#[test]
fn should_settle_up_twenty_people_in_reasonable_time() {
    let g = sut!(create());
    let members: Vec<String> = (0..20).map(|i| format!("p{i}")).collect();
    let refs: Vec<&str> = members.iter().map(|s| s.as_str()).collect();
    g.create_group("big", &refs).unwrap();
    let mut s = 9u64;
    for _ in 0..60 {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        let payer = (s % 20) as usize;
        let debtor = (payer + 1 + (s >> 8) as usize % 19) % 20;
        g.add_exact_expense(
            "big",
            &format!("p{payer}"),
            &[(&format!("p{debtor}"), 1 + ((s >> 20) % 100) as i64)],
        )
        .unwrap();
    }

    let start = Instant::now();
    let payments = g.settle_up("big").unwrap();

    assert!(start.elapsed().as_secs() < 10);
    assert!(payments.len() <= 19);
}

#[test]
fn should_find_the_perfect_pairing_when_it_exists() {
    let g = sut!(create());
    let members: Vec<String> = (0..10).map(|i| format!("p{i}")).collect();
    let refs: Vec<&str> = members.iter().map(|s| s.as_str()).collect();
    g.create_group("pairs", &refs).unwrap();
    for i in (0..10).step_by(2) {
        g.add_exact_expense(
            "pairs",
            &format!("p{i}"),
            &[(&format!("p{}", i + 1), 10 * (i as i64 + 1))],
        )
        .unwrap();
    }

    assert_eq!(g.settle_up("pairs").unwrap().len(), 5);
}
