#![allow(unused_imports)]
use p104_quorum_kv::{contract::*, create};
use playground_support::{skipped, Rng, SimNet};
use std::collections::HashMap;
use std::sync::Arc;

const IDS: [&str; 5] = ["n1", "n2", "n3", "n4", "n5"];

fn cluster(net: &SimNet) -> Option<HashMap<String, Arc<dyn KvNode>>> {
    let mut nodes = HashMap::new();
    for id in IDS {
        let node = create(id, &IDS, net.sender(id))?;
        let (n1, n2) = (node.clone(), node.clone());
        net.register(
            id,
            Arc::new(move |f, p| n1.on_message(f, p)),
            Arc::new(move |t| n2.on_tick(t)),
        );
        nodes.insert(id.to_string(), node);
    }
    Some(nodes)
}

fn await_reply(net: &SimNet, node: &Arc<dyn KvNode>, rid: &str) -> Reply {
    let (n, r) = (node.clone(), rid.to_string());
    net.run_until(&move || n.reply(&r) != Reply::Pending, 3_000);
    node.reply(rid)
}

#[test]
fn should_use_a_majority_of_five_not_of_three() {
    let net = SimNet::new(1);
    let Some(nodes) = cluster(&net) else {
        return skipped();
    };
    net.crash("n3");
    net.crash("n4");
    nodes["n1"].put("a", "k", "v");
    assert_eq!(await_reply(&net, &nodes["n1"], "a"), Reply::PutOk);

    net.crash("n5");
    nodes["n1"].put("b", "k", "w");
    assert_eq!(await_reply(&net, &nodes["n1"], "b"), Reply::Failed);
}

#[test]
fn should_stay_consistent_with_rotating_crashes_and_partitions() {
    for seed in 0..60u64 {
        let net = SimNet::new(seed);
        let Some(nodes) = cluster(&net) else {
            return skipped();
        };
        let mut rng = Rng::new(seed + 1);
        net.set_drop_rate(0.1);
        net.set_duplicate_rate(0.1);
        let mut last_ok: Option<String> = None;
        for op in 0..40 {
            net.heal();
            for id in IDS {
                net.recover(id);
            }
            net.run(2); // a recovered node needs a tick to learn the time
            match rng.below(3) {
                1 => {
                    net.crash(IDS[rng.below(5) as usize]);
                    net.crash(IDS[rng.below(5) as usize]);
                }
                2 => net.partition(&[&["n1", "n2", "n3"], &["n4", "n5"]]),
                _ => {}
            }
            let via = IDS[rng.below(5) as usize];
            if net.is_crashed(via) {
                continue;
            }
            let rid = format!("r{op}");
            let node = &nodes[via];
            if rng.below(2) == 0 {
                node.put(&rid, "k", &format!("v{op}"));
                last_ok = if await_reply(&net, node, &rid) == Reply::PutOk {
                    Some(format!("v{op}"))
                } else {
                    None
                };
            } else {
                node.get(&rid, "k");
                if let (Reply::Value(v), Some(expected)) = (await_reply(&net, node, &rid), &last_ok)
                {
                    assert_eq!(&v, expected, "seed {seed} op {op}");
                }
            }
            net.run(rng.below(30));
        }
    }
}

#[test]
fn should_converge_every_replica_after_the_network_heals() {
    let net = SimNet::new(5);
    let Some(nodes) = cluster(&net) else {
        return skipped();
    };
    net.partition(&[&["n1", "n2", "n3"], &["n4", "n5"]]);
    nodes["n1"].put("p", "k", "during-partition");
    assert_eq!(await_reply(&net, &nodes["n1"], "p"), Reply::PutOk);
    net.heal();

    nodes["n4"].get("g", "k");
    assert_eq!(
        await_reply(&net, &nodes["n4"], "g"),
        Reply::Value("during-partition".into())
    );
    net.run(500);

    for id in IDS {
        assert_eq!(
            nodes[id].local("k").as_deref(),
            Some("during-partition"),
            "{id}"
        );
    }
}
