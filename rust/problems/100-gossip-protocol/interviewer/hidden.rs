#![allow(unused_imports, clippy::needless_range_loop)]
use p100_gossip_protocol::{contract::*, create};
use playground_support::SimNet;
use std::sync::Arc;

fn build(net: &SimNet, n: usize, seed: u64) -> Option<Vec<Arc<dyn GossipNode>>> {
    let ids: Vec<String> = (0..n).map(|i| format!("n{i}")).collect();
    let mut nodes = Vec::new();
    for (i, id) in ids.iter().enumerate() {
        let peers: Vec<&str> = ids
            .iter()
            .filter(|p| *p != id)
            .map(|s| s.as_str())
            .collect();
        let node = create(id, &peers, net.sender(id), seed + i as u64 * 31)?;
        let (n1, n2) = (node.clone(), node.clone());
        net.register(
            id,
            Arc::new(move |f, p| n1.on_message(f, p)),
            Arc::new(move |t| n2.on_tick(t)),
        );
        nodes.push(node);
    }
    Some(nodes)
}

#[test]
fn should_spread_a_write_to_hundred_nodes_in_order_log_n() {
    let net = SimNet::new(3);
    let Some(nodes) = build(&net, 100, 7) else {
        return playground_support::skipped();
    };
    nodes[0].put("k", "v");

    let spread = net.run_until(
        &|| nodes.iter().all(|n| n.get("k").as_deref() == Some("v")),
        2_500,
    );

    assert!(spread);
}

#[test]
fn should_detect_ten_simultaneous_crashes_in_a_hundred_node_cluster() {
    let net = SimNet::new(4);
    let Some(nodes) = build(&net, 100, 9) else {
        return playground_support::skipped();
    };
    net.run(1_000);
    for i in 0..10 {
        net.crash(&format!("n{i}"));
    }

    net.run(6_000);

    for node in nodes.iter().skip(10) {
        assert_eq!(node.alive_members().len(), 90);
    }
}
