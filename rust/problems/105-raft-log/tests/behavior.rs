#![allow(unused_imports, clippy::needless_range_loop)]
use p105_raft_log::{contract::*, create};
use playground_support::{Rng, SimNet};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct Watch {
    leader_of_term: HashMap<u64, String>,
    last_committed: HashMap<String, Vec<String>>,
    longest: Vec<String>,
    violations: Vec<String>,
}

/// A cluster in the simulation with the Raft safety invariants checked after every simulated millisecond.
struct Cluster {
    net: SimNet,
    ids: Vec<String>,
    nodes: HashMap<String, Arc<dyn RaftNode>>,
    watch: Arc<Mutex<Watch>>,
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
                seed.wrapping_mul(31).wrapping_add(i as u64 * 17 + 3),
            )?;
            let (n1, n2) = (node.clone(), node.clone());
            net.register(
                id,
                Arc::new(move |f, p| n1.on_message(f, p)),
                Arc::new(move |t| n2.on_tick(t)),
            );
            nodes.insert(id.clone(), node);
        }
        let watch = Arc::new(Mutex::new(Watch::default()));
        let (w, ns, is) = (watch.clone(), nodes.clone(), ids.clone());
        net.observe(Arc::new(move || {
            let mut w = w.lock().unwrap();
            for id in &is {
                let n = &ns[id];
                if n.role() == Role::Leader {
                    let prev = w
                        .leader_of_term
                        .entry(n.term())
                        .or_insert_with(|| id.clone())
                        .clone();
                    if prev != *id {
                        w.violations
                            .push(format!("two leaders in term {}: {prev}, {id}", n.term()));
                    }
                }
                let now = n.committed();
                let before = w.last_committed.get(id).cloned().unwrap_or_default();
                if now.len() < before.len() || now[..before.len()] != before[..] {
                    w.violations.push(format!(
                        "{id} changed its committed entries: {before:?} -> {now:?}"
                    ));
                }
                w.last_committed.insert(id.clone(), now.clone());
                let common = now.len().min(w.longest.len());
                if now[..common] != w.longest[..common] {
                    let longest = w.longest.clone();
                    w.violations.push(format!(
                        "{id} committed {now:?} but another node committed {longest:?}"
                    ));
                }
                if now.len() > w.longest.len() {
                    w.longest = now;
                }
            }
        }));
        Some(Cluster {
            net,
            ids,
            nodes,
            watch,
        })
    }

    fn violations(&self) -> Vec<String> {
        self.watch.lock().unwrap().violations.clone()
    }

    fn leader(&self) -> Option<String> {
        let l: Vec<&String> = self
            .ids
            .iter()
            .filter(|i| !self.net.is_crashed(i) && self.nodes[*i].role() == Role::Leader)
            .collect();
        if l.len() == 1 {
            Some(l[0].clone())
        } else {
            None
        }
    }

    fn await_leader(&self, max_ms: u64) -> String {
        self.net.run_until(&|| self.leader().is_some(), max_ms);
        self.leader()
            .unwrap_or_else(|| panic!("no single leader after {max_ms} ms"))
    }

    fn propose(&self, command: &str) -> u64 {
        let l = self.await_leader(2_000);
        self.nodes[&l]
            .propose(command)
            .expect("the leader accepts proposals")
    }

    fn everyone_committed(&self, expected: &[&str]) -> bool {
        self.ids
            .iter()
            .filter(|i| !self.net.is_crashed(i))
            .all(|i| self.nodes[i].committed() == expected)
    }

    fn committed_on(&self, id: &str) -> Vec<String> {
        self.nodes[id].committed()
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
fn should_elect_one_leader_and_everyone_knows_it() {
    let c = cluster!(3, 1);

    let leader = c.await_leader(1_000);
    c.net.run(100);

    for id in &c.ids {
        assert_eq!(
            c.nodes[id].leader().as_deref(),
            Some(leader.as_str()),
            "{id}"
        );
    }
    assert!(c.violations().is_empty());
}

#[test]
fn should_refuse_to_propose_on_a_follower() {
    let c = cluster!(3, 2);
    let leader = c.await_leader(1_000);
    let follower = c.ids.iter().find(|i| **i != leader).unwrap();

    assert_eq!(c.nodes[follower].propose("x"), None);
    assert!(c.nodes[&leader].propose("x").is_some());
}

#[test]
fn should_replicate_and_commit_commands_in_order_on_every_node() {
    let c = cluster!(3, 3);
    c.await_leader(1_000);

    c.propose("a");
    c.propose("b");
    c.propose("c");

    assert!(c
        .net
        .run_until(&|| c.everyone_committed(&["a", "b", "c"]), 1_000));
    assert!(c.violations().is_empty());
}

#[test]
fn should_return_increasing_indexes_and_hide_the_no_op() {
    let c = cluster!(3, 4);
    let first = c.propose("a");
    let second = c.propose("b");

    assert!(first >= 2, "the leader's no-op took index 1");
    assert_eq!(second, first + 1);
    c.net.run(300);
    assert_eq!(c.committed_on(&c.await_leader(500)), ["a", "b"]);
}

#[test]
fn should_not_commit_without_a_majority() {
    let c = cluster!(3, 5);
    let leader = c.await_leader(1_000);
    c.net.run(100);
    for id in &c.ids {
        if *id != leader {
            c.net.crash(id);
        }
    }

    assert!(c.nodes[&leader].propose("lonely").is_some());
    c.net.run(700);

    assert!(c.committed_on(&leader).is_empty());
    assert_ne!(c.nodes[&leader].role(), Role::Leader, "check-quorum");
}

#[test]
fn should_keep_committed_entries_across_a_leader_failure() {
    let c = cluster!(3, 6);
    let old = c.await_leader(1_000);
    c.propose("a");
    c.propose("b");
    c.net
        .run_until(&|| c.everyone_committed(&["a", "b"]), 1_000);

    c.net.crash(&old);
    let next = c.await_leader(2_000);
    c.propose("c");

    assert_ne!(next, old);
    assert!(c
        .net
        .run_until(&|| c.everyone_committed(&["a", "b", "c"]), 1_000));

    c.net.recover(&old);
    assert!(c
        .net
        .run_until(&|| c.everyone_committed(&["a", "b", "c"]), 2_000));
    assert_eq!(c.nodes[&old].role(), Role::Follower);
    assert!(c.violations().is_empty());
}

#[test]
fn should_discard_the_uncommitted_entries_of_a_deposed_leader() {
    let c = cluster!(3, 7);
    let old = c.await_leader(1_000);
    c.propose("kept");
    c.net.run_until(&|| c.everyone_committed(&["kept"]), 1_000);
    let others: Vec<String> = c.ids.iter().filter(|i| **i != old).cloned().collect();

    c.net
        .partition(&[&[old.as_str()], &[others[0].as_str(), others[1].as_str()]]);
    assert!(c.nodes[&old].propose("doomed").is_some());
    c.net.run_until(
        &|| others.iter().any(|i| c.nodes[i].role() == Role::Leader),
        2_000,
    );
    let fresh = others
        .iter()
        .find(|i| c.nodes[*i].role() == Role::Leader)
        .unwrap();
    c.nodes[fresh].propose("winner");
    c.net.run(300);
    c.net.heal();

    assert!(c
        .net
        .run_until(&|| c.everyone_committed(&["kept", "winner"]), 3_000));
    assert!(!c.committed_on(&old).contains(&"doomed".to_string()));
    assert!(c.violations().is_empty());
}

#[test]
fn should_catch_up_a_follower_that_missed_many_entries() {
    let c = cluster!(3, 8);
    let leader = c.await_leader(1_000);
    let slow = c.ids.iter().find(|i| **i != leader).unwrap().clone();
    c.net.crash(&slow);
    let mut expected = Vec::new();
    for i in 0..30 {
        c.propose(&format!("c{i}"));
        expected.push(format!("c{i}"));
        c.net.run(20);
    }
    c.net.recover(&slow);

    let refs: Vec<&str> = expected.iter().map(|s| s.as_str()).collect();
    assert!(c.net.run_until(&|| c.everyone_committed(&refs), 3_000));
}

#[test]
fn should_never_elect_a_node_with_an_outdated_log() {
    let c = cluster!(3, 9);
    let leader = c.await_leader(1_000);
    let others: Vec<String> = c.ids.iter().filter(|i| **i != leader).cloned().collect();
    let (stale, fresh) = (others[0].clone(), others[1].clone());
    c.net.crash(&stale);
    c.propose("a");
    c.propose("b");
    c.net.run(50);
    c.net
        .run_until(&|| c.nodes[&fresh].committed() == ["a", "b"], 1_000);
    c.net.crash(&leader);
    c.net.run(20); // let the heartbeats that were already on the wire arrive (or be lost) while the stale node is still down
    c.net.recover(&stale);

    c.net.run(3_000);

    assert_ne!(c.nodes[&stale].role(), Role::Leader);
    assert_eq!(c.committed_on(&fresh), ["a", "b"]);
    assert!(c.violations().is_empty());
}

#[test]
fn should_stay_consistent_with_duplicated_and_reordered_messages() {
    let c = cluster!(5, 10);
    c.net.set_duplicate_rate(0.3);
    c.net.set_latency(1, 40);
    let mut accepted = Vec::new();
    for i in 0..20 {
        let l = c.await_leader(3_000);
        if c.nodes[&l].propose(&format!("c{i}")).is_some() {
            accepted.push(format!("c{i}"));
        }
        c.net.run(30);
    }

    c.net.run(2_000);

    assert!(c.violations().is_empty());
    assert!(!accepted.is_empty());
    let l = c.await_leader(1_000);
    assert!(!c.committed_on(&l).is_empty());
}

#[test]
fn should_commit_everything_when_the_network_loses_messages() {
    let c = cluster!(3, 11);
    c.net.set_drop_rate(0.25);
    c.await_leader(3_000);
    let mut expected = Vec::new();
    for i in 0..10 {
        c.propose(&format!("c{i}"));
        expected.push(format!("c{i}"));
        c.net.run(200);
    }

    let refs: Vec<&str> = expected.iter().map(|s| s.as_str()).collect();
    assert!(c.net.run_until(&|| c.everyone_committed(&refs), 5_000));
    assert!(c.violations().is_empty());
}

#[test]
fn should_never_break_safety_under_random_crashes_and_partitions() {
    if Cluster::new(3, 0).is_none() {
        return playground_support::skipped();
    }
    for seed in 0..100u64 {
        let c = Cluster::new(5, seed).unwrap();
        let mut rng = Rng::new(seed * 17 + 3);
        c.net.set_drop_rate(0.05);
        c.net.set_duplicate_rate(0.2);
        c.net.set_latency(1, 30);
        let mut proposed = 0;
        for _ in 0..25 {
            match rng.below(5) {
                0 => c.net.crash(&c.ids[rng.below(5) as usize]),
                1 => c.ids.iter().for_each(|i| c.net.recover(i)),
                2 => c.net.partition(&[&["n0", "n1"], &["n2", "n3", "n4"]]),
                3 => c.net.heal(),
                _ => {}
            }
            for id in &c.ids {
                if !c.net.is_crashed(id)
                    && c.nodes[id]
                        .propose(&format!("s{seed}-{proposed}"))
                        .is_some()
                {
                    proposed += 1;
                }
            }
            c.net.run(50 + rng.below(250));
        }
        assert!(
            c.violations().is_empty(),
            "seed {seed}: {:?}",
            c.violations()
        );

        c.net.heal();
        c.ids.iter().for_each(|i| c.net.recover(i));
        let mut committed = false;
        for attempt in 0..5 {
            // a proposal can be lost if the leader is deposed right away: that is legal Raft, so ask again
            let cmd = format!("final{attempt}");
            c.nodes[&c.await_leader(3_000)].propose(&cmd);
            committed = c.net.run_until(
                &|| c.ids.iter().all(|i| c.nodes[i].committed().contains(&cmd)),
                1_500,
            );
            if committed {
                break;
            }
        }
        assert!(committed, "seed {seed}: the healed cluster must commit");
        let first = c.committed_on(&c.ids[0]);
        assert!(
            c.ids.iter().all(|i| c.committed_on(i) == first),
            "seed {seed}: all logs equal"
        );
        assert!(c.violations().is_empty(), "seed {seed}");
    }
}
