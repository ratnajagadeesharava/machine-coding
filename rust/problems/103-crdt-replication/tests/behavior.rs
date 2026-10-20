#![allow(unused_imports, clippy::needless_range_loop)]
use p103_crdt_replication::{contract::*, create};
use playground_support::{run_concurrently, skipped, sut, Rng};
use std::sync::Arc;

fn replica(id: &str) -> Option<Arc<dyn Replica>> {
    create(id)
}

/// Both directions: afterwards a and b have seen everything either has seen.
fn sync(a: &Arc<dyn Replica>, b: &Arc<dyn Replica>) {
    let from_a = a.snapshot();
    let from_b = b.snapshot();
    a.merge(&from_b);
    b.merge(&from_a);
}

fn sorted(set: std::collections::HashSet<String>) -> Vec<String> {
    let mut v: Vec<String> = set.into_iter().collect();
    v.sort();
    v
}

#[test]
fn should_count_locally_and_start_at_zero() {
    let a = sut!(replica("a"));

    assert_eq!(a.counter("likes"), 0);
    a.increment("likes", 5);
    a.decrement("likes", 2);
    a.increment("likes", 1);

    assert_eq!(a.counter("likes"), 4);
    assert_eq!(a.counter("other"), 0);
}

#[test]
fn should_not_lose_concurrent_increments_on_different_replicas() {
    let a = sut!(replica("a"));
    let b = replica("b").unwrap();

    a.increment("views", 5);
    b.increment("views", 3);
    b.decrement("views", 1);
    sync(&a, &b);

    assert_eq!(a.counter("views"), 7);
    assert_eq!(b.counter("views"), 7);
}

#[test]
fn should_not_count_twice_when_the_same_snapshot_is_merged_again() {
    let a = sut!(replica("a"));
    let b = replica("b").unwrap();
    a.increment("c", 10);
    let snap = a.snapshot();

    b.merge(&snap);
    b.merge(&snap);
    b.merge(&a.snapshot());

    assert_eq!(b.counter("c"), 10);
}

#[test]
fn should_not_revert_newer_state_when_an_old_snapshot_is_merged() {
    let a = sut!(replica("a"));
    let b = replica("b").unwrap();
    a.increment("c", 1);
    let old = a.snapshot();
    a.increment("c", 4);
    b.merge(&a.snapshot());

    b.merge(&old);

    assert_eq!(b.counter("c"), 5);
}

#[test]
fn should_take_snapshots_by_value() {
    let a = sut!(replica("a"));
    let b = replica("b").unwrap();
    a.increment("c", 1);
    let snap = a.snapshot();
    a.increment("c", 100);
    a.add("s", "x");

    b.merge(&snap);

    assert_eq!(b.counter("c"), 1);
    assert!(!b.contains("s", "x"));
}

#[test]
fn should_add_remove_and_re_add_elements() {
    let a = sut!(replica("a"));

    a.add("tags", "x");
    a.add("tags", "y");
    assert_eq!(sorted(a.elements("tags")), ["x", "y"]);

    a.remove("tags", "x");
    assert!(!a.contains("tags", "x"));
    assert_eq!(sorted(a.elements("tags")), ["y"]);

    a.add("tags", "x");
    assert!(a.contains("tags", "x"));
    a.remove("tags", "nope");
    assert_eq!(sorted(a.elements("tags")), ["x", "y"]);
}

#[test]
fn should_return_a_copy_of_the_elements() {
    let a = sut!(replica("a"));
    a.add("s", "x");
    let copy = a.elements("s");

    a.add("s", "y");

    assert_eq!(sorted(copy), ["x"]);
}

#[test]
fn should_replicate_additions_and_removals() {
    let a = sut!(replica("a"));
    let b = replica("b").unwrap();
    a.add("s", "x");
    sync(&a, &b);
    assert!(b.contains("s", "x"));

    b.remove("s", "x");
    sync(&a, &b);

    assert!(!a.contains("s", "x"));
    assert!(!b.contains("s", "x"));
}

#[test]
fn should_let_a_concurrent_add_win_over_a_remove() {
    let a = sut!(replica("a"));
    let b = replica("b").unwrap();
    a.add("s", "x");
    sync(&a, &b);

    a.remove("s", "x");
    b.add("s", "x");
    sync(&a, &b);

    assert!(a.contains("s", "x"));
    assert!(b.contains("s", "x"));
}

#[test]
fn should_only_remove_what_the_remover_had_observed() {
    let a = sut!(replica("a"));
    let b = replica("b").unwrap();
    a.add("s", "x");
    b.remove("s", "x");
    sync(&a, &b);

    assert!(a.contains("s", "x"));
    assert!(b.contains("s", "x"));
}

#[test]
fn should_not_resurrect_a_removed_element_from_a_stale_snapshot() {
    let a = sut!(replica("a"));
    let b = replica("b").unwrap();
    a.add("s", "x");
    let stale = a.snapshot();
    b.merge(&stale);
    b.remove("s", "x");
    a.merge(&b.snapshot());

    b.merge(&stale);
    a.merge(&stale);

    assert!(!a.contains("s", "x"));
    assert!(!b.contains("s", "x"));
}

#[test]
fn should_keep_a_re_added_element_after_an_old_remove_arrives_again() {
    let a = sut!(replica("a"));
    let b = replica("b").unwrap();
    a.add("s", "x");
    a.remove("s", "x");
    let after_remove = a.snapshot();
    a.add("s", "x");
    b.merge(&a.snapshot());

    b.merge(&after_remove);

    assert!(b.contains("s", "x"));
}

