#![allow(unused_imports)]
use p105_raft_log::{contract::*, create};
use playground_support::{skipped, Rng, SimNet};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn cluster(net: &SimNet, ids: &[&str], seed: u64) -> Option<HashMap<String, Arc<dyn RaftNode>>> {
    let mut nodes = HashMap::new();
    for (i, id) in ids.iter().enumerate() {
        let peers: Vec<&str> = ids.iter().copied().filter(|p| p != id).collect();
        let node = create(id, &peers, net.sender(id), seed.wrapping_mul(7).wrapping_add(i as u64 * 13 + 1))?;
        let (n1, n2) = (node.clone(), node.clone());
        net.register(id, Arc::new(move |f, p| n1.on_message(f, p)), Arc::new(move |t| n2.on_tick(t)));
        nodes.insert(id.to_string(), node);
    }
    Some(nodes)
}

fn leader(net: &SimNet, nodes: &HashMap<String, Arc<dyn RaftNode>>) -> Option<String> {
    let l: Vec<&String> = nodes.keys().filter(|k| !net.is_crashed(k) && nodes[*k].role() == Role::Leader).collect();
    if l.len() == 1 {
        Some(l[0].clone())
    } else {
        None
    }
}

#[test]
fn should_replicate_a_hundred_commands_quickly_through_a_five_node_cluster() {
    let net = SimNet::new(1);
    let ids = ["a", "b", "c", "d", "e"];
    let Some(nodes) = cluster(&net, &ids, 1) else { return skipped() };
    net.run_until(&|| leader(&net, &nodes).is_some(), 2_000);
    let l = nodes[&leader(&net, &nodes).unwrap()].clone();
    let mut expected = Vec::new();

    for i in 0..100 {
        assert!(l.propose(&format!("cmd{i}")).is_some());
        expected.push(format!("cmd{i}"));
    }

    assert!(net.run_until(&|| ids.iter().all(|i| nodes[*i].committed() == expected), 1_000));
}

#[test]
fn should_survive_a_rolling_restart_of_every_node_without_losing_committed_commands() {
    let net = SimNet::new(2);
    let ids = ["a", "b", "c"];
    let Some(nodes) = cluster(&net, &ids, 2) else { return skipped() };
    let mut expected: Vec<String> = Vec::new();
    for round in 0..6 {
        net.run_until(&|| leader(&net, &nodes).is_some(), 3_000);
        let l = nodes[&leader(&net, &nodes).unwrap()].clone();
        let cmd = format!("r{round}");
        if l.propose(&cmd).is_some() {
            let target = expected.len() + 1;
            if net.run_until(&|| l.committed().len() >= target, 1_000) {
                expected.push(cmd);
            }
        }
        let victim = ids[round % 3];
        net.crash(victim);
        net.run(600);
        net.recover(victim);
        net.run(400);
    }

    net.run(2_000);

    for id in ids {
        assert!(nodes[id].committed().starts_with(&expected), "{id}");
    }
}

#[derive(Default)]
struct Watch {
    leader_of_term: HashMap<u64, String>,
    before: HashMap<String, Vec<String>>,
    longest: Vec<String>,
    violations: Vec<String>,
}

#[test]
fn should_keep_safety_with_five_nodes_and_heavy_faults() {
    for seed in 0..60u64 {
        let net = SimNet::new(seed + 1_000);
        let ids = ["a", "b", "c", "d", "e"];
        let Some(nodes) = cluster(&net, &ids, seed) else { return skipped() };
        let mut rng = Rng::new(seed + 1);
        let watch = Arc::new(Mutex::new(Watch::default()));
        let (w, ns) = (watch.clone(), nodes.clone());
        net.observe(Arc::new(move || {
            let mut w = w.lock().unwrap();
            for id in ["a", "b", "c", "d", "e"] {
                let n = &ns[id];
                if n.role() == Role::Leader {
                    let prev = w.leader_of_term.entry(n.term()).or_insert_with(|| id.to_string()).clone();
                    if prev != id {
                        w.violations.push(format!("two leaders in term {}", n.term()));
                    }
                }
                let now = n.committed();
                let old = w.before.get(id).cloned().unwrap_or_default();
                if now.len() < old.len() || now[..old.len()] != old[..] {
                    w.violations.push(format!("{id} rewrote its committed entries"));
                }
                w.before.insert(id.to_string(), now.clone());
                let common = now.len().min(w.longest.len());
                if now[..common] != w.longest[..common] {
                    w.violations.push(format!("{id} diverged"));
                }
                if now.len() > w.longest.len() {
                    w.longest = now;
                }
            }
        }));
        net.set_drop_rate(0.1);
        net.set_duplicate_rate(0.2);
        net.set_latency(1, 60);
        for round in 0..40 {
            match rng.below(6) {
                0 => net.crash(ids[rng.below(5) as usize]),
                1 => ids.iter().for_each(|i| net.recover(i)),
                2 => net.partition(&[&["a", "b"], &["c", "d", "e"]]),
                3 => net.partition(&[&["a", "c", "e"], &["b", "d"]]),
                4 => net.heal(),
                _ => {}
            }
            for id in ids {
                if !net.is_crashed(id) {
                    nodes[id].propose(&format!("s{seed}-{round}{id}"));
                }
            }
            net.run(30 + rng.below(300));
        }
        let violations = watch.lock().unwrap().violations.clone();
        assert!(violations.is_empty(), "seed {seed}: {violations:?}");
    }
}
