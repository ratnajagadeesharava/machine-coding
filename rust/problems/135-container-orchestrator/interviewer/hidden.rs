#![allow(unused_imports, clippy::needless_range_loop)]
use p135_container_orchestrator::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

fn at(node: &str) -> Result<Option<String>, OrchError> {
    Ok(Some(node.to_string()))
}

fn s(v: &str) -> Result<String, OrchError> {
    Ok(v.to_string())
}

#[test]
fn should_replace_an_evicted_container_on_another_node_immediately() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n1", 1000, 1000).unwrap();
    o.add_node("n2", 400, 400).unwrap();
    o.submit("v", 400, 400, 1).unwrap();
    o.submit("u", 100, 100, 300).unwrap();

    assert_eq!(o.submit("hi", 700, 700, 100), at("n1"));

    assert_eq!(o.placement("v"), s("n2"));
    assert!(o.pending().is_empty());
    assert_eq!(o.node_usage("n1"), s("800/1000,800/1000"));
}

#[test]
fn should_never_preempt_when_retrying_pending_containers() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n1", 100, 100).unwrap();
    o.add_node("n2", 100, 100).unwrap();
    o.submit("important", 100, 100, 150).unwrap(); // n1
    o.submit("low", 100, 100, 1).unwrap(); // n2

    assert_eq!(o.remove_node("n1"), Ok(vec!["important->PENDING".to_string()]));

    assert_eq!(o.placement("low"), s("n2"));
    assert_eq!(o.pending(), vec!["important"]);
    o.add_node("n3", 100, 100).unwrap();
    assert_eq!(o.placement("important"), s("n3"));
    assert_eq!(o.placement("low"), s("n2"));
}

#[test]
fn should_keep_the_original_submission_order_of_containers_displaced_by_node_removal() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n1", 100, 100).unwrap();
    o.add_node("n2", 100, 100).unwrap();
    o.submit("a", 100, 100, 1).unwrap(); // n1, oldest
    o.submit("c", 100, 100, 1).unwrap(); // n2
    o.submit("b", 100, 100, 1).unwrap(); // pending, newer than a

    o.remove_node("n1").unwrap();
    assert_eq!(o.pending(), vec!["a", "b"]);

    o.add_node("n3", 100, 100).unwrap();
    assert_eq!(o.placement("a"), s("n3"));
    assert_eq!(o.pending(), vec!["b"]);
}

#[test]
fn should_choose_victims_by_lowest_priority_then_newest_across_priorities() {
    for needed in [400u32, 700] {
        let o = sut!(create(Strategy::FirstFit));
        o.add_node("n", 1000, 1000).unwrap();
        o.submit("a", 300, 300, 3).unwrap();
        o.submit("b", 300, 300, 1).unwrap();
        o.submit("c", 300, 300, 3).unwrap();

        assert_eq!(o.submit("hi", needed, needed, 100), at("n"));

        let expected: Vec<&str> = if needed == 400 {
            vec!["a", "c", "hi"]
        } else {
            vec!["a", "hi"] // b, then the newer of the prio-3 pair
        };
        assert_eq!(o.containers_on("n").unwrap(), expected);
    }
}

#[test]
fn should_let_a_higher_preemptor_evict_containers_that_themselves_could_preempt() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n", 100, 100).unwrap();
    o.submit("r", 100, 100, 100).unwrap();

    assert_eq!(o.submit("s", 100, 100, 200), at("n"));
    assert_eq!(o.placement("r"), s("PENDING"));
    assert_eq!(o.submit("t", 100, 100, 150), Ok(None)); // 150 < 200: cannot evict s
}

#[test]
fn should_choose_the_preemption_node_independently_of_the_strategy() {
    let o = sut!(create(Strategy::WorstFit));
    o.add_node("n1", 1000, 1000).unwrap();
    o.add_node("n2", 2000, 2000).unwrap();
    o.submit("y", 1900, 1900, 1).unwrap(); // only n2 can host it
    o.submit("x", 900, 900, 1).unwrap(); //   n1 (n2 has 100 left)

    // both nodes would need one eviction; the tie goes to n1 although worst fit prefers the bigger n2
    assert_eq!(o.submit("urgent", 950, 950, 100), at("n1"));
    assert_eq!(o.placement("x"), s("PENDING"));
    assert_eq!(o.placement("y"), s("n2"));
}

// small deterministic generator, so the sequence is the same on every run
struct Lcg(u64);
impl Lcg {
    fn next(&mut self, n: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) % n
    }
}

struct Model {
    o: Arc<dyn Orchestrator>,
    caps: HashMap<String, (u64, u64)>,
    live: HashMap<String, (u64, u64, i32, u64)>, // cpu, mem, prio, seq
    seq: u64,
}

impl Model {
    fn add_node(&mut self, id: &str, cpu: u32, mem: u32) {
        if self.caps.contains_key(id) {
            return;
        }
        self.o.add_node(id, cpu, mem).unwrap();
        self.caps.insert(id.to_string(), (cpu as u64, mem as u64));
    }

    fn remove_node(&mut self, id: &str) {
        if !self.caps.contains_key(id) {
            return;
        }
        let on = self.o.containers_on(id).unwrap();
        let result = self.o.remove_node(id).unwrap();
        self.caps.remove(id);
        assert_eq!(result.len(), on.len());
        for (r, c) in result.iter().zip(on.iter()) {
            assert!(r.starts_with(&format!("{c}->")), "{r} vs {c}");
        }
    }

