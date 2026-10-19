#![allow(unused_imports, clippy::needless_range_loop)]
use p78_connection_pool::{contract::*, create};
use playground_support::{eventually, run_concurrently, stays, sut};
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

type Slot<T> = Arc<Mutex<Option<T>>>;

#[test]
fn should_hand_out_each_connection_once_and_then_run_dry() {
    let p = sut!(create(3));

    let ids: HashSet<usize> = (0..3).map(|_| p.acquire(0).unwrap().unwrap()).collect();

    assert_eq!(ids, HashSet::from([0, 1, 2]));
    assert_eq!(p.acquire(0), Ok(None));
    assert_eq!(p.available(), 0);
    assert_eq!(p.in_use(), 3);
}

#[test]
fn should_reuse_a_released_connection() {
    let p = sut!(create(1));
    let id = p.acquire(0).unwrap().unwrap();

    p.release(id).unwrap();

    assert_eq!(p.available(), 1);
    assert_eq!(p.in_use(), 0);
    assert_eq!(p.acquire(0), Ok(Some(id)));
}

#[test]
fn should_give_up_after_the_timeout_when_nothing_is_free() {
    let p = sut!(create(1));
    p.acquire(0).unwrap();

    let start = Instant::now();
    let got = p.acquire(200);
    let elapsed = start.elapsed().as_millis();

    assert_eq!(got, Ok(None));
    assert!(
        (180..=1_500).contains(&elapsed),
        "must wait roughly the timeout, waited {elapsed}ms"
    );
}

#[test]
fn should_wake_a_waiter_promptly_when_a_connection_is_released() {
    let p = sut!(create(1));
    let id = p.acquire(0).unwrap().unwrap();
    let got: Slot<Option<usize>> = Arc::new(Mutex::new(None));
    {
        let (p, got) = (p.clone(), got.clone());
        thread::spawn(move || {
            let r = p.acquire(10_000).unwrap();
            *got.lock().unwrap() = Some(r);
        });
    }
    assert!(
        stays(100, || got.lock().unwrap().is_none()),
        "the waiter must block while the pool is empty"
    );

    p.release(id).unwrap();

    assert!(
        eventually(2_000, || got.lock().unwrap().is_some()),
        "waiter should be woken long before its timeout"
    );
    assert_eq!(*got.lock().unwrap(), Some(Some(id)));
}

#[test]
fn should_reject_bad_releases() {
    let p = sut!(create(2));
    let id = p.acquire(0).unwrap().unwrap();
    p.release(id).unwrap();

    assert_eq!(p.release(id), Err(ReleaseError::NotAcquired));
    assert_eq!(p.release(1 - id), Err(ReleaseError::NotAcquired));
    assert_eq!(p.release(5), Err(ReleaseError::UnknownConnection));
    assert_eq!(p.available(), 2);
}

#[test]
fn should_never_hand_the_same_connection_to_two_holders() {
    let size = 4;
    let p = sut!(create(size));
    let held: Vec<AtomicUsize> = (0..size).map(|_| AtomicUsize::new(0)).collect();
    let violations = AtomicUsize::new(0);
    let holders = AtomicUsize::new(0);
    let max_holders = AtomicUsize::new(0);
    let uses = AtomicUsize::new(0);

    run_concurrently(16, |_| {
        for _ in 0..500 {
            let id = p.acquire(10_000).unwrap().expect("timed out waiting");
            if held[id]
                .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
            {
                violations.fetch_add(1, Ordering::SeqCst);
            }
            max_holders.fetch_max(holders.fetch_add(1, Ordering::SeqCst) + 1, Ordering::SeqCst);
            uses.fetch_add(1, Ordering::SeqCst);
            thread::yield_now();
            holders.fetch_sub(1, Ordering::SeqCst);
            held[id].store(0, Ordering::SeqCst);
            p.release(id).unwrap();
        }
    });

    assert_eq!(violations.load(Ordering::SeqCst), 0);
    assert!(max_holders.load(Ordering::SeqCst) <= size);
    assert_eq!(uses.load(Ordering::SeqCst), 16 * 500);
    assert_eq!(p.available(), size);
    assert_eq!(p.in_use(), 0);
}

