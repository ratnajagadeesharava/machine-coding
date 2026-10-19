#![allow(unused_imports, clippy::needless_range_loop)]
use p99_leader_election::{contract::*, create};
use playground_support::SimNet;
use std::collections::HashMap;
use std::sync::Arc;

fn build(
    net: &SimNet,
    ids: &[String],
    seed_of: impl Fn(&str) -> u64,
) -> Option<HashMap<String, Arc<dyn ElectionNode>>> {
    let mut nodes = HashMap::new();
    for id in ids {
        let peers: Vec<&str> = ids
            .iter()
            .filter(|p| *p != id)
            .map(|s| s.as_str())
            .collect();
        let node = create(id, &peers, net.sender(id), seed_of(id))?;
        let (n1, n2) = (node.clone(), node.clone());
        net.register(
            id,
            Arc::new(move |f, p| n1.on_message(f, p)),
            Arc::new(move |t| n2.on_tick(t)),
        );
        nodes.insert(id.clone(), node);
    }
    Some(nodes)
}

#[test]
fn should_elect_within_a_few_timeouts_in_an_eleven_node_cluster_with_jittery_latency() {
    let net = SimNet::new(5);
    net.set_latency(1, 40);
    let ids: Vec<String> = (0..11).map(|i| format!("n{i}")).collect();
    let Some(nodes) = build(&net, &ids, |id| {
        id.len() as u64 * 977 + id.as_bytes()[1] as u64
    }) else {
        return playground_support::skipped();
    };

    let elected = net.run_until(
        &|| nodes.values().filter(|n| n.role() == Role::Leader).count() == 1,
        5_000,
    );

    assert!(elected);
}

#[test]
fn should_survive_the_leader_crashing_ten_times_in_a_row() {
    let net = SimNet::new(9);
    let ids: Vec<String> = ["a", "b", "c", "d", "e"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let Some(nodes) = build(&net, &ids, |id| id.as_bytes()[0] as u64 * 7) else {
        return playground_support::skipped();
    };
    let leader_of = |net: &SimNet| {
        ids.iter()
            .find(|id| !net.is_crashed(id) && nodes[*id].role() == Role::Leader)
            .cloned()
    };
    let mut last_term = 0;

    for _ in 0..10 {
        net.run_until(
            &|| {
                ids.iter()
                    .filter(|id| !net.is_crashed(id) && nodes[*id].role() == Role::Leader)
                    .count()
                    == 1
            },
            2_000,
        );
        let leader = leader_of(&net).expect("a leader");
        assert!(nodes[&leader].term() + 1 > last_term);
        last_term = nodes[&leader].term();
        net.crash(&leader);
        net.run(800); // long enough for the others to elect a new leader
        net.recover(&leader);
        net.run(400);
    }

    assert!(last_term >= 10);
}
