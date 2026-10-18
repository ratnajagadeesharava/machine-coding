#![allow(unused_imports, clippy::needless_range_loop)]
use p29_worker_pool::{contract::*, create};
use playground_support::{collect_concurrently, eventually, stays, sut};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Jobs that report when they start and then wait until released.
#[derive(Clone, Default)]
struct Gate {
    release: Arc<AtomicBool>,
    started: Arc<AtomicUsize>,
    running: Arc<AtomicUsize>,
    max_running: Arc<AtomicUsize>,
}

impl Gate {
    fn job(&self, value: i64) -> Job {
        let g = self.clone();
        Box::new(move || {
            g.started.fetch_add(1, Ordering::SeqCst);
            let now = g.running.fetch_add(1, Ordering::SeqCst) + 1;
            g.max_running.fetch_max(now, Ordering::SeqCst);
            eventually(30_000, || g.release.load(Ordering::SeqCst));
            g.running.fetch_sub(1, Ordering::SeqCst);
            value
        })
    }

    fn open(&self) {
        self.release.store(true, Ordering::SeqCst);
    }

    fn await_started(&self, n: usize) {
        assert!(
            eventually(10_000, || self.started.load(Ordering::SeqCst) >= n),
            "{n} jobs started"
        );
    }
}

fn get(rx: &Receiver<JobOutcome>) -> JobOutcome {
    rx.recv_timeout(Duration::from_secs(10))
        .expect("job outcome within 10 s")
}

#[test]
fn should_run_a_submitted_job_and_return_its_result() {
    let p = sut!(create(2, 2));

    let rx = p.submit(Box::new(|| 21 * 2)).unwrap();

    assert_eq!(get(&rx), Ok(42));
    p.shutdown();
}

#[test]
fn should_never_run_more_jobs_in_parallel_than_workers_and_reach_that_parallelism() {
    let p = sut!(create(3, 20));
    let gate = Gate::default();
    let receivers: Vec<_> = (0..10).map(|i| p.submit(gate.job(i)).unwrap()).collect();

    gate.await_started(3);
    assert!(
        stays(200, || gate.started.load(Ordering::SeqCst) == 3
            && gate.running.load(Ordering::SeqCst) == 3),
        "exactly 3 run"
    );

    gate.open();
    for (i, rx) in receivers.iter().enumerate() {
        assert_eq!(get(rx), Ok(i as i64));
    }
    assert_eq!(gate.max_running.load(Ordering::SeqCst), 3);
    p.shutdown();
}

#[test]
fn should_reject_when_the_queue_is_full_and_all_workers_are_busy() {
    let p = sut!(create(2, 3));
    let gate = Gate::default();
    let mut accepted = vec![
        p.submit(gate.job(0)).unwrap(),
        p.submit(gate.job(1)).unwrap(),
    ];
    gate.await_started(2);
    for i in 2..5 {
        accepted.push(p.submit(gate.job(i)).unwrap());
    }
    let rejected_runs = Arc::new(AtomicUsize::new(0));

    let counter = rejected_runs.clone();
    let result = p.submit(Box::new(move || {
        counter.fetch_add(1, Ordering::SeqCst);
        99
    }));
    assert!(matches!(result, Err(Rejected)));

    gate.open();
    for (i, rx) in accepted.iter().enumerate() {
        assert_eq!(get(rx), Ok(i as i64));
    }
    assert_eq!(gate.started.load(Ordering::SeqCst), 5);
    assert_eq!(rejected_runs.load(Ordering::SeqCst), 0);
    p.shutdown();
}

#[test]
fn should_accept_again_once_capacity_frees_up() {
    let p = sut!(create(1, 1));
    let gate = Gate::default();
    let first = p.submit(gate.job(1)).unwrap();
    gate.await_started(1);
    let second = p.submit(gate.job(2)).unwrap();
    assert!(p.submit(Box::new(|| 3)).is_err());

    gate.open();
    assert_eq!(get(&first), Ok(1));
    assert_eq!(get(&second), Ok(2));

    let later = Mutex::new(None);
    assert!(
        eventually(10_000, || match p.submit(Box::new(|| 4)) {
            Ok(rx) => {
                *later.lock().unwrap() = Some(rx);
                true
            }
            Err(_) => false,
        }),
        "a new job is accepted after capacity frees up"
    );
    let rx = later.lock().unwrap().take().unwrap();
    assert_eq!(get(&rx), Ok(4));
    p.shutdown();
}

