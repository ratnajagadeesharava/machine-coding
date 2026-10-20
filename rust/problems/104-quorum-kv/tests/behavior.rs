#![allow(unused_imports, clippy::needless_range_loop)]
use p104_quorum_kv::{contract::*, create};
use playground_support::{Rng, SimNet};
use std::collections::HashMap;
use std::sync::Arc;

const IDS: [&str; 3] = ["n1", "n2", "n3"];

/// Three replicas n1, n2, n3.
struct Cluster {
    net: SimNet,
    nodes: HashMap<String, Arc<dyn KvNode>>,
    requests: usize,
}

impl Cluster {
    fn new(seed: u64) -> Option<Cluster> {
        let net = SimNet::new(seed);
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
        Some(Cluster {
            net,
            nodes,
            requests: 0,
        })
    }

    fn node(&self, id: &str) -> &Arc<dyn KvNode> {
        &self.nodes[id]
    }

    fn start(&mut self, via: &str, put: Option<&str>, key: &str) -> String {
        self.requests += 1;
        let rid = format!("r{}", self.requests);
        match put {
            Some(v) => self.nodes[via].put(&rid, key, v),
            None => self.nodes[via].get(&rid, key),
        }
        rid
    }

    fn finish(&self, via: &str, rid: &str) -> Reply {
        let n = self.nodes[via].clone();
        let r = rid.to_string();
        self.net
            .run_until(&move || n.reply(&r) != Reply::Pending, 3_000);
        self.nodes[via].reply(rid)
    }

    fn put(&mut self, via: &str, key: &str, value: &str) -> Reply {
        let rid = self.start(via, Some(value), key);
        self.finish(via, &rid)
    }

    fn get(&mut self, via: &str, key: &str) -> Reply {
        let rid = self.start(via, None, key);
        self.finish(via, &rid)
    }
}

macro_rules! cluster {
    ($seed:expr) => {
        match Cluster::new($seed) {
            Some(c) => c,
            None => return playground_support::skipped(),
        }
    };
}

fn value(s: &str) -> Reply {
    Reply::Value(s.to_string())
}

#[test]
fn should_read_what_was_written_from_any_node() {
    let mut c = cluster!(1);

    assert_eq!(c.put("n1", "k", "v1"), Reply::PutOk);

    for via in IDS {
        assert_eq!(c.get(via, "k"), value("v1"), "{via}");
    }
}

#[test]
fn should_report_not_found_for_an_unknown_key() {
    let mut c = cluster!(1);

    assert_eq!(c.get("n2", "missing"), Reply::NotFound);
}

#[test]
fn should_stay_pending_until_a_majority_has_answered() {
    let mut c = cluster!(1);
    c.net.crash("n2");
    c.net.crash("n3");

    let rid = c.start("n1", Some("v"), "k");
    c.net.run(500);

    assert_eq!(c.node("n1").reply(&rid), Reply::Pending);
}

#[test]
fn should_fail_within_a_second_when_no_majority_is_reachable() {
    let mut c = cluster!(1);
    c.net.crash("n2");
    c.net.crash("n3");

    let put = c.start("n1", Some("v"), "k");
    let get = c.start("n1", None, "k");
    c.net.run(1_100);

    assert_eq!(c.node("n1").reply(&put), Reply::Failed);
    assert_eq!(c.node("n1").reply(&get), Reply::Failed);
}

#[test]
fn should_keep_working_with_one_node_down() {
    let mut c = cluster!(2);
    c.net.crash("n3");

    assert_eq!(c.put("n1", "k", "v1"), Reply::PutOk);

    assert_eq!(c.get("n2", "k"), value("v1"));
}

#[test]
fn should_read_the_latest_write_even_when_a_different_node_is_down_than_during_the_write() {
    let mut c = cluster!(3);
    c.net.crash("n3");
    c.put("n1", "k", "old");
    c.net.run(10);
    c.put("n1", "k", "new");
    c.net.recover("n3");
    c.net.crash("n1");

    assert_eq!(c.get("n3", "k"), value("new"));
}

#[test]
fn should_repair_a_stale_replica_on_read() {
    let mut c = cluster!(4);
    c.net.crash("n3");
    c.put("n1", "k", "v1");
    c.net.recover("n3");
    assert_eq!(c.node("n3").local("k"), None);

    assert_eq!(c.get("n1", "k"), value("v1"));
    c.net.run(300);

    assert_eq!(c.node("n3").local("k").as_deref(), Some("v1"));
}

