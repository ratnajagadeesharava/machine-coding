#![allow(unused_imports, clippy::needless_range_loop)]
use p101_two_phase_commit::{contract::*, create_coordinator, create_participant};
use playground_support::{Rng, SimNet};
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

/// A coordinator "K" and participants "A", "B", "C" in the simulation. Restarting a node builds a NEW instance on the SAME log.
struct Cluster {
    net: SimNet,
    logs: HashMap<String, Log>,
    participants: HashMap<String, Arc<dyn ParticipantNode>>,
    coordinator: Arc<dyn CoordinatorNode>,
}

impl Cluster {
    fn new(seed: u64) -> Option<Cluster> {
        let net = SimNet::new(seed);
        let mut logs: HashMap<String, Log> = HashMap::new();
        for id in ["K", "A", "B", "C"] {
            logs.insert(id.to_string(), Arc::new(MemoryLog::default()));
        }
        let coordinator = Self::start_coordinator(&net, &logs)?;
        let mut participants = HashMap::new();
        for p in PARTICIPANTS {
            participants.insert(p.to_string(), Self::start_participant(&net, &logs, p)?);
        }
        Some(Cluster { net, logs, participants, coordinator })
    }

    fn start_coordinator(net: &SimNet, logs: &HashMap<String, Log>) -> Option<Arc<dyn CoordinatorNode>> {
        let node = create_coordinator("K", &PARTICIPANTS, net.sender("K"), logs["K"].clone())?;
        let (n1, n2) = (node.clone(), node.clone());
        net.register("K", Arc::new(move |f, p| n1.on_message(f, p)), Arc::new(move |t| n2.on_tick(t)));
        Some(node)
    }

    fn start_participant(net: &SimNet, logs: &HashMap<String, Log>, p: &str) -> Option<Arc<dyn ParticipantNode>> {
        let node = create_participant(p, "K", net.sender(p), logs[p].clone())?;
        let (n1, n2) = (node.clone(), node.clone());
        net.register(p, Arc::new(move |f, m| n1.on_message(f, m)), Arc::new(move |t| n2.on_tick(t)));
        Some(node)
    }

    fn restart_coordinator(&mut self) {
        self.coordinator = Self::start_coordinator(&self.net, &self.logs).unwrap();
    }

    fn restart_participant(&mut self, p: &str) {
        let node = Self::start_participant(&self.net, &self.logs, p).unwrap();
        self.participants.insert(p.to_string(), node);
    }

    fn outcome(&self, tx: &str) -> Outcome {
        self.coordinator.outcome(tx).unwrap()
    }

    fn await_outcome(&self, tx: &str, max_ms: u64) -> Outcome {
        self.net.run_until(&|| self.outcome(tx) != Outcome::Pending, max_ms);
        self.outcome(tx)
    }

    fn quiet(&self) -> bool {
        self.participants.values().all(|p| p.in_doubt().is_empty())
    }