#[test]
fn should_start_queued_jobs_in_submission_order() {
    let p = sut!(create(1, 10));
    let gate = Gate::default();
    let blocker = p.submit(gate.job(-1)).unwrap();
    gate.await_started(1);
    let order = Arc::new(Mutex::new(Vec::new()));
    let receivers: Vec<_> = (0..10)
        .map(|i| {
            let order = order.clone();
            p.submit(Box::new(move || {
                order.lock().unwrap().push(i);
                i
            }))
            .unwrap()
        })
        .collect();

    gate.open();
    get(&blocker).unwrap();
    for rx in &receivers {
        get(rx).unwrap();
    }

    assert_eq!(*order.lock().unwrap(), (0..10).collect::<Vec<i64>>());
    p.shutdown();
}

#[test]
fn should_report_a_job_failure_to_its_caller_and_keep_the_worker_alive() {
    let p = sut!(create(1, 10));
    let failing: Vec<_> = (0..5)
        .map(|_| p.submit(Box::new(|| panic!("boom"))).unwrap())
        .collect();
    let good = p.submit(Box::new(|| 7)).unwrap();

    for rx in &failing {
        let outcome = get(rx);
        assert!(
            matches!(&outcome, Err(msg) if msg.contains("boom")),
            "unexpected outcome {outcome:?}"
        );
    }
    assert_eq!(get(&good), Ok(7));
    p.shutdown();
}

#[test]
fn should_reject_new_jobs_after_shutdown() {
    let p = sut!(create(2, 2));

    p.shutdown();

    assert!(p.submit(Box::new(|| 1)).is_err());
}

#[test]
fn should_complete_running_and_queued_jobs_on_shutdown() {
    let p = sut!(create(1, 5));
    let gate = Gate::default();
    let mut receivers = vec![p.submit(gate.job(0)).unwrap()];
    gate.await_started(1);
    for i in 1..=3 {
        receivers.push(p.submit(gate.job(i)).unwrap());
    }

    p.shutdown();

    assert!(p.submit(Box::new(|| 9)).is_err());
    assert!(!p.await_termination(100));
    gate.open();
    assert!(p.await_termination(10_000));
    for (i, rx) in receivers.iter().enumerate() {
        assert_eq!(get(rx), Ok(i as i64));
    }
}

#[test]
fn should_tolerate_repeated_shutdown() {
    let p = sut!(create(2, 2));
    let rx = p.submit(Box::new(|| 5)).unwrap();

    p.shutdown();
    p.shutdown();

    assert_eq!(get(&rx), Ok(5));
    assert!(p.await_termination(10_000));
    p.shutdown();
    assert!(p.await_termination(10));
}

#[test]
fn should_not_terminate_without_shutdown() {
    let p = sut!(create(2, 2));

    let t0 = Instant::now();
    let terminated = p.await_termination(100);
    let elapsed = t0.elapsed().as_millis();

    assert!(!terminated);
    assert!((80..4_000).contains(&elapsed), "elapsed {elapsed} ms");
    assert_eq!(get(&p.submit(Box::new(|| 1)).unwrap()), Ok(1));
    p.shutdown();
}

#[test]
fn should_deliver_each_result_to_the_right_caller_when_many_threads_submit() {
    let p = sut!(create(4, 1_000));
    let runs = Arc::new(AtomicUsize::new(0));

    let per_thread = collect_concurrently(8, |t| {
        (0..100)
            .map(|i| {
                let value = t as i64 * 1_000 + i;
                let runs = runs.clone();
                p.submit(Box::new(move || {
                    runs.fetch_add(1, Ordering::SeqCst);
                    value * 2
                }))
                .unwrap()
            })
            .collect::<Vec<_>>()
    });

    for (t, receivers) in per_thread.iter().enumerate() {
        for (i, rx) in receivers.iter().enumerate() {
            assert_eq!(get(rx), Ok((t as i64 * 1_000 + i as i64) * 2));
        }
    }
    assert_eq!(runs.load(Ordering::SeqCst), 800);
    p.shutdown();
    assert!(p.await_termination(10_000));
}
