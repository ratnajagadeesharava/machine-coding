#![allow(unused_imports)]
use p103_crdt_replication::{contract::*, create};
use playground_support::{skipped, Rng};
use std::sync::Arc;
use std::time::Instant;

#[test]
fn should_give_the_same_result_for_any_merge_order_of_the_same_snapshots() {
    if create("probe").is_none() {
        return skipped();
    }
    for seed in 0..50u64 {
        let mut rng = Rng::new(seed + 1);
        let mut snaps = Vec::new();
        for id in ["a", "b", "c", "d"] {
            let r = create(id).unwrap();
            for step in 0..20u64 {
                match rng.below(5) {
                    0 => r.increment("c", rng.below(9)),
                    1 => r.decrement("c", rng.below(9)),
                    2 => r.add("s", &format!("e{}", rng.below(5))),
                    3 => r.remove("s", &format!("e{}", rng.below(5))),
                    _ => r.write("r", &format!("v{}", rng.below(9)), rng.below(10) * 100 + step),
                }
            }
            snaps.push(r.snapshot());
        }
        let first = create("x").unwrap();
        let second = create("y").unwrap();
        let mut shuffled = snaps.clone();
        for i in (1..shuffled.len()).rev() {
            shuffled.swap(i, rng.below(i as u64 + 1) as usize);
        }
        snaps.iter().for_each(|s| first.merge(s));
        shuffled.iter().for_each(|s| second.merge(s));
        shuffled.iter().for_each(|s| second.merge(s));

        assert_eq!(second.counter("c"), first.counter("c"), "seed {seed}");
        assert_eq!(second.elements("s"), first.elements("s"), "seed {seed}");
        assert_eq!(second.read("r"), first.read("r"), "seed {seed}");
    }
}

#[test]
fn should_keep_counter_snapshots_small() {
    let Some(a) = create("a") else { return skipped() };
    for _ in 0..1_000 {
        a.increment("c", 1);
    }
    let size = a.snapshot().len();
    for _ in 0..1_000 {
        a.increment("c", 1);
    }

    assert!(a.snapshot().len() < size * 2, "a counter must not grow with the number of increments");
}

#[test]
fn should_scale_to_many_elements() {
    let Some(a) = create("a") else { return skipped() };
    let b = create("b").unwrap();
    for i in 0..20_000 {
        a.add("s", &format!("e{i}"));
    }
    for i in (0..20_000).step_by(2) {
        a.remove("s", &format!("e{i}"));
    }

    let start = Instant::now();
    b.merge(&a.snapshot());

    assert_eq!(b.elements("s").len(), 10_000);
    assert!(start.elapsed().as_secs() < 10);
}
