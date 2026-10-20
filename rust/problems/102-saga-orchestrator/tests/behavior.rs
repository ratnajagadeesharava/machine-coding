#![allow(unused_imports, clippy::needless_range_loop)]
use p102_saga_orchestrator::{contract::*, create};
use playground_support::{collect_concurrently, skipped};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

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

fn journal() -> Journal {
    Arc::new(Mutex::new(Vec::new()))
}

fn entries(j: &Journal) -> Vec<String> {
    j.lock().unwrap().clone()
}

/// Scripted step: fails its first `exec_fails` executes and first `comp_fails` compensations;
/// every call is recorded into the shared journal ("exec:name", "comp:name").
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
    fn execute(&self, _saga_id: &str) -> bool {
        self.journal
            .lock()
            .unwrap()
            .push(format!("exec:{}", self.name));
        self.execs.fetch_add(1, Ordering::SeqCst) >= self.exec_fails
    }
    fn compensate(&self, _saga_id: &str) -> bool {
        self.journal
            .lock()
            .unwrap()
            .push(format!("comp:{}", self.name));
        self.comps.fetch_add(1, Ordering::SeqCst) >= self.comp_fails
    }
}

fn step(name: &str, exec_fails: usize, comp_fails: usize, j: &Journal) -> Arc<dyn SagaStep> {
    Arc::new(Scripted {
        name: name.to_string(),
        exec_fails,
        comp_fails,
        execs: AtomicUsize::new(0),
        comps: AtomicUsize::new(0),
        journal: j.clone(),
    })
}

fn good(name: &str, j: &Journal) -> Arc<dyn SagaStep> {
    step(name, 0, 0, j)
}

fn failing(name: &str, j: &Journal) -> Arc<dyn SagaStep> {
    step(name, usize::MAX, 0, j)
}

type Sleeps = Arc<Mutex<Vec<u64>>>;

fn setup_with(log: Log) -> Option<(Arc<dyn SagaOrchestrator>, Sleeps)> {
    let sleeps: Sleeps = Arc::new(Mutex::new(Vec::new()));
    let s = sleeps.clone();
    let o = create(log, Arc::new(move |ms| s.lock().unwrap().push(ms)))?;
    Some((o, sleeps))
}

fn setup() -> Option<(Arc<dyn SagaOrchestrator>, Sleeps)> {
    setup_with(Arc::new(MemoryLog::default()))
}

macro_rules! sut {
    ($e:expr) => {
        match $e {
            Some(v) => v,
            None => return skipped(),
        }
    };
}

fn sleeps_of(s: &Sleeps) -> Vec<u64> {
    s.lock().unwrap().clone()
}

fn count(j: &Journal, what: &str) -> usize {
    entries(j).iter().filter(|e| *e == what).count()
}

#[test]
fn should_run_all_steps_in_order_and_complete() {
    let (o, _) = sut!(setup());
    let j = journal();

    let result = o.run(
        "s1",
        &[good("reserve", &j), good("charge", &j), good("ship", &j)],
    );

    assert_eq!(result, SagaStatus::Completed);
    assert_eq!(entries(&j), ["exec:reserve", "exec:charge", "exec:ship"]);
    assert_eq!(o.status("s1"), SagaStatus::Completed);
}

#[test]
fn should_complete_an_empty_saga() {
    let (o, _) = sut!(setup());

    assert_eq!(o.run("empty", &[]), SagaStatus::Completed);
}

#[test]
fn should_report_not_found_for_an_unknown_saga() {
    let (o, _) = sut!(setup());

    assert_eq!(o.status("nope"), SagaStatus::NotFound);
}

#[test]
fn should_compensate_completed_steps_in_reverse_order_when_a_step_fails() {
    let (o, _) = sut!(setup());
    let j = journal();

    let result = o.run(
        "s1",
        &[
            good("reserve", &j),
            good("charge", &j),
            failing("ship", &j),
            good("notify", &j),
        ],
    );

    assert_eq!(result, SagaStatus::Compensated);
    assert_eq!(
        entries(&j),
        [
            "exec:reserve",
            "exec:charge",
            "exec:ship",
            "exec:ship",
            "exec:ship",
            "comp:charge",
            "comp:reserve"
        ]
    );
    assert_eq!(o.status("s1"), SagaStatus::Compensated);
}