    fn has(&self, p: &str, key: &str) -> bool {
        self.participants[p].get(key).is_some()
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

#[test]
fn should_commit_across_participants_when_everyone_votes_yes() {
    let c = cluster!(1);

    c.coordinator.begin("t1", &[("A", &[("x", "1")]), ("B", &[("y", "2")])]).unwrap();

    assert_eq!(c.await_outcome("t1", 1_000), Outcome::Committed);
    c.net.run(300);
    assert_eq!(c.participants["A"].get("x"), Some("1".to_string()));
    assert_eq!(c.participants["B"].get("y"), Some("2".to_string()));
    assert_eq!(c.participants["C"].get("x"), None);
    assert!(!c.participants["A"].is_locked("x"));
    assert!(c.quiet());
}

#[test]
fn should_abort_everywhere_when_one_participant_votes_no() {
    let c = cluster!(2);

    c.coordinator.begin("t1", &[("A", &[("x", "1")]), ("B", &[("y", "REJECT")]), ("C", &[("z", "3")])]).unwrap();

    assert_eq!(c.await_outcome("t1", 1_000), Outcome::Aborted);
    c.net.run(500);
    assert!(!c.has("A", "x") && !c.has("C", "z"));
    assert!(!c.participants["A"].is_locked("x"), "the YES voters release their locks");
    assert!(!c.participants["C"].is_locked("z"));
    assert!(c.quiet());
}

#[test]
fn should_hold_locks_between_vote_and_decision_and_refuse_a_conflicting_transaction() {
    let c = cluster!(3);
    c.net.set_latency(40, 40); // long enough to observe the window
    c.coordinator.begin("t1", &[("A", &[("k", "from-t1")])]).unwrap();
    c.net.run(60); // PREPARE has arrived, the YES is on its way

    assert!(c.participants["A"].is_locked("k"));
    assert_eq!(c.participants["A"].in_doubt(), vec!["t1".to_string()]);
    c.coordinator.begin("t2", &[("A", &[("k", "from-t2")])]).unwrap();

    assert_eq!(c.await_outcome("t1", 1_000), Outcome::Committed);
    assert_eq!(c.await_outcome("t2", 1_000), Outcome::Aborted);
    c.net.run(500);
    assert_eq!(c.participants["A"].get("k"), Some("from-t1".to_string()));
}

#[test]
fn should_allow_a_second_transaction_on_the_same_key_once_the_first_is_done() {
    let c = cluster!(4);
    c.coordinator.begin("t1", &[("A", &[("k", "one")])]).unwrap();
    c.await_outcome("t1", 1_000);
    c.net.run(300);

    c.coordinator.begin("t2", &[("A", &[("k", "two")])]).unwrap();

    assert_eq!(c.await_outcome("t2", 1_000), Outcome::Committed);
    c.net.run(300);
    assert_eq!(c.participants["A"].get("k"), Some("two".to_string()));
}

#[test]
fn should_abort_after_half_a_second_when_a_participant_never_answers() {
    let c = cluster!(5);
    c.net.crash("B");

    c.coordinator.begin("t1", &[("A", &[("x", "1")]), ("B", &[("y", "2")])]).unwrap();

    assert_eq!(c.await_outcome("t1", 400), Outcome::Pending, "not before the 500 ms vote timeout");
    assert_eq!(c.await_outcome("t1", 400), Outcome::Aborted);
    c.net.run(500);
    assert!(!c.participants["A"].is_locked("x"), "A voted YES and must be told to release");
}

#[test]
fn should_tell_the_late_participant_the_outcome_when_it_comes_back() {
    let c = cluster!(6);
    c.net.crash("B");
    c.coordinator.begin("t1", &[("A", &[("x", "1")]), ("B", &[("y", "2")])]).unwrap();
    c.await_outcome("t1", 1_000);
    c.net.run(1_000);

    c.net.recover("B");
    c.net.run(1_000);

    assert!(!c.has("B", "y"), "aborted: nothing applied");
    assert!(!c.participants["B"].is_locked("y"));
    assert!(c.participants["B"].in_doubt().is_empty());
}

#[test]
fn should_keep_a_yes_voters_transaction_in_doubt_until_it_learns_the_commit() {
    let c = cluster!(7);
    c.net.set_latency(10, 10);
    c.coordinator.begin("t1", &[("A", &[("x", "1")]), ("B", &[("y", "2")])]).unwrap();
    c.net.run_until(&|| c.participants["B"].in_doubt().contains(&"t1".to_string()), 500); // B has voted YES
    c.net.crash("B");

    assert_eq!(c.await_outcome("t1", 1_000), Outcome::Committed);
    c.net.run(1_000);
    assert_eq!(c.participants["A"].get("x"), Some("1".to_string()));
    assert_eq!(c.participants["B"].in_doubt(), vec!["t1".to_string()], "B is still in doubt and may not decide alone");
    assert!(c.participants["B"].is_locked("y"));

    c.net.recover("B");
    c.net.run(1_000);

    assert_eq!(c.participants["B"].get("y"), Some("2".to_string()));
    assert!(c.participants["B"].in_doubt().is_empty());
}

#[test]
fn should_rebuild_a_participants_locks_and_doubt_from_its_log_after_a_restart_and_finish_the_commit() {
    let mut c = cluster!(8);
    c.net.set_latency(10, 10);
    c.coordinator.begin("t1", &[("A", &[("x", "1")]), ("B", &[("y", "2")])]).unwrap();
    c.net.run_until(&|| c.participants["B"].in_doubt().contains(&"t1".to_string()), 500);
    c.net.crash("B");
    c.await_outcome("t1", 1_000);
    c.net.run(500);

    c.restart_participant("B"); // memory lost, log kept
    assert_eq!(c.participants["B"].in_doubt(), vec!["t1".to_string()]);
    assert!(c.participants["B"].is_locked("y"));
    c.net.run(1_000);

    assert_eq!(c.participants["B"].get("y"), Some("2".to_string()));
    assert!(c.quiet());
}

#[test]
fn should_block_participants_while_the_coordinator_is_gone_and_finish_when_it_returns() {
    let mut c = cluster!(9);
    c.net.set_latency(10, 10);
    c.coordinator.begin("t1", &[("A", &[("x", "1")]), ("B", &[("y", "2")])]).unwrap();
    // the moment the commit decision is logged, the coordinator dies before the decision can be delivered
    c.net.run_until(&|| c.outcome("t1") == Outcome::Committed, 1_000);
    c.net.partition(&[&["K"], &["A", "B", "C"]]);
    c.net.crash("K");
    c.net.run(2_000);

    assert_eq!(c.participants["A"].in_doubt(), vec!["t1".to_string()], "2PC blocks: A cannot decide without K");
    assert!(c.participants["A"].is_locked("x"));
    assert!(!c.has("A", "x"));

    c.net.heal();
    c.restart_coordinator(); // new instance, same log
    assert_eq!(c.outcome("t1"), Outcome::Committed, "the decision survived in the log");
    c.net.run(1_000);

    assert_eq!(c.participants["A"].get("x"), Some("1".to_string()));
    assert_eq!(c.participants["B"].get("y"), Some("2".to_string()));
    assert!(c.quiet());
}

#[test]
fn should_presume_abort_when_the_coordinator_restarts_without_having_decided() {
    let mut c = cluster!(10);
    c.net.set_latency(10, 10);
    c.coordinator.begin("t1", &[("A", &[("x", "1")]), ("B", &[("y", "2")])]).unwrap();
    c.net.run(15); // PREPAREs delivered, no decision yet
    c.net.crash("K");
    c.net.run(100);

    c.restart_coordinator();

    assert_eq!(c.outcome("t1"), Outcome::Aborted);
    c.net.run(1_000);
    assert!(c.quiet());
    assert!(!c.has("A", "x"));
    assert!(!c.participants["A"].is_locked("x"));
}

#[test]
fn should_answer_an_inquiry_about_an_unknown_transaction_without_breaking_anything() {
    let mut c = cluster!(11);
    c.coordinator.begin("t1", &[("A", &[("x", "1")])]).unwrap();
    c.await_outcome("t1", 1_000);
    // a brand-new coordinator with an EMPTY log knows nothing about t1
    c.logs.insert("K".to_string(), Arc::new(MemoryLog::default()));
    c.restart_coordinator();

    c.net.run(1_000);

    assert_eq!(c.participants["A"].get("x"), Some("1".to_string()));
    assert_eq!(c.coordinator.outcome("t1"), Err(TxError::UnknownTransaction));
}

#[test]
fn should_reject_invalid_begin_calls() {
    let c = cluster!(12);
    c.coordinator.begin("t1", &[("A", &[("x", "1")])]).unwrap();

    assert_eq!(c.coordinator.begin("t1", &[("A", &[("y", "1")])]), Err(TxError::InvalidTransaction));
    assert_eq!(c.coordinator.begin("t2", &[]), Err(TxError::InvalidTransaction));
    assert_eq!(c.coordinator.begin("t3", &[("Z", &[("y", "1")])]), Err(TxError::InvalidTransaction));
    assert_eq!(c.coordinator.outcome("nope"), Err(TxError::UnknownTransaction));
}

#[test]
fn should_decide_every_transaction_consistently_on_a_lossy_network() {
    let c = cluster!(13);
    c.net.set_drop_rate(0.2);
    for i in 0..30 {
        let (a, b, cc) = (format!("a{i}"), format!("b{i}"), format!("c{i}"));
        let v = format!("v{i}");
        c.coordinator.begin(&format!("tx{i}"), &[("A", &[(&a, &v)]), ("B", &[(&b, &v)]), ("C", &[(&cc, &v)])]).unwrap();
        c.net.run(40);
    }
    c.net.run(3_000);

    let mut committed = 0;
    for i in 0..30 {
        let o = c.outcome(&format!("tx{i}"));
        assert_ne!(o, Outcome::Pending, "tx{i} decided");
        let (a, b, cc) = (c.has("A", &format!("a{i}")), c.has("B", &format!("b{i}")), c.has("C", &format!("c{i}")));
        assert!(a == b && b == cc, "tx{i} atomic");
        assert_eq!(a, o == Outcome::Committed, "tx{i} outcome matches the data");
        committed += a as usize;
    }
    assert!(c.quiet());
    assert!(committed > 5, "most transactions still make it through");
}

#[test]
fn should_stay_atomic_over_100_random_crash_and_partition_schedules() {
    for seed in 0..100u64 {
        let Some(mut c) = Cluster::new(2_000 + seed) else { return playground_support::skipped() };
        let mut r = Rng::new(seed);
        c.net.set_latency(1, 15);
        for i in 0..6 {
            let (a, b, cc) = (format!("a{i}"), format!("b{i}"), format!("c{i}"));
            let bv = if i % 3 == 0 { "REJECT" } else { "v" };
            c.coordinator.begin(&format!("tx{i}"), &[("A", &[(&a, "v")]), ("B", &[(&b, bv)]), ("C", &[(&cc, "v")])]).unwrap();
            c.net.run(r.below(150));
            let victim = ["A", "B", "C", "K"][r.below(4) as usize];
            match r.below(5) {
                0 => c.net.crash(victim),
                1 => c.net.recover(victim),
                2 => {
                    if victim == "K" {
                        c.net.crash("K");
                        c.restart_coordinator();
                        c.net.crash("K");
                    } else {
                        c.net.crash(victim);
                        c.restart_participant(victim);
                        c.net.crash(victim);
                    }
                }
                3 => {
                    let others: Vec<&str> = ["A", "B", "C", "K"].into_iter().filter(|x| *x != victim).collect();
                    c.net.partition(&[&[victim], &others]);
                }
                _ => c.net.heal(),
            }
        }
        c.net.heal();
        for id in ["A", "B", "C", "K"] {
            c.net.recover(id);
        }
        c.net.run(5_000);

        for i in 0..6 {
            let o = c.outcome(&format!("tx{i}"));
            assert_ne!(o, Outcome::Pending, "seed {seed} tx{i} decided");
            let (a, b, cc) = (c.has("A", &format!("a{i}")), c.has("B", &format!("b{i}")), c.has("C", &format!("c{i}")));
            assert!(a == b && b == cc, "seed {seed} tx{i} atomic ({a} {b} {cc})");
            assert_eq!(a, o == Outcome::Committed, "seed {seed} tx{i} matches outcome {o:?}");
            if i % 3 == 0 {
                assert_eq!(o, Outcome::Aborted, "a REJECT vote can never commit");
            }
        }
        assert!(c.quiet(), "seed {seed}: nobody left in doubt");
    }
}
