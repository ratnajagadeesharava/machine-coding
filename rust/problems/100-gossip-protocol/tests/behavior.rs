#![allow(unused_imports, clippy::needless_range_loop)]
use p100_gossip_protocol::{contract::*, create};
use playground_support::{Rng, SimNet};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

struct Cluster {
    net: SimNet,
    ids: Vec<String>,
    nodes: HashMap<String, Arc<dyn GossipNode>>,
}

impl Cluster {
    fn new(size: usize, seed: u64) -> Option<Cluster> {
        let net = SimNet::new(seed);
        let ids: Vec<String> = (0..size).map(|i| format!("n{i}")).collect();
        let mut nodes = HashMap::new();
        for (i, id) in ids.iter().enumerate() {
            let peers: Vec<&str> = ids
                .iter()
                .filter(|p| *p != id)
                .map(|s| s.as_str())
                .collect();
            let node = create(
                id,
                &peers,
                net.sender(id),
                seed.wrapping_mul(131).wrapping_add(i as u64 * 17 + 3),
            )?;
            let (n1, n2) = (node.clone(), node.clone());
            net.register(
                id,
                Arc::new(move |f, p| n1.on_message(f, p)),
                Arc::new(move |t| n2.on_tick(t)),
            );
            nodes.insert(id.clone(), node);
        }
        Some(Cluster { net, ids, nodes })
    }

    fn everyone_has(&self, key: &str, value: &str) -> bool {
        self.ids
            .iter()
            .all(|id| self.net.is_crashed(id) || self.nodes[id].get(key).as_deref() == Some(value))
    }

    fn agree_on(&self, key: &str) -> bool {
        let mut first: Option<String> = None;
        for id in &self.ids {
            if self.net.is_crashed(id) {
                continue;
            }
            let Some(v) = self.nodes[id].get(key) else {
                return false;
            };
            match &first {
                Some(f) if *f != v => return false,
                _ => first = Some(v),
            }
        }
        true
    }

    fn alive(&self, id: &str) -> Vec<String> {
        self.nodes[id].alive_members()
    }
}

macro_rules! cluster {
    ($size:expr, $seed:expr) => {
        match Cluster::new($size, $seed) {
            Some(c) => c,
            None => return playground_support::skipped(),
        }
    };
}

#[test]
fn should_spread_a_write_to_every_node_in_logarithmically_many_rounds() {
    let c = cluster!(16, 1);
    c.nodes["n0"].put("color", "blue");

    let spread = c.net.run_until(&|| c.everyone_has("color", "blue"), 1_500);

    assert!(
        spread,
        "16 nodes, fanout 2, 100 ms rounds: well within 15 rounds"
    );
}

#[test]
fn should_spread_many_keys_from_many_writers() {
    let c = cluster!(12, 2);
    for i in 0..12 {
        c.nodes[&format!("n{i}")].put(&format!("key-{i}"), &format!("value-{i}"));
    }

    c.net.run(2_500);

    for i in 0..12 {
        assert!(
            c.everyone_has(&format!("key-{i}"), &format!("value-{i}")),
            "key-{i}"
        );
    }
}

#[test]
fn should_let_a_later_write_overwrite_an_earlier_one_everywhere() {
    let c = cluster!(10, 3);
    c.nodes["n0"].put("k", "first");
    c.net.run_until(&|| c.everyone_has("k", "first"), 2_000);

    c.nodes["n7"].put("k", "second");
    let spread = c.net.run_until(&|| c.everyone_has("k", "second"), 2_000);

    assert!(spread);
    c.net.run(1_000);
    assert!(
        c.everyone_has("k", "second"),
        "the old value must never come back"
    );
}

#[test]
fn should_resolve_concurrent_writes_to_the_same_value_everywhere() {
    let c = cluster!(8, 4);
    c.nodes["n2"].put("k", "from-n2");
    c.nodes["n5"].put("k", "from-n5"); // same version: the larger writer id wins

    c.net.run(2_000);

    assert!(c.agree_on("k"));
    assert_eq!(c.nodes["n0"].get("k"), Some("from-n5".to_string()));
}

#[test]
fn should_still_converge_with_thirty_percent_message_loss() {
    let c = cluster!(16, 5);
    c.net.set_drop_rate(0.3);
    c.nodes["n3"].put("k", "v");

    let spread = c.net.run_until(&|| c.everyone_has("k", "v"), 5_000);

    assert!(spread);
}

#[test]
fn should_converge_with_duplicated_and_reordered_messages() {
    let c = cluster!(10, 6);
    c.net.set_duplicate_rate(0.5);
    c.net.set_latency(1, 120);
    c.nodes["n1"].put("k", "a");
    c.nodes["n8"].put("k2", "b");

    c.net.run(4_000);

    assert!(c.everyone_has("k", "a"));
    assert!(c.everyone_has("k2", "b"));
}

