#![allow(unused_imports, clippy::needless_range_loop)]
use p29_worker_pool::{contract::*, create};
use playground_support::{eventually, sut};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::time::Duration;

fn get(rx: &Receiver<JobOutcome>) -> JobOutcome {
    rx.recv_timeout(Duration::from_secs(10))
        .expect("job outcome within 10 s")
}

fn waiting_job(release: &Arc<AtomicBool>, started: &Arc<AtomicUsize>, value: i64) -> Job {
    let (release, started) = (release.clone(), started.clone());
    Box::new(move || {
        started.fetch_add(1, Ordering::SeqCst);
        eventually(30_000, || release.load(Ordering::SeqCst));
        value
    })
}

#[test]
fn zero_queue_capacity_should_accept_only_as_many_jobs_as_there_are_free_workers() {
    let p = sut!(create(2, 0));
    let release = Arc::new(AtomicBool::new(false));
    let started = Arc::new(AtomicUsize::new(0));

    let a = p.submit(waiting_job(&release, &started, 1)).unwrap();
    let b = p.submit(waiting_job(&release, &started, 2)).unwrap();
    assert!(eventually(10_000, || started.load(Ordering::SeqCst) == 2));

    assert!(p.submit(Box::new(|| 3)).is_err());
    release.store(true, Ordering::SeqCst);
    assert_eq!(get(&a), Ok(1));
    assert_eq!(get(&b), Ok(2));
    p.shutdown();
}

#[test]
fn should_keep_full_parallelism_after_many_failing_jobs() {
    let p = sut!(create(3, 100));
    let outcomes: Vec<_> = (0..30)
        .map(|i| {
            if i % 2 == 0 {
                p.submit(Box::new(|| panic!("bad"))).unwrap()
            } else {
                p.submit(Box::new(|| 1)).unwrap()
            }
        })
        .collect();
    for (i, rx) in outcomes.iter().enumerate() {
        if i % 2 == 0 {
            assert!(get(rx).is_err());
        } else {
            assert_eq!(get(rx), Ok(1));
        }
    }

    let arrived = Arc::new(AtomicUsize::new(0));
    let meeting: Vec<_> = (0..3)
        .map(|_| {
            let arrived = arrived.clone();
            p.submit(Box::new(move || {
                arrived.fetch_add(1, Ordering::SeqCst);
                i64::from(eventually(10_000, || arrived.load(Ordering::SeqCst) >= 3))
            }))
            .unwrap()
        })
        .collect();

    for rx in &meeting {
        assert_eq!(get(rx), Ok(1), "three workers must still run in parallel");
    }
    p.shutdown();
}

#[test]
fn a_job_should_be_able_to_submit_further_jobs_to_the_same_pool() {
    let p = sut!(create(1, 1));
    let (tx, inner) = channel::<Receiver<JobOutcome>>();
    let pool = p.clone();

    let outer = p
        .submit(Box::new(move || {
            if let Ok(rx) = pool.submit(Box::new(|| 10)) {
                tx.send(rx).unwrap();
            }
            1
        }))
        .unwrap();

    assert_eq!(get(&outer), Ok(1));
    let inner = inner
        .recv_timeout(Duration::from_secs(10))
        .expect("inner job accepted");
    assert_eq!(get(&inner), Ok(10));
    p.shutdown();
    assert!(p.await_termination(10_000));
}

#[test]
fn shutdown_on_an_idle_pool_should_terminate_immediately_and_reject_later_jobs() {
    let p = sut!(create(3, 3));

    p.shutdown();

    assert!(p.await_termination(10_000));
    assert!(p.submit(Box::new(|| 1)).is_err());
}

#[test]
fn should_run_accepted_queued_jobs_even_when_shutdown_is_called_before_any_worker_frees_up() {
    let p = sut!(create(2, 10));
    let release = Arc::new(AtomicBool::new(false));
    let started = Arc::new(AtomicUsize::new(0));
    let receivers: Vec<_> = (0..12)
        .map(|i| p.submit(waiting_job(&release, &started, i)).unwrap())
        .collect();
    assert!(p.submit(Box::new(|| 0)).is_err());

    p.shutdown();
    release.store(true, Ordering::SeqCst);

    assert!(p.await_termination(10_000));
    for (i, rx) in receivers.iter().enumerate() {
        assert_eq!(get(rx), Ok(i as i64));
    }
}