#[test]
fn should_compensate_nothing_when_the_first_step_fails() {
    let (o, _) = sut!(setup());
    let j = journal();

    let result = o.run("s1", &[failing("reserve", &j), good("charge", &j)]);

    assert_eq!(result, SagaStatus::Compensated);
    assert!(entries(&j).iter().all(|e| e == "exec:reserve"));
}

#[test]
fn should_retry_a_failing_step_with_backoff_and_succeed_on_the_third_attempt() {
    let (o, sleeps) = sut!(setup());
    let j = journal();

    let result = o.run("s1", &[good("a", &j), step("flaky", 2, 0, &j)]);

    assert_eq!(result, SagaStatus::Completed);
    assert_eq!(
        entries(&j),
        ["exec:a", "exec:flaky", "exec:flaky", "exec:flaky"]
    );
    assert_eq!(sleeps_of(&sleeps), [100, 200]);
}

#[test]
fn should_not_sleep_after_the_last_failed_attempt() {
    let (o, sleeps) = sut!(setup());
    let j = journal();

    o.run("s1", &[failing("a", &j)]);

    assert_eq!(sleeps_of(&sleeps), [100, 200]);
}

#[test]
fn should_retry_compensations_up_to_five_times() {
    let (o, sleeps) = sut!(setup());
    let j = journal();

    let result = o.run(
        "s1",
        &[
            good("reserve", &j),
            step("charge", 0, 3, &j),
            failing("ship", &j),
        ],
    );

    assert_eq!(result, SagaStatus::Compensated);
    assert_eq!(count(&j, "comp:charge"), 4);
    assert_eq!(sleeps_of(&sleeps), [100, 200, 100, 200, 400]);
    assert!(entries(&j).ends_with(&["comp:charge".to_string(), "comp:reserve".to_string()]));
}

#[test]
fn should_stop_with_compensation_failed_and_not_compensate_the_rest() {
    let (o, sleeps) = sut!(setup());
    let j = journal();

    let result = o.run(
        "s1",
        &[
            good("reserve", &j),
            step("charge", 0, usize::MAX, &j),
            failing("ship", &j),
        ],
    );

    assert_eq!(result, SagaStatus::CompensationFailed);
    assert_eq!(count(&j, "comp:charge"), 5);
    assert_eq!(count(&j, "comp:reserve"), 0);
    assert_eq!(sleeps_of(&sleeps)[2..], [100, 200, 400, 800]);
    assert_eq!(o.status("s1"), SagaStatus::CompensationFailed);
}

#[test]
fn should_not_run_a_finished_saga_again() {
    let (o, _) = sut!(setup());
    let j = journal();
    let steps = [good("a", &j), good("b", &j)];
    o.run("s1", &steps);
    j.lock().unwrap().clear();

    assert_eq!(o.run("s1", &steps), SagaStatus::Completed);
    assert!(entries(&j).is_empty());
}

#[test]
fn should_keep_sagas_independent() {
    let (o, _) = sut!(setup());
    let j = journal();

    assert_eq!(o.run("ok", &[good("a", &j)]), SagaStatus::Completed);
    assert_eq!(o.run("bad", &[failing("a", &j)]), SagaStatus::Compensated);
    assert_eq!(o.status("ok"), SagaStatus::Completed);
    assert_eq!(o.status("bad"), SagaStatus::Compensated);
}

#[test]
fn should_recover_the_status_from_the_log_after_a_restart() {
    let log: Log = Arc::new(MemoryLog::default());
    let (first, _) = sut!(setup_with(log.clone()));
    let j = journal();
    first.run("done", &[good("a", &j)]);
    first.run("undone", &[good("a", &j), failing("b", &j)]);

    let (second, _) = setup_with(log).unwrap();

    assert_eq!(second.status("done"), SagaStatus::Completed);
    assert_eq!(second.status("undone"), SagaStatus::Compensated);
    assert_eq!(second.status("other"), SagaStatus::NotFound);
}

fn log_with(records: &[String]) -> Log {
    let l = MemoryLog::default();
    for r in records {
        l.append(r);
    }
    Arc::new(l)
}

#[test]
fn should_continue_a_crashed_saga_without_repeating_finished_steps() {
    let full: Log = Arc::new(MemoryLog::default());
    let (o0, _) = sut!(setup_with(full.clone()));
    let j0 = journal();
    o0.run("s1", &[good("a", &j0), good("b", &j0), good("c", &j0)]);
    let all = full.records();

    for k in 0..=all.len() {
        let j = journal();
        let (o, _) = setup_with(log_with(&all[..k])).unwrap();

        let result = o.run("s1", &[good("a", &j), good("b", &j), good("c", &j)]);

        assert_eq!(result, SagaStatus::Completed, "crash after {k} writes");
        let e = entries(&j);
        let mut dedup = e.clone();
        dedup.sort();
        dedup.dedup();
        assert_eq!(
            dedup.len(),
            e.len(),
            "crash after {k} writes: a step ran twice"
        );
        assert!(
            e.iter().all(|x| x.starts_with("exec:")),
            "crash after {k} writes: compensated a healthy saga"
        );
    }
}

