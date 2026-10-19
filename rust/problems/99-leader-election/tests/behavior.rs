#![allow(unused_imports, clippy::needless_range_loop)]
use p99_leader_election::{contract::*, create};
use playground_support::{Rng, SimNet};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// A cluster inside the simulation, with the election-safety invariant checked after every simulated millisecond.
struct Cluster {
    net: SimNet,
    ids: Vec<String>,
    nodes: HashMap<String, Arc<dyn ElectionNode>>,
    violations: Arc<Mutex<Vec<String>>>,
}

impl Cluster {
    fn new(size: usize, seed: u64) -> Option<Cluster> {
        let net = SimNet::new(seed);
        let ids: Vec<String> = (0..size).map(|i| format!("n{i}")).collect();
        let mut nodes = HashMap::new();
        for id in &ids {
            let peers: Vec<&str> = ids.iter().filter(|p| *p != id).map(|s| s.as_str()).collect();
            let node = create(id, &peers, net.sender(id), seed.wrapping_mul(31).wrapping_add(id.len() as u64 * 7 + id.as_bytes()[1] as u64))?;
            let (n1, n2) = (node.clone(), node.clone());
            net.register(id, Arc::new(move |from, payload| n1.on_message(from, payload)), Arc::new(move |now| n2.on_tick(now)));
            nodes.insert(id.clone(), node);
        }
        let violations = Arc::new(Mutex::new(Vec::new()));
        let leader_of_term: Arc<Mutex<HashMap<u64, String>>> = Arc::new(Mutex::new(HashMap::new()));
        {
            let (nodes, violations, leader_of_term, net2) = (nodes.clone(), violations.clone(), leader_of_term, net.clone());
            net.observe(Arc::new(move || {
                for (id, n) in &nodes {
                    if n.role() == Role::Leader {
                        let mut m = leader_of_term.lock().unwrap();
                        let prev = m.entry(n.term()).or_insert_with(|| id.clone()).clone();
                        if &prev != id {
                            violations.lock().unwrap().push(format!("two leaders in term {}: {prev} and {id} at t={}", n.term(), net2.now()));
                        }
                    }
                }
            }));
        }
        Some(Cluster { net, ids, nodes, violations })
    }

    fn leaders(&self) -> Vec<String> {
        self.ids.iter().filter(|id| !self.net.is_crashed(id) && self.nodes[*id].role() == Role::Leader).cloned().collect()
    }

    fn has_single_leader_among(&self, members: &[String]) -> bool {
        members.iter().filter(|id| self.nodes[*id].role() == Role::Leader).count() == 1
    }

    fn all_agree(&self) -> bool {
        let mut seen: Option<(String, u64)> = None;
        for id in &self.ids {
            let n = &self.nodes[id];
            let Some(l) = n.leader() else { return false };
            match &seen {
                None => seen = Some((l, n.term())),
                Some((sl, st)) if *sl != l || *st != n.term() => return false,
                _ => {}
            }
        }
        true
    }

    fn assert_safe(&self, context: &str) {
        let v = self.violations.lock().unwrap();
        assert!(v.is_empty(), "{context}: {:?}", *v);
    }
}

#[test]
fn should_elect_exactly_one_leader_in_clusters_of_three_five_and_seven() {
    for size in [3usize, 5, 7] {
        let Some(c) = Cluster::new(size, 100 + size as u64) else { return playground_support::skipped() };

        let elected = c.net.run_until(&|| c.leaders().len() == 1, 1_500);
        c.net.run(500);

        assert!(elected, "{size} nodes elect a leader");
        assert_eq!(c.leaders().len(), 1);
        assert!(c.all_agree(), "{size} nodes agree on leader and term");
        c.assert_safe("election");
    }
}

#[test]
fn should_elect_itself_when_alone_in_the_cluster() {
    let Some(c) = Cluster::new(1, 1) else { return playground_support::skipped() };

    let elected = c.net.run_until(&|| c.leaders().len() == 1, 600);

    assert!(elected);
    assert_eq!(c.nodes["n0"].leader(), Some("n0".to_string()));
}

#[test]
fn should_keep_the_same_leader_and_term_while_nothing_fails() {
    let Some(c) = Cluster::new(5, 7) else { return playground_support::skipped() };
    c.net.run_until(&|| c.leaders().len() == 1, 1_500);
    c.net.run(500);
    let leader = c.leaders()[0].clone();
    let term = c.nodes[&leader].term();

    c.net.run(10_000);

    assert_eq!(c.leaders(), vec![leader.clone()]);
    assert_eq!(c.nodes[&leader].term(), term, "no unnecessary elections");
    c.assert_safe("stable run");
}

#[test]
fn should_elect_a_new_leader_with_a_higher_term_when_the_leader_crashes() {
    let Some(c) = Cluster::new(5, 11) else { return playground_support::skipped() };
    c.net.run_until(&|| c.leaders().len() == 1, 1_500);
    let old_leader = c.leaders()[0].clone();
    let old_term = c.nodes[&old_leader].term();

    c.net.crash(&old_leader);
    let replaced = c.net.run_until(&|| c.leaders().len() == 1 && c.leaders()[0] != old_leader, 1_500);

    assert!(replaced);
    assert!(c.nodes[&c.leaders()[0]].term() > old_term);
    c.assert_safe("leader crash");
}

