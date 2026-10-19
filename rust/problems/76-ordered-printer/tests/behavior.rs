#![allow(unused_imports, clippy::needless_range_loop)]
use p76_ordered_printer::{contract::*, create};
use playground_support::{eventually, run_concurrently, stays, sut};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

fn run_rounds(p: &Arc<dyn OrderedPrinter>, workers: usize, rounds: usize) -> Vec<usize> {
    let log = Mutex::new(Vec::new());
    run_concurrently(workers, |id| {
        for _ in 0..rounds {
            p.take_turn(id, &mut || log.lock().unwrap().push(id))
                .unwrap();
        }
    });
    log.into_inner().unwrap()
}

fn expected(workers: usize, rounds: usize) -> Vec<usize> {
    (0..rounds).flat_map(|_| 0..workers).collect()
}

#[test]
fn should_run_workers_in_strict_round_robin() {
    let p = sut!(create(3));

    let log = run_rounds(&p, 3, 50);

    assert_eq!(log, expected(3, 50));
}

#[test]
fn should_work_with_a_single_worker() {
    let p = sut!(create(1));

    let log = run_rounds(&p, 1, 100);

    assert_eq!(log, vec![0; 100]);
}

#[test]
fn should_work_with_many_workers() {
    let p = sut!(create(16));

    let log = run_rounds(&p, 16, 30);

    assert_eq!(log, expected(16, 30));
}

#[test]
fn should_keep_order_when_workers_arrive_in_reverse() {
    let p = sut!(create(4));
    let log = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();
    for id in (0..4).rev() {
        let (p, log) = (p.clone(), log.clone());
        handles.push(thread::spawn(move || {
            p.take_turn(id, &mut || log.lock().unwrap().push(id))
                .unwrap();
        }));
        thread::sleep(std::time::Duration::from_millis(30));
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(*log.lock().unwrap(), vec![0, 1, 2, 3]);
}

#[test]
fn should_block_an_out_of_turn_worker_until_its_turn() {
    let p = sut!(create(2));
    let ran = Arc::new(AtomicBool::new(false));
    {
        let (p, ran) = (p.clone(), ran.clone());
        thread::spawn(move || {
            p.take_turn(1, &mut || ran.store(true, Ordering::SeqCst))
                .unwrap();
        });
    }

    assert!(
        stays(150, || !ran.load(Ordering::SeqCst)),
        "worker 1 must wait for worker 0"
    );
    p.take_turn(0, &mut || {}).unwrap();

    assert!(
        eventually(5_000, || ran.load(Ordering::SeqCst)),
        "worker 1 runs once worker 0 is done"
    );
}

#[test]
fn should_never_run_two_actions_at_the_same_time() {
    let workers = 6;
    let p = sut!(create(workers));
    let inside = AtomicUsize::new(0);
    let max_inside = AtomicUsize::new(0);

    run_concurrently(workers, |id| {
        for _ in 0..100 {
            p.take_turn(id, &mut || {
                let now = inside.fetch_add(1, Ordering::SeqCst) + 1;
                max_inside.fetch_max(now, Ordering::SeqCst);
                thread::yield_now();
                inside.fetch_sub(1, Ordering::SeqCst);
            })
            .unwrap();
        }
    });

    assert_eq!(max_inside.load(Ordering::SeqCst), 1);
}

#[test]
fn should_hand_the_turn_over_even_when_the_action_panics() {
    let p = sut!(create(2));

    let result = catch_unwind(AssertUnwindSafe(|| {
        let _ = p.take_turn(0, &mut || panic!("boom"));
    }));
    assert!(result.is_err(), "the panic propagates to the caller");

    let ran = Arc::new(AtomicBool::new(false));
    let handle = {
        let (p, ran) = (p.clone(), ran.clone());
        thread::spawn(move || {
            p.take_turn(1, &mut || ran.store(true, Ordering::SeqCst))
                .unwrap();
        })
    };
    assert!(
        eventually(5_000, || ran.load(Ordering::SeqCst)),
        "worker 1 must not be stuck behind the panicked turn"
    );
    handle.join().unwrap();
}

#[test]
fn should_reject_unknown_workers() {
    let p = sut!(create(3));

    assert_eq!(p.take_turn(3, &mut || {}), Err(UnknownWorker));
    assert_eq!(p.take_turn(99, &mut || {}), Err(UnknownWorker));
}

#[test]
fn should_stay_correct_over_100_runs() {
    for run in 0..100 {
        let workers = 2 + run % 6;
        let p = sut!(create(workers));

        let log = run_rounds(&p, workers, 20);

        assert_eq!(
            log,
            expected(workers, 20),
            "run {run} with {workers} workers"
        );
    }
}
