#![allow(unused_imports, clippy::needless_range_loop)]
use p96_consistent_hashing::{contract::*, create};
use playground_support::{run_concurrently, sut, Rng};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const KEYS: usize = 50_000;

fn ring(vnodes: usize, nodes: usize) -> Option<Arc<dyn HashRing>> {
    let r = create(vnodes)?;
    for i in 0..nodes {
        r.add_node(&format!("node-{i}"), 1).unwrap();
    }
    Some(r)
}

fn ownership(r: &Arc<dyn HashRing>) -> HashMap<String, String> {
    (0..KEYS)
        .map(|k| (format!("key-{k}"), r.node_for(&format!("key-{k}")).unwrap()))
        .collect()
}

fn counts(o: &HashMap<String, String>) -> HashMap<String, usize> {
    let mut c = HashMap::new();
    for n in o.values() {
        *c.entry(n.clone()).or_insert(0) += 1;
    }
    c
}

#[test]
fn should_refuse_lookups_on_an_empty_ring() {
    let r = sut!(ring(100, 0));

    assert_eq!(r.node_for("k"), None);
    assert!(r.nodes_for("k", 3).is_empty());
}

#[test]
fn should_send_everything_to_the_only_node() {
    let r = sut!(ring(100, 1));

    let c = counts(&ownership(&r));

    assert_eq!(c.len(), 1);
    assert!(c.contains_key("node-0"));
}

#[test]
fn should_reject_duplicates_and_bad_nodes() {
    let r = sut!(ring(100, 2));

    assert_eq!(r.add_node("node-0", 1), Err(RingError::InvalidNode));
    assert_eq!(r.add_node(" ", 1), Err(RingError::InvalidNode));
    assert_eq!(r.add_node("x", 0), Err(RingError::InvalidNode));
    assert!(!r.remove_node("nope"));
}

#[test]
fn should_map_a_key_to_the_same_node_every_time_and_regardless_of_insertion_order() {
    let a = sut!(ring(100, 0));
    let b = sut!(ring(100, 0));
    let mut names: Vec<String> = (0..8).map(|i| format!("node-{i}")).collect();
    for n in &names {
        a.add_node(n, 1).unwrap();
    }
    let mut rng = Rng::new(3);
    for i in (1..names.len()).rev() {
        names.swap(i, rng.below(i as u64 + 1) as usize);
    }
    for n in &names {
        b.add_node(n, 1).unwrap();
    }

    for k in 0..5_000 {
        let key = format!("key-{k}");
        assert_eq!(a.node_for(&key), a.node_for(&key));
        assert_eq!(a.node_for(&key), b.node_for(&key));
    }
}

#[test]
fn should_spread_keys_evenly() {
    let r = sut!(ring(200, 10));

    let c = counts(&ownership(&r));

    assert_eq!(c.len(), 10);
    let mean = KEYS as f64 / 10.0;
    for v in c.values() {
        assert!(
            (mean * 0.65..=mean * 1.35).contains(&(*v as f64)),
            "share {v} too far from {mean}"
        );
    }
}

#[test]
fn should_only_move_keys_to_the_new_node_when_one_joins() {
    let r = sut!(ring(200, 10));
    let before = ownership(&r);

    r.add_node("newcomer", 1).unwrap();
    let after = ownership(&r);

    let mut moved = 0;
    for (key, old) in &before {
        if old != &after[key] {
            moved += 1;
            assert_eq!(after[key], "newcomer", "{key} moved between two old nodes");
        }
    }
    let frac = moved as f64 / KEYS as f64;
    assert!(
        (0.04..=0.15).contains(&frac),
        "about 1/11 of the keys should move, got {frac}"
    );
}