#[test]
fn should_repair_a_replica_whose_answer_arrives_after_the_reply_was_decided() {
    let mut c = cluster!(5);
    c.net.crash("n3");
    c.put("n1", "k", "v1");
    c.net.recover("n3");
    c.net.set_latency(1, 1);

    c.get("n1", "k");
    c.net.run(500);

    assert_eq!(c.node("n3").local("k").as_deref(), Some("v1"));
}

#[test]
fn should_keep_resending_a_write_so_every_replica_gets_it_despite_loss() {
    let mut c = cluster!(6);
    c.net.set_drop_rate(0.4);

    let r = c.put("n1", "k", "v1");
    c.net.run(900);

    assert_eq!(r, Reply::PutOk);
    assert_eq!(c.node("n2").local("k").as_deref(), Some("v1"));
    assert_eq!(c.node("n3").local("k").as_deref(), Some("v1"));
}

#[test]
fn should_not_write_on_the_minority_partition_but_on_the_majority() {
    let mut c = cluster!(7);
    c.net.partition(&[&["n1"], &["n2", "n3"]]);

    assert_eq!(c.put("n1", "k", "lonely"), Reply::Failed);
    assert_eq!(c.put("n2", "k", "majority"), Reply::PutOk);
    c.net.heal();
    c.net.run(200);

    assert_eq!(c.get("n1", "k"), value("majority"));
}

#[test]
fn should_let_the_newer_write_win() {
    let mut c = cluster!(8);
    c.put("n1", "k", "first");
    c.net.run(5);
    c.put("n2", "k", "second");
    c.net.run(5);
    c.put("n3", "k", "third");

    for via in IDS {
        assert_eq!(c.get(via, "k"), value("third"), "{via}");
    }
}

#[test]
fn should_converge_on_the_larger_node_id_for_writes_in_the_same_millisecond() {
    let mut c = cluster!(9);

    let a = c.start("n1", Some("from-n1"), "k");
    let b = c.start("n3", Some("from-n3"), "k");
    c.finish("n1", &a);
    c.finish("n3", &b);
    c.net.run(1_200);

    for id in IDS {
        assert_eq!(c.node(id).local("k").as_deref(), Some("from-n3"), "{id}");
    }
}

#[test]
fn should_ignore_a_request_id_that_was_already_used() {
    let mut c = cluster!(10);
    let rid = c.start("n1", Some("v1"), "k");
    c.finish("n1", &rid);

    c.node("n1").put(&rid, "k", "other");
    c.net.run(300);

    assert_eq!(c.node("n1").reply(&rid), Reply::PutOk);
    assert_eq!(c.get("n2", "k"), value("v1"));
}

#[test]
fn should_report_pending_for_an_unknown_request_id() {
    let c = cluster!(11);

    assert_eq!(c.node("n1").reply("nope"), Reply::Pending);
}

#[test]
fn should_survive_duplicated_messages() {
    let mut c = cluster!(12);
    c.net.set_duplicate_rate(0.5);

    c.put("n1", "k", "v1");
    c.put("n2", "k", "v2");

    assert_eq!(c.get("n3", "k"), value("v2"));
}

#[test]
fn should_always_read_the_last_acknowledged_write_under_random_faults() {
    if create("probe", &["probe"], Arc::new(|_, _| {})).is_none() {
        return playground_support::skipped();
    }
    for seed in 0..100u64 {
        let mut c = Cluster::new(seed).unwrap();
        let mut rng = Rng::new(seed * 31 + 7);
        c.net.set_drop_rate(0.1);
        let mut last_ok: HashMap<String, String> = HashMap::new();
        let mut down: Option<&str> = None;
        for op in 0..40 {
            if rng.below(4) == 0 {
                if let Some(d) = down {
                    c.net.recover(d);
                }
                down = if rng.below(2) == 0 {
                    None
                } else {
                    Some(IDS[rng.below(3) as usize])
                };
                if let Some(d) = down {
                    c.net.crash(d);
                }
            }
            let up: Vec<&str> = IDS.iter().copied().filter(|i| Some(*i) != down).collect();
            let via = up[rng.below(up.len() as u64) as usize];
            let key = format!("k{}", rng.below(3));
            c.net.run(1 + rng.below(20));
            if rng.below(2) == 0 {
                let v = format!("v{op}");
                assert_eq!(
                    c.put(via, &key, &v),
                    Reply::PutOk,
                    "seed {seed} op {op} put"
                );
                last_ok.insert(key, v);
            } else {
                let r = c.get(via, &key);
                match last_ok.get(&key) {
                    Some(v) => assert_eq!(r, value(v), "seed {seed} op {op} get {key}"),
                    None => assert_eq!(r, Reply::NotFound, "seed {seed} op {op} get {key}"),
                }
            }
        }
    }
}
