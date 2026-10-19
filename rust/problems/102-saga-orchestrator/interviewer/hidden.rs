#![allow(unused_imports)]
use p102_saga_orchestrator::{contract::*, create};
use playground_support::{run_concurrently, skipped, Rng};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct MemoryLog(Mutex<Vec<String>>);
impl SagaLog for MemoryLog {
    fn append(&self, record: &str) {
        self.0.lock().unwrap().push(record.to_string());
    }
    fn records(&self) -> Vec<String> {
        self.0.lock().unwrap().clone()
    }
}

type Journal = Arc<Mutex<Vec<String>>>;

struct Scripted {
    name: String,
    exec_fails: usize,
    comp_fails: usize,
    execs: AtomicUsize,
    comps: AtomicUsize,
    journal: Journal,
}
impl SagaStep for Scripted {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn execute(&self, _: &str) -> bool {
        self.journal
            .lock()
            .unwrap()
            .push(format!("exec:{}", self.name));
        self.execs.fetch_add(1, Ordering::SeqCst) >= self.exec_fails
    }
    fn compensate(&self, _: &str) -> bool {
        self.journal
            .lock()
            .unwrap()
            .push(format!("comp:{}", self.name));
        self.comps.fetch_add(1, Ordering::SeqCst) >= self.comp_fails
    }
}

fn build(exec_fails: &[usize], comp_fails: &[usize], j: &Journal) -> Vec<Arc<dyn SagaStep>> {
    (0..exec_fails.len())
        .map(|i| {
            Arc::new(Scripted {
                name: format!("s{i}"),
                exec_fails: exec_fails[i],
                comp_fails: comp_fails[i],
                execs: AtomicUsize::new(0),
                comps: AtomicUsize::new(0),
                journal: j.clone(),
            }) as Arc<dyn SagaStep>
        })
        .collect()
}

fn orch(log: Log) -> Option<Arc<dyn SagaOrchestrator>> {
    create(log, Arc::new(|_| {}))
}

#[test]
fn should_survive_a_crash_at_every_point_of_random_sagas() {
    for seed in 0..60u64 {
        let mut rng = Rng::new(seed + 1);
        let n = 2 + rng.below(4) as usize;
        let mut exec_fails = vec![0; n];
        let mut comp_fails = vec![0; n];
        for i in 0..n {
            exec_fails[i] = if rng.below(5) == 0 {
                3
            } else {
                rng.below(3) as usize
            };
            comp_fails[i] = if rng.below(6) == 0 {
                5
            } else {
                rng.below(3) as usize
            };
        }
        let full: Log = Arc::new(MemoryLog::default());
        let Some(o0) = orch(full.clone()) else {
            return skipped();
        };
        let j0: Journal = Arc::default();
        let expected = o0.run("s", &build(&exec_fails, &comp_fails, &j0));
        let all = full.records();

        for k in 0..=all.len() {
            let l = MemoryLog::default();
            for r in &all[..k] {
                l.append(r);
            }
            let o = orch(Arc::new(l)).unwrap();
            let j: Journal = Arc::default();
            let got = o.run("s", &build(&exec_fails, &comp_fails, &j));
            assert_eq!(got, expected, "seed {seed} crash after {k}");
            assert_eq!(o.status("s"), expected);
            let e = j.lock().unwrap().clone();
            if k == all.len() {
                assert!(e.is_empty(), "a finished saga must not run again");
            }
            let comps: Vec<String> = e.into_iter().filter(|x| x.starts_with("comp:")).collect();
            let mut sorted = comps.clone();
            sorted.sort();
            sorted.reverse();
            assert_eq!(comps, sorted, "reverse order");
        }
    }
}

#[test]
fn should_run_the_same_saga_only_once_when_many_threads_race_for_it() {
    let Some(o) = orch(Arc::new(MemoryLog::default())) else {
        return skipped();
    };
    for round in 0..100 {
        let id = format!("race{round}");
        let j: Journal = Arc::default();
        let steps = build(&[0, 0], &[0, 0], &j);
        run_concurrently(8, |_| {
            o.run(&id, &steps);
        });
        assert_eq!(*j.lock().unwrap(), ["exec:s0", "exec:s1"], "{id}");
        assert_eq!(o.status(&id), SagaStatus::Completed);
    }
}