#[test]
fn should_keep_the_highest_timestamp_in_a_register() {
    let a = sut!(replica("a"));

    assert_eq!(a.read("title"), None);
    a.write("title", "v1", 10);
    a.write("title", "v0", 5);
    assert_eq!(a.read("title").as_deref(), Some("v1"));
    a.write("title", "v2", 20);
    assert_eq!(a.read("title").as_deref(), Some("v2"));
}

#[test]
fn should_converge_registers_whatever_the_merge_order() {
    let a = sut!(replica("a"));
    let b = replica("b").unwrap();
    let c = replica("c").unwrap();
    a.write("r", "from-a", 5);
    b.write("r", "from-b", 9);
    c.write("r", "from-c", 7);

    a.merge(&b.snapshot());
    a.merge(&c.snapshot());
    c.merge(&a.snapshot());
    b.merge(&c.snapshot());
    b.merge(&a.snapshot());

    for r in [&a, &b, &c] {
        assert_eq!(r.read("r").as_deref(), Some("from-b"));
    }
}

#[test]
fn should_break_timestamp_ties_by_the_larger_replica_id() {
    let a = sut!(replica("a"));
    let b = replica("b").unwrap();
    a.write("r", "from-a", 10);
    b.write("r", "from-b", 10);

    sync(&a, &b);

    assert_eq!(a.read("r").as_deref(), Some("from-b"));
    assert_eq!(b.read("r").as_deref(), Some("from-b"));
}

#[test]
fn should_keep_the_three_types_in_separate_namespaces() {
    let a = sut!(replica("a"));
    let b = replica("b").unwrap();

    a.increment("x", 3);
    a.add("x", "e");
    a.write("x", "v", 1);
    sync(&a, &b);

    assert_eq!(b.counter("x"), 3);
    assert_eq!(sorted(b.elements("x")), ["e"]);
    assert_eq!(b.read("x").as_deref(), Some("v"));
    assert_eq!(b.counter("e"), 0);
    assert!(!b.contains("v", "e"));
}

#[test]
fn should_handle_awkward_names_and_values() {
    let a = sut!(replica("a"));
    let b = replica("b").unwrap();
    let weird = "line1\nline2 | with ;,'\" and ünïcode ✓";

    a.add(weird, weird);
    a.write(weird, weird, 1);
    a.increment(weird, 2);
    b.merge(&a.snapshot());

    assert!(b.contains(weird, weird));
    assert_eq!(b.read(weird).as_deref(), Some(weird));
    assert_eq!(b.counter(weird), 2);
}

#[test]
fn should_merge_an_empty_replica_and_itself() {
    let a = sut!(replica("a"));
    let empty = replica("e").unwrap();
    a.increment("c", 2);
    a.add("s", "x");

    a.merge(&empty.snapshot());
    a.merge(&a.snapshot());
    empty.merge(&a.snapshot());

    assert_eq!(a.counter("c"), 2);
    assert_eq!(empty.counter("c"), 2);
    assert_eq!(sorted(empty.elements("s")), ["x"]);
}

#[test]
fn should_not_lose_increments_from_many_threads_on_one_replica() {
    let a = sut!(replica("a"));

    run_concurrently(8, |t| {
        for i in 0..1_000 {
            a.increment("c", 1);
            a.add("s", &format!("t{t}"));
            a.write("r", &format!("v{t}"), i);
        }
    });

    assert_eq!(a.counter("c"), 8_000);
    assert_eq!(a.elements("s").len(), 8);
    assert!(a.read("r").is_some());
}

#[test]
fn should_converge_after_random_operations_and_random_gossip() {
    if replica("probe").is_none() {
        return playground_support::skipped();
    }
    for seed in 0..100u64 {
        let mut rng = Rng::new(seed + 1);
        let rs: Vec<Arc<dyn Replica>> = ["a", "b", "c"]
            .iter()
            .map(|id| replica(id).unwrap())
            .collect();
        let mut expected: i64 = 0;
        let mut in_flight: Vec<String> = Vec::new();
        for step in 0..150u64 {
            let r = &rs[rng.below(3) as usize];
            match rng.below(8) {
                0 | 1 => {
                    let n = rng.below(5);
                    r.increment("c", n);
                    expected += n as i64;
                }
                2 => {
                    let n = rng.below(5);
                    r.decrement("c", n);
                    expected -= n as i64;
                }
                3 | 4 => r.add("s", &format!("e{}", rng.below(4))),
                5 => r.remove("s", &format!("e{}", rng.below(4))),
                6 => r.write(
                    "r",
                    &format!("v{}", rng.below(100)),
                    rng.below(50) * 1000 + step,
                ),
                _ => in_flight.push(r.snapshot()),
            }
            if !in_flight.is_empty() && rng.below(3) == 0 {
                let s = in_flight[rng.below(in_flight.len() as u64) as usize].clone();
                rs[rng.below(3) as usize].merge(&s); // old, duplicated, reordered deliveries
            }
        }
        sync(&rs[0], &rs[1]);
        sync(&rs[1], &rs[2]);
        sync(&rs[0], &rs[2]);

        for r in &rs {
            assert_eq!(r.counter("c"), expected, "seed {seed} counter");
            assert_eq!(
                sorted(r.elements("s")),
                sorted(rs[0].elements("s")),
                "seed {seed} set"
            );
            assert_eq!(r.read("r"), rs[0].read("r"), "seed {seed} register");
        }
    }
}