#[test]
fn should_make_a_recovered_old_leader_step_down() {
    let Some(c) = Cluster::new(5, 13) else { return playground_support::skipped() };
    c.net.run_until(&|| c.leaders().len() == 1, 1_500);
    let old_leader = c.leaders()[0].clone();
    c.net.crash(&old_leader);
    c.net.run_until(&|| c.leaders().len() == 1 && c.leaders()[0] != old_leader, 1_500);
    let new_leader = c.leaders()[0].clone();

    c.net.recover(&old_leader);
    c.net.run(600);

    assert_eq!(c.nodes[&old_leader].role(), Role::Follower);
    assert_eq!(c.nodes[&old_leader].leader(), Some(new_leader.clone()));
    assert_eq!(c.leaders(), vec![new_leader]);
    c.assert_safe("old leader returns");
}

#[test]
fn should_not_elect_anything_without_a_majority() {
    let Some(c) = Cluster::new(5, 17) else { return playground_support::skipped() };
    c.net.run_until(&|| c.leaders().len() == 1, 1_500);
    let leader = c.leaders()[0].clone();
    let followers: Vec<String> = c.ids.iter().filter(|i| **i != leader).cloned().collect();
    c.net.crash(&leader);
    for f in followers.iter().take(3) {
        c.net.crash(f); // four of five down
    }

    c.net.run(3_000);

    assert!(c.leaders().is_empty(), "a single survivor of five can never win");
    c.assert_safe("no majority");
}

#[test]
fn should_let_the_majority_side_elect_and_strip_the_minority_leader_of_its_leadership() {
    let Some(c) = Cluster::new(5, 19) else { return playground_support::skipped() };
    c.net.run_until(&|| c.leaders().len() == 1, 1_500);
    let old_leader = c.leaders()[0].clone();
    let mut minority = vec![old_leader.clone()];
    let mut majority = Vec::new();
    for id in &c.ids {
        if *id != old_leader {
            if minority.len() < 2 { minority.push(id.clone()) } else { majority.push(id.clone()) }
        }
    }
    let (min_refs, maj_refs): (Vec<&str>, Vec<&str>) = (minority.iter().map(|s| s.as_str()).collect(), majority.iter().map(|s| s.as_str()).collect());

    c.net.partition(&[&min_refs, &maj_refs]);
    c.net.run(1_500);

    assert!(c.has_single_leader_among(&majority), "the majority elects a leader");
    for id in &minority {
        assert_ne!(c.nodes[id].role(), Role::Leader, "{id} in the minority");
    }
    c.assert_safe("partition");
}

#[test]
fn should_converge_to_one_leader_and_one_term_after_a_partition_heals() {
    let Some(c) = Cluster::new(5, 23) else { return playground_support::skipped() };
    c.net.run_until(&|| c.leaders().len() == 1, 1_500);
    c.net.partition(&[&["n0", "n1"], &["n2", "n3", "n4"]]);
    c.net.run(2_000);

    c.net.heal();
    let converged = c.net.run_until(&|| c.leaders().len() == 1 && c.all_agree(), 2_000);

    assert!(converged);
    c.assert_safe("heal");
}

#[test]
fn should_still_elect_with_twenty_percent_message_loss() {
    let Some(c) = Cluster::new(5, 29) else { return playground_support::skipped() };
    c.net.set_drop_rate(0.2);

    let elected = c.net.run_until(&|| c.leaders().len() == 1, 4_000);
    c.net.run(3_000);

    assert!(elected);
    c.assert_safe("lossy network");
}

#[test]
fn should_replay_the_same_run_from_the_same_seed() {
    let mut runs = Vec::new();
    for _ in 0..2 {
        let Some(c) = Cluster::new(5, 31) else { return playground_support::skipped() };
        c.net.run(3_000);
        runs.push(format!("{:?}@{}", c.leaders(), c.nodes["n0"].term()));
    }

    assert_eq!(runs[0], runs[1]);
}

#[test]
fn should_keep_election_safety_over_100_random_fault_schedules() {
    for seed in 0..100u64 {
        let Some(c) = Cluster::new(5, 1_000 + seed) else { return playground_support::skipped() };
        let mut faults = Rng::new(seed);
        for _ in 0..12 {
            c.net.run(150 + faults.below(400));
            let victim = c.ids[faults.below(5) as usize].clone();
            match faults.below(4) {
                0 => c.net.crash(&victim),
                1 => c.net.recover(&victim),
                2 => {
                    let others: Vec<&str> = c.ids.iter().filter(|x| **x != victim).map(|s| s.as_str()).collect();
                    c.net.partition(&[&[victim.as_str()], &others]);
                }
                _ => c.net.heal(),
            }
        }
        c.net.heal();
        for id in &c.ids {
            c.net.recover(id);
        }
        let settled = c.net.run_until(&|| c.leaders().len() == 1 && c.all_agree(), 3_000);

        c.assert_safe(&format!("seed {seed}"));
        assert!(settled, "seed {seed} settles after faults end");
    }
}
