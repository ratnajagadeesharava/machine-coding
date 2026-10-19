#![allow(unused_imports, clippy::needless_range_loop)]
use p101_two_phase_commit::{contract::*, create_coordinator, create_participant};
use playground_support::SimNet;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct MemoryLog(Mutex<Vec<String>>);
impl DurableLog for MemoryLog {
    fn append(&self, record: &str) {
        self.0.lock().unwrap().push(record.to_string());
    }
    fn records(&self) -> Vec<String> {
        self.0.lock().unwrap().clone()
    }
}

const PARTICIPANTS: [&str; 3] = ["A", "B", "C"];

fn env(seed: u64) -> Option<(SimNet, Arc<dyn CoordinatorNode>, HashMap<String, Arc<dyn ParticipantNode>>)> {
    let net = SimNet::new(seed);
    let coordinator = create_coordinator("K", &PARTICIPANTS, net.sender("K"), Arc::new(MemoryLog::default()))?;
    let (n1, n2) = (coordinator.clone(), coordinator.clone());
    net.register("K", Arc::new(move |f, p| n1.on_message(f, p)), Arc::new(move |t| n2.on_tick(t)));
    let mut participants = HashMap::new();
    for p in PARTICIPANTS {
        let node = create_participant(p, "K", net.sender(p), Arc::new(MemoryLog::default()))?;
        let (m1, m2) = (node.clone(), node.clone());
        net.register(p, Arc::new(move |f, m| m1.on_message(f, m)), Arc::new(move |t| m2.on_tick(t)));
        participants.insert(p.to_string(), node);
    }
    Some((net, coordinator, participants))
}

#[test]
fn should_commit_two_hundred_independent_transactions_running_at_the_same_time() {
    let Some((net, k, ps)) = env(1) else { return playground_support::skipped() };
    for i in 0..200 {
        let (a, b, c) = (format!("a{i}"), format!("b{i}"), format!("c{i}"));
        k.begin(&format!("t{i}"), &[("A", &[(&a, "v")]), ("B", &[(&b, "v")]), ("C", &[(&c, "v")])]).unwrap();
    }

    net.run(3_000);

    for i in 0..200 {
        assert_eq!(k.outcome(&format!("t{i}")), Ok(Outcome::Committed));
        assert_eq!(ps["C"].get(&format!("c{i}")), Some("v".to_string()));
    }
}

#[test]
fn should_let_at_most_one_of_many_conflicting_transactions_commit_at_a_time() {
    let Some((net, k, ps)) = env(2) else { return playground_support::skipped() };
    net.set_latency(5, 5);
    for i in 0..50 {
        let v = format!("from-{i}");
        k.begin(&format!("t{i}"), &[("A", &[("hot", &v)]), ("B", &[("hot", &v)])]).unwrap();
        net.run(7);
    }
    net.run(3_000);

    let winners: Vec<usize> = (0..50).filter(|i| k.outcome(&format!("t{i}")) == Ok(Outcome::Committed)).collect();
    assert!(!winners.is_empty());
    assert_eq!(ps["A"].get("hot"), ps["B"].get("hot"));
    assert_eq!(ps["A"].get("hot"), Some(format!("from-{}", winners.last().unwrap()))); // the last committed one wins on both
}