#[test]
fn should_not_lose_connections_when_waiters_time_out() {
    let p = sut!(create(2));
    let a = p.acquire(0).unwrap().unwrap();
    let b = p.acquire(0).unwrap().unwrap();

    run_concurrently(20, |_| {
        assert_eq!(p.acquire(20), Ok(None));
    });

    p.release(a).unwrap();
    p.release(b).unwrap();
    assert_eq!(p.available(), 2);
    assert!(p.acquire(0).unwrap().is_some());
    assert!(p.acquire(0).unwrap().is_some());
    assert_eq!(p.acquire(0), Ok(None));
}

#[test]
fn should_not_lose_connections_when_timeouts_race_with_releases() {
    let size = 3;
    let p = sut!(create(size));
    let acquired = AtomicUsize::new(0);

    run_concurrently(12, |_| {
        for _ in 0..400 {
            // tiny timeout: expires right when releases happen
            if let Some(id) = p.acquire(1).unwrap() {
                acquired.fetch_add(1, Ordering::SeqCst);
                thread::yield_now();
                p.release(id).unwrap();
            }
        }
    });

    assert!(acquired.load(Ordering::SeqCst) > 0);
    assert_eq!(p.available(), size, "a connection was lost or duplicated");
    assert_eq!(p.in_use(), 0);
}

#[test]
fn should_pass_a_single_connection_between_many_threads() {
    let p = sut!(create(1));
    let uses = AtomicUsize::new(0);

    run_concurrently(8, |_| {
        for _ in 0..300 {
            let id = p.acquire(20_000).unwrap().expect("starved or lost wake-up");
            uses.fetch_add(1, Ordering::SeqCst);
            p.release(id).unwrap();
        }
    });

    assert_eq!(uses.load(Ordering::SeqCst), 8 * 300);
}

#[test]
fn should_fail_waiters_and_new_callers_after_close() {
    let p = sut!(create(1));
    let id = p.acquire(0).unwrap().unwrap();
    let outcome: Slot<Result<Option<usize>, Closed>> = Arc::new(Mutex::new(None));
    {
        let (p, outcome) = (p.clone(), outcome.clone());
        thread::spawn(move || {
            let r = p.acquire(10_000);
            *outcome.lock().unwrap() = Some(r);
        });
    }
    assert!(
        stays(100, || outcome.lock().unwrap().is_none()),
        "waiter blocks first"
    );

    p.close();
    p.close();

    assert!(
        eventually(2_000, || outcome.lock().unwrap().is_some()),
        "close wakes the waiter"
    );
    assert_eq!(*outcome.lock().unwrap(), Some(Err(Closed)));
    assert_eq!(p.acquire(0), Err(Closed));
    p.release(id).unwrap(); // returning a connection after close is fine
    assert_eq!(p.in_use(), 0);
}

#[test]
fn should_stay_correct_over_100_runs() {
    for run in 0..100 {
        let size = 1 + run % 4;
        let p = sut!(create(size));
        let violations = AtomicUsize::new(0);
        let held: Vec<AtomicUsize> = (0..size).map(|_| AtomicUsize::new(0)).collect();

        run_concurrently(8, |_| {
            for i in 0..100 {
                let Some(id) = p.acquire(if i % 3 == 0 { 1 } else { 5_000 }).unwrap() else {
                    continue;
                };
                if held[id]
                    .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
                    .is_err()
                {
                    violations.fetch_add(1, Ordering::SeqCst);
                }
                held[id].store(0, Ordering::SeqCst);
                p.release(id).unwrap();
            }
        });

        assert_eq!(violations.load(Ordering::SeqCst), 0, "run {run}");
        assert_eq!(p.available(), size, "run {run}");
    }
}