#[test]
fn should_only_move_the_keys_of_the_node_that_leaves() {
    let r = sut!(ring(200, 10));
    let before = ownership(&r);

    assert!(r.remove_node("node-3"));
    let after = ownership(&r);

    for (key, old) in &before {
        if old == "node-3" {
            assert_ne!(after[key], "node-3");
        } else {
            assert_eq!(&after[key], old, "{key} must not move");
        }
    }
    let nodes = r.nodes();
    assert_eq!(nodes.len(), 9);
    assert!(!nodes.contains(&"node-3".to_string()));
}

#[test]
fn should_return_distinct_replica_nodes_starting_with_the_primary() {
    let r = sut!(ring(100, 6));

    for k in 0..2_000 {
        let key = format!("key-{k}");
        let replicas = r.nodes_for(&key, 3);
        assert_eq!(replicas.len(), 3);
        assert_eq!(replicas.iter().collect::<HashSet<_>>().len(), 3);
        assert_eq!(Some(replicas[0].clone()), r.node_for(&key));
    }
    let all = r.nodes_for("key-1", 10);
    assert_eq!(all.len(), 6);
    assert_eq!(all.iter().collect::<HashSet<_>>().len(), 6);
    assert_eq!(r.nodes_for("key-1", 1), vec![r.node_for("key-1").unwrap()]);
    assert!(r.nodes_for("key-1", 0).is_empty());
}

#[test]
fn should_promote_the_next_replica_when_the_primary_leaves() {
    let r = sut!(ring(100, 6));
    let before: Vec<(String, Vec<String>)> = (0..2_000)
        .map(|k| (format!("key-{k}"), r.nodes_for(&format!("key-{k}"), 3)))
        .collect();

    r.remove_node("node-2");

    for (key, replicas) in before {
        if replicas[0] == "node-2" {
            assert_eq!(r.node_for(&key), Some(replicas[1].clone()));
        }
    }
}

#[test]
fn should_give_heavier_nodes_proportionally_more_keys() {
    let r = sut!(ring(200, 0));
    r.add_node("small-1", 1).unwrap();
    r.add_node("small-2", 1).unwrap();
    r.add_node("big", 3).unwrap();

    let c = counts(&ownership(&r));

    let expected_big = KEYS as f64 * 3.0 / 5.0;
    assert!((expected_big * 0.8..=expected_big * 1.2).contains(&(c["big"] as f64)));
    assert!((KEYS as f64 / 5.0 * 0.7..=KEYS as f64 / 5.0 * 1.3).contains(&(c["small-1"] as f64)));
}

#[test]
fn should_answer_lookups_while_nodes_join_and_leave() {
    let r = sut!(ring(100, 5));
    let ever_seen: HashSet<String> = (0..5)
        .map(|i| format!("node-{i}"))
        .chain((0..20).map(|i| format!("dyn-{i}")))
        .collect();
    let bad = AtomicUsize::new(0);

    run_concurrently(8, |t| {
        if t == 0 {
            for i in 0..20 {
                r.add_node(&format!("dyn-{i}"), 1).unwrap();
                r.remove_node(&format!("dyn-{}", i / 2));
            }
            return;
        }
        for i in 0..20_000 {
            let key = format!("key-{i}");
            let owner = r.node_for(&key);
            if owner.map(|o| !ever_seen.contains(&o)).unwrap_or(true)
                || r.nodes_for(&key, 3).is_empty()
            {
                bad.fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    assert_eq!(bad.load(Ordering::SeqCst), 0);
    let nodes = r.nodes();
    assert!(nodes.contains(&"node-0".to_string()) && nodes.contains(&"node-4".to_string()));
}

#[test]
fn should_stay_correct_over_100_runs() {
    for run in 0..100 {
        let r = sut!(ring(50, 4));

        run_concurrently(4, |t| {
            r.add_node(&format!("extra-{t}"), 1).unwrap();
            for i in 0..100 {
                r.node_for(&format!("k{i}")).unwrap();
            }
        });

        assert_eq!(r.nodes().len(), 8, "run {run}");
        let all = r.nodes_for("k", 8);
        assert_eq!(all.iter().collect::<HashSet<_>>().len(), 8, "run {run}");
    }
}