    fn submit(&mut self, id: &str, cpu: u32, mem: u32, prio: i32) {
        self.live
            .insert(id.to_string(), (cpu as u64, mem as u64, prio, self.seq));
        self.seq += 1;
        self.o.submit(id, cpu, mem, prio).unwrap();
    }

    fn check(&self) {
        let mut seen = HashSet::new();
        let mut free = vec![];
        for (nid, cap) in &self.caps {
            let on = self.o.containers_on(nid).unwrap();
            let mut sorted = on.clone();
            sorted.sort();
            assert_eq!(on, sorted);
            let (mut cpu, mut mem) = (0, 0);
            for c in &on {
                assert!(seen.insert(c.clone()));
                cpu += self.live[c].0;
                mem += self.live[c].1;
                assert_eq!(self.o.placement(c), s(nid));
            }
            assert!(cpu <= cap.0 && mem <= cap.1);
            assert_eq!(
                self.o.node_usage(nid),
                s(&format!("{cpu}/{},{mem}/{}", cap.0, cap.1))
            );
            free.push((cap.0 - cpu, cap.1 - mem));
        }
        let mut prev: Option<(i32, u64)> = None;
        for c in self.o.pending() {
            assert!(seen.insert(c.clone()));
            assert_eq!(self.o.placement(&c), s("PENDING"));
            let (cpu, mem, prio, seq) = self.live[&c];
            if let Some((pp, ps)) = prev {
                assert!(pp > prio || (pp == prio && ps < seq), "pending order");
            }
            prev = Some((prio, seq));
            for (fc, fm) in &free {
                assert!(!(cpu <= *fc && mem <= *fm), "{c} fits a node but is pending");
            }
        }
        assert_eq!(seen, self.live.keys().cloned().collect::<HashSet<_>>());
    }
}

#[test]
fn should_keep_invariants_through_a_long_mixed_sequence() {
    let mut rnd = Lcg(12345);
    for strategy in [Strategy::FirstFit, Strategy::BestFit, Strategy::WorstFit] {
        let o = sut!(create(strategy));
        let mut m = Model {
            o: o.clone(),
            caps: HashMap::new(),
            live: HashMap::new(),
            seq: 0,
        };
        for i in 0..400 {
            let op = rnd.next(100);
            if op < 8 || m.caps.len() < 2 {
                let id = format!("n{}", rnd.next(12));
                m.add_node(&id, 200 + rnd.next(800) as u32, 200 + rnd.next(800) as u32);
            } else if op < 14 {
                m.remove_node(&format!("n{}", rnd.next(12)));
            } else if op < 22 {
                o.set_strategy([Strategy::FirstFit, Strategy::BestFit, Strategy::WorstFit][rnd.next(3) as usize]);
            } else if op < 72 {
                let prio = if rnd.next(4) == 0 {
                    100 + 50 * rnd.next(3) as i32
                } else {
                    rnd.next(60) as i32
                };
                m.submit(
                    &format!("c{i}"),
                    20 + rnd.next(300) as u32,
                    20 + rnd.next(300) as u32,
                    prio,
                );
            } else if !m.live.is_empty() {
                let mut ids: Vec<String> = m.live.keys().cloned().collect();
                ids.sort();
                let id = ids[rnd.next(ids.len() as u64) as usize].clone();
                o.remove(&id).unwrap();
                m.live.remove(&id);
            }
            m.check();
        }
    }
}

#[test]
fn should_stay_consistent_under_concurrent_submits_removals_and_node_churn() {
    let o = sut!(create(Strategy::BestFit));
    for i in 0..6 {
        o.add_node(&format!("base{i}"), 1000, 1000).unwrap();
    }
    let sizes: Mutex<HashMap<String, (u64, u64)>> = Mutex::new(HashMap::new());

    run_concurrently(16, |t| {
        if t >= 12 {
            let id = format!("extra{}", t - 12);
            for i in 0..300 {
                if i % 2 == 0 {
                    o.add_node(&id, 600, 600).unwrap();
                } else {
                    o.remove_node(&id).unwrap();
                }
            }
            return;
        }
        for i in 0..150u32 {
            let id = format!("t{t}-{i}");
            let cpu = 10 * (1 + i % 5);
            let mem = 10 * (1 + (i / 5) % 4);
            sizes.lock().unwrap().insert(id.clone(), (cpu as u64, mem as u64));
            o.submit(&id, cpu, mem, if i % 7 == 0 { 100 } else { (i % 5) as i32 })
                .unwrap();
            if i % 3 == 2 {
                let old = format!("t{t}-{}", i - 2);
                o.remove(&old).unwrap();
                sizes.lock().unwrap().remove(&old);
            }
        }
    });

    let sizes = sizes.into_inner().unwrap();
    let mut seen = HashSet::new();
    let mut ids: Vec<String> = (0..6).map(|i| format!("base{i}")).collect();
    ids.extend((0..4).map(|i| format!("extra{i}")));
    for n in ids {
        let (Ok(usage), Ok(on)) = (o.node_usage(&n), o.containers_on(&n)) else {
            continue;
        };
        let (mut cpu, mut mem) = (0u64, 0u64);
        for c in &on {
            assert!(seen.insert(c.clone()), "{c} listed twice");
            cpu += sizes[c].0;
            mem += sizes[c].1;
        }
        let p: Vec<u64> = usage.split(['/', ',']).map(|x| x.parse().unwrap()).collect();
        assert!(p[0] == cpu && cpu <= p[1]);
        assert!(p[2] == mem && mem <= p[3]);
    }
    for c in o.pending() {
        assert!(seen.insert(c.clone()), "{c} both pending and running or listed twice");
    }
    assert_eq!(seen, sizes.keys().cloned().collect::<HashSet<_>>());
}