#[test]
fn should_continue_a_crashed_compensation_in_reverse_order() {
    let full: Log = Arc::new(MemoryLog::default());
    let (o0, _) = sut!(setup_with(full.clone()));
    let j0 = journal();
    o0.run(
        "s1",
        &[
            good("a", &j0),
            good("b", &j0),
            good("c", &j0),
            failing("d", &j0),
        ],
    );
    let all = full.records();

    for k in 0..=all.len() {
        let j = journal();
        let (o, _) = setup_with(log_with(&all[..k])).unwrap();

        let result = o.run(
            "s1",
            &[
                good("a", &j),
                good("b", &j),
                good("c", &j),
                failing("d", &j),
            ],
        );

        assert_eq!(result, SagaStatus::Compensated, "crash after {k} writes");
        assert_eq!(
            count(&j, "comp:d"),
            0,
            "the failed step is never compensated"
        );
        let comps: Vec<String> = entries(&j)
            .into_iter()
            .filter(|e| e.starts_with("comp:"))
            .collect();
        let mut sorted = comps.clone();
        sorted.sort();
        sorted.reverse();
        assert_eq!(
            comps, sorted,
            "crash after {k} writes: compensations out of order"
        );
    }
}

#[test]
fn should_run_many_sagas_concurrently_without_mixing_them_up() {
    let (o, _) = sut!(setup());
    let results = collect_concurrently(8, |t| {
        let mut out = Vec::new();
        for i in 0..50 {
            let id = format!("t{t}-{i}");
            let j = journal();
            let doomed = i % 3 == 0;
            let last = if doomed {
                failing("last", &j)
            } else {
                good("last", &j)
            };
            let status = o.run(&id, &[good("a", &j), good("b", &j), last]);
            out.push((id, doomed, status, entries(&j)));
        }
        out
    });

    let all: Vec<_> = results.into_iter().flatten().collect();
    assert_eq!(all.len(), 400);
    for (id, doomed, status, e) in all {
        if doomed {
            assert_eq!(status, SagaStatus::Compensated, "{id}");
            assert_eq!(
                e,
                [
                    "exec:a",
                    "exec:b",
                    "exec:last",
                    "exec:last",
                    "exec:last",
                    "comp:b",
                    "comp:a"
                ],
                "{id}"
            );
        } else {
            assert_eq!(status, SagaStatus::Completed, "{id}");
            assert_eq!(e, ["exec:a", "exec:b", "exec:last"], "{id}");
        }
        assert_eq!(o.status(&id), status);
    }
}

#[test]
fn should_return_running_for_a_saga_that_another_thread_is_running() {
    let (o, _) = sut!(setup());
    let (inside_tx, inside_rx) = mpsc::channel::<()>();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    struct Slow {
        inside: Mutex<mpsc::Sender<()>>,
        release: Mutex<mpsc::Receiver<()>>,
    }
    impl SagaStep for Slow {
        fn name(&self) -> String {
            "slow".into()
        }
        fn execute(&self, _: &str) -> bool {
            self.inside.lock().unwrap().send(()).ok();
            self.release.lock().unwrap().recv().ok();
            true
        }
        fn compensate(&self, _: &str) -> bool {
            true
        }
    }
    let slow: Arc<dyn SagaStep> = Arc::new(Slow {
        inside: Mutex::new(inside_tx),
        release: Mutex::new(release_rx),
    });

    thread::scope(|s| {
        let o2 = o.clone();
        let slow2 = slow.clone();
        let worker = s.spawn(move || o2.run("s1", &[slow2]));
        inside_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("the saga never started");

        assert_eq!(o.status("s1"), SagaStatus::Running);
        assert_eq!(
            o.run("s1", std::slice::from_ref(&slow)),
            SagaStatus::Running
        );
        release_tx.send(()).unwrap();
        assert_eq!(worker.join().unwrap(), SagaStatus::Completed);
    });
    assert_eq!(o.status("s1"), SagaStatus::Completed);
}
