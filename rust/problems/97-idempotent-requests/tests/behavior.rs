#![allow(unused_imports, clippy::needless_range_loop)]
use p97_idempotent_requests::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut, FakeClock};
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const HOUR: u64 = 3_600_000;

fn setup() -> Option<(Arc<dyn IdempotentProcessor>, FakeClock)> {
    let clock = FakeClock::new(0);
    Some((create(clock.as_clock())?, clock))
}

#[test]
fn should_run_the_handler_once_and_replay_the_stored_response() {
    let (p, _) = sut!(setup());
    let calls = AtomicUsize::new(0);
    let h = |body: &str| {
        Ok(format!(
            "done:{body}:{}",
            calls.fetch_add(1, Ordering::SeqCst) + 1
        ))
    };

    let first = p.process("key-1", "pay 10", &h);
    let second = p.process("key-1", "pay 10", &h);

    assert_eq!(first, Ok("done:pay 10:1".to_string()));
    assert_eq!(second, first);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn should_treat_different_keys_independently() {
    let (p, _) = sut!(setup());
    let calls = AtomicUsize::new(0);
    let h = |_: &str| Ok(format!("r{}", calls.fetch_add(1, Ordering::SeqCst)));

    p.process("a", "x", &h).unwrap();
    p.process("b", "x", &h).unwrap();

    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn should_reject_a_key_reused_for_a_different_body() {
    let (p, _) = sut!(setup());
    let calls = AtomicUsize::new(0);
    let h = |body: &str| {
        Ok(format!(
            "done:{body}:{}",
            calls.fetch_add(1, Ordering::SeqCst) + 1
        ))
    };
    p.process("key-1", "pay 10", &h).unwrap();

    assert_eq!(
        p.process("key-1", "pay 99", &h),
        Err(IdemError::KeyConflict)
    );

    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        p.process("key-1", "pay 10", &h),
        Ok("done:pay 10:1".to_string())
    );
}

#[test]
fn should_reject_a_blank_key() {
    let (p, _) = sut!(setup());

    assert_eq!(
        p.process(" ", "x", &|_: &str| Ok("r".to_string())),
        Err(IdemError::InvalidInput)
    );
}

#[test]
fn should_not_store_a_failure_and_run_the_handler_again_on_retry() {
    let (p, _) = sut!(setup());
    let calls = AtomicUsize::new(0);
    let flaky = |_: &str| {
        let n = calls.fetch_add(1, Ordering::SeqCst) + 1;
        if n == 1 {
            Err("boom".to_string())
        } else {
            Ok(format!("ok:{n}"))
        }
    };

    assert_eq!(
        p.process("k", "x", &flaky),
        Err(IdemError::HandlerFailed("boom".to_string()))
    );

    assert_eq!(p.process("k", "x", &flaky), Ok("ok:2".to_string()));
    assert_eq!(p.process("k", "x", &flaky), Ok("ok:2".to_string()));
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn should_expire_stored_responses_after_24_hours() {
    let (p, clock) = sut!(setup());
    let calls = AtomicUsize::new(0);
    let h = |body: &str| {
        Ok(format!(
            "done:{body}:{}",
            calls.fetch_add(1, Ordering::SeqCst) + 1
        ))
    };
    p.process("k", "x", &h).unwrap();

    clock.advance(24 * HOUR - 1);
    assert_eq!(p.process("k", "x", &h), Ok("done:x:1".to_string()));
    clock.advance(1);

    assert_eq!(p.process("k", "x", &h), Ok("done:x:2".to_string()));
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn should_allow_a_different_body_once_the_key_has_expired() {
    let (p, clock) = sut!(setup());
    p.process("k", "old", &|_: &str| Ok("r1".to_string()))
        .unwrap();

    clock.advance(25 * HOUR);

    assert_eq!(
        p.process("k", "new", &|_: &str| Ok("r2".to_string())),
        Ok("r2".to_string())
    );
}

#[test]
fn should_run_the_handler_only_once_when_many_threads_race_with_the_same_key() {
    let (p, _) = sut!(setup());
    let calls = AtomicUsize::new(0);
    let slow = |_: &str| {
        thread::sleep(Duration::from_millis(100));
        Ok(format!("r{}", calls.fetch_add(1, Ordering::SeqCst) + 1))
    };

    let start = Instant::now();
    let responses = collect_concurrently(32, |_| p.process("same", "body", &slow).unwrap());

    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(responses.iter().collect::<HashSet<_>>().len(), 1);
    assert_eq!(responses[0], "r1");
    assert!(
        start.elapsed() < Duration::from_millis(1_500),
        "waiters share the single execution"
    );
}

#[test]
fn should_not_make_different_keys_wait_for_each_other() {
    let (p, _) = sut!(setup());
    let slow = |_: &str| {
        thread::sleep(Duration::from_millis(150));
        Ok("r".to_string())
    };

    let start = Instant::now();
    run_concurrently(16, |t| {
        p.process(&format!("key-{t}"), "body", &slow).unwrap();
    });

    assert!(
        start.elapsed() < Duration::from_millis(1_200),
        "a global lock would need 2.4 s"
    );
}

#[test]
fn should_hand_over_to_a_waiting_caller_when_the_running_one_fails() {
    let (p, _) = sut!(setup());
    let calls = AtomicUsize::new(0);
    let fails_first = |_: &str| {
        thread::sleep(Duration::from_millis(100));
        if calls.fetch_add(1, Ordering::SeqCst) == 0 {
            Err("first one fails".to_string())
        } else {
            Ok("ok".to_string())
        }
    };

    let outcomes = collect_concurrently(8, |_| p.process("k", "b", &fails_first));

    assert_eq!(
        outcomes
            .iter()
            .filter(|o| matches!(o, Err(IdemError::HandlerFailed(_))))
            .count(),
        1
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|o| **o == Ok("ok".to_string()))
            .count(),
        7
    );
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn should_report_a_conflict_even_while_the_first_call_is_still_running() {
    let (p, _) = sut!(setup());
    let (started_tx, started_rx) = mpsc::channel::<()>();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let release_rx = Mutex::new(release_rx);

    thread::scope(|s| {
        s.spawn(|| {
            p.process("k", "A", &|_: &str| {
                started_tx.send(()).unwrap();
                let _ = release_rx.lock().unwrap().recv();
                Ok("r".to_string())
            })
            .unwrap();
        });
        started_rx.recv().unwrap();

        assert_eq!(
            p.process("k", "B", &|_: &str| Ok("x".to_string())),
            Err(IdemError::KeyConflict)
        );

        release_tx.send(()).unwrap();
    });
}

#[test]
fn should_stay_exactly_once_over_100_runs() {
    for run in 0..100 {
        let (p, _) = sut!(setup());
        let calls = AtomicUsize::new(0);
        let h = |_: &str| Ok(format!("r{}", calls.fetch_add(1, Ordering::SeqCst) + 1));

        let responses = collect_concurrently(8, |_| p.process("k", "b", &h).unwrap());

        assert_eq!(calls.load(Ordering::SeqCst), 1, "run {run}");
        assert!(responses.iter().all(|r| r == "r1"), "run {run}");
    }
}