#[test]
fn should_not_suspect_anyone_in_a_steady_healthy_cluster() {
    let c = cluster!(16, 7);

    c.net.run(10_000);

    for id in &c.ids {
        assert_eq!(c.alive(id).len(), 16, "alive set of {id}");
    }
}

#[test]
fn should_not_suspect_anyone_when_a_fifth_of_the_messages_are_lost() {
    let c = cluster!(16, 8);
    c.net.set_drop_rate(0.2);

    c.net.run(10_000);

    for id in &c.ids {
        assert_eq!(c.alive(id).len(), 16, "alive set of {id}");
    }
}

#[test]
fn should_detect_a_crashed_node_everywhere_after_the_timeout() {
    let c = cluster!(16, 9);
    c.net.run(1_000);

    c.net.crash("n3");
    c.net.run(1_500);
    for id in c.ids.iter().filter(|i| *i != "n3") {
        assert!(
            c.alive(id).contains(&"n3".to_string()),
            "{id} should not suspect n3 yet (under 2 s)"
        );
    }
    c.net.run(2_000);

    for id in c.ids.iter().filter(|i| *i != "n3") {
        let alive = c.alive(id);
        assert!(
            !alive.contains(&"n3".to_string()) && alive.len() == 15,
            "{id} must have noticed"
        );
    }
}

#[test]
fn should_readmit_a_recovered_node_and_catch_it_up_on_missed_writes() {
    let c = cluster!(12, 10);
    c.net.run(500);
    c.net.crash("n4");
    c.nodes["n0"].put("missed", "while-away");
    c.net.run(4_000);

    c.net.recover("n4");
    let caught_up = c.net.run_until(
        &|| c.nodes["n4"].get("missed").as_deref() == Some("while-away"),
        2_000,
    );
    c.net.run(1_500);

    assert!(caught_up);
    for id in &c.ids {
        assert_eq!(c.alive(id).len(), 12, "alive set of {id}");
    }
}

#[test]
fn should_split_into_two_views_during_a_partition_and_merge_after_it_heals() {
    let c = cluster!(8, 11);
    c.net.run(500);
    let left = ["n0", "n1", "n2", "n3"];
    let right = ["n4", "n5", "n6", "n7"];

    c.net.partition(&[&left, &right]);
    c.nodes["n0"].put("left-key", "L");
    c.nodes["n7"].put("right-key", "R");
    c.net.run(4_000);

    assert_eq!(c.nodes["n1"].get("right-key"), None);
    assert_eq!(
        c.alive("n1"),
        left.iter().map(|s| s.to_string()).collect::<Vec<_>>()
    );
    assert_eq!(
        c.alive("n6"),
        right.iter().map(|s| s.to_string()).collect::<Vec<_>>()
    );

    c.net.heal();
    c.net.run(3_000);

    assert!(c.everyone_has("left-key", "L"));
    assert!(c.everyone_has("right-key", "R"));
    for id in &c.ids {
        assert_eq!(c.alive(id).len(), 8);
    }
}

#[test]
fn should_gossip_with_a_few_peers_per_round_instead_of_broadcasting() {
    let c = cluster!(32, 12);
    c.nodes["n0"].put("k", "v");

    c.net.run(3_000); // 30 rounds

    let max = c.ids.iter().map(|id| c.net.sent_by(id)).max().unwrap();
    assert!(
        max < 250,
        "a broadcast would be 31 messages per round (about 930), saw {max}"
    );
    assert!(c.everyone_has("k", "v"));
}

#[test]
fn should_replay_the_same_run_from_the_same_seed() {
    let mut runs = Vec::new();
    for _ in 0..2 {
        let c = cluster!(10, 13);
        c.net.set_drop_rate(0.2);
        c.nodes["n0"].put("k", "v");
        c.net.run(1_000);
        let summary: Vec<String> = c
            .ids
            .iter()
            .map(|id| {
                format!(
                    "{}{}",
                    c.nodes[id].get("k").is_some() as u8,
                    c.net.sent_by(id)
                )
            })
            .collect();
        runs.push(summary.join(","));
    }

    assert_eq!(runs[0], runs[1]);
}

#[test]
fn should_converge_over_100_random_schedules_of_writes_and_losses() {
    for seed in 0..100u64 {
        let c = cluster!(8, 500 + seed);
        let mut r = Rng::new(seed);
        c.net.set_drop_rate(0.1 + r.below(3) as f64 * 0.1);
        let mut written = HashSet::new();
        for step in 0..20 {
            let key = format!("k{}", r.below(5));
            let value = format!("v{step}");
            c.nodes[&format!("n{}", r.below(8))].put(&key, &value);
            written.insert(value);
            c.net.run(50 + r.below(150));
        }
        c.net.set_drop_rate(0.0);

        c.net.run(3_000);

        for k in 0..5 {
            let key = format!("k{k}");
            if let Some(v) = c.nodes["n0"].get(&key) {
                assert!(c.agree_on(&key), "seed {seed} key {key}");
                assert!(written.contains(&v));
            }
        }
    }
}
