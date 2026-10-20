#![allow(unused_imports, clippy::needless_range_loop)]
use p96_consistent_hashing::{contract::*, create};
use playground_support::sut;
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn should_look_up_a_many_keys_on_a_large_ring_quickly() {
    let r = sut!(create(100));
    for i in 0..1_000 {
        r.add_node(&format!("node-{i}"), 1).unwrap();
    }

    let start = Instant::now();
    for k in 0..300_000 {
        r.node_for(&format!("key-{k}")).unwrap();
    }

    assert!(start.elapsed().as_secs() < 30);
}

#[test]
fn should_keep_the_balance_when_many_nodes_join_and_leave() {
    let r = sut!(create(200));
    for i in 0..50 {
        r.add_node(&format!("n{i}"), 1).unwrap();
    }
    for i in 0..25 {
        r.remove_node(&format!("n{i}"));
    }
    for i in 50..75 {
        r.add_node(&format!("n{i}"), 1).unwrap();
    }

    let mut counts: HashMap<String, usize> = HashMap::new();
    for k in 0..100_000 {
        *counts
            .entry(r.node_for(&format!("key-{k}")).unwrap())
            .or_insert(0) += 1;
    }

    assert_eq!(counts.len(), 50);
    assert!(*counts.values().max().unwrap() < 3_000);
}
