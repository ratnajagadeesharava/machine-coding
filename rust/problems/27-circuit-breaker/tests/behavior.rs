#![allow(unused_imports, clippy::needless_range_loop)]
use p27_circuit_breaker::{contract::*, create};
use playground_support::{collect_concurrently, eventually, stays, sut, FakeClock};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

const OPEN_MS: u64 = 1000;

/// threshold 3, cool-down 1000 ms, 2 probe successes required.
fn system(clock: &FakeClock) -> Option<Arc<dyn CircuitBreakerApi>> {
    create(3, OPEN_MS, 2, clock.as_clock())
}

fn trip(cb: &Arc<dyn CircuitBreakerApi>) {
    for _ in 0..3 {
        cb.call(&|| false);
    }
}

fn half_open(clock: &FakeClock) -> Option<Arc<dyn CircuitBreakerApi>> {
    let cb = system(clock)?;
    trip(&cb);
    clock.advance(OPEN_MS);
    Some(cb)
}

#[test]
fn should_start_closed_and_pass_successful_calls() {
    let clock = FakeClock::new(10_000);
    let cb = sut!(system(&clock));

    assert_eq!(cb.state(), "CLOSED");
    assert_eq!(cb.call(&|| true), "SUCCESS");
    assert_eq!(cb.state(), "CLOSED");
}

#[test]
fn should_report_failures_below_threshold_without_tripping() {
    let clock = FakeClock::new(10_000);
    let cb = sut!(system(&clock));

    assert_eq!(cb.call(&|| false), "FAILURE");
    assert_eq!(cb.call(&|| false), "FAILURE");

    assert_eq!(cb.state(), "CLOSED");
}

#[test]
fn should_trip_after_consecutive_failures() {
    let clock = FakeClock::new(10_000);
    let cb = sut!(system(&clock));

    trip(&cb);

    assert_eq!(cb.state(), "OPEN");
}

#[test]
fn should_reset_failure_count_on_success() {
    let clock = FakeClock::new(10_000);
    let cb = sut!(system(&clock));

    cb.call(&|| false);
    cb.call(&|| false);
    cb.call(&|| true);
    cb.call(&|| false);
    cb.call(&|| false);
    assert_eq!(cb.state(), "CLOSED");

    cb.call(&|| false);
    assert_eq!(cb.state(), "OPEN");
}

#[test]
fn should_reject_without_running_the_operation_while_open() {
    let clock = FakeClock::new(10_000);
    let cb = sut!(system(&clock));
    trip(&cb);
    let runs = AtomicUsize::new(0);

    let result = cb.call(&|| {
        runs.fetch_add(1, Ordering::SeqCst);
        true
    });

    assert_eq!(result, "REJECTED");
    assert_eq!(runs.load(Ordering::SeqCst), 0);
    assert_eq!(cb.state(), "OPEN");
}

#[test]
fn should_stay_open_until_the_cool_down_has_elapsed() {
    let clock = FakeClock::new(10_000);
    let cb = sut!(system(&clock));
    trip(&cb);

    clock.advance(OPEN_MS - 1);
    assert_eq!(cb.state(), "OPEN");
    assert_eq!(cb.call(&|| true), "REJECTED");

    clock.advance(1);
    assert_eq!(cb.state(), "HALF_OPEN");
}

#[test]
fn should_close_after_the_required_number_of_successful_trials() {
    let clock = FakeClock::new(10_000);
    let cb = sut!(half_open(&clock));

    assert_eq!(cb.call(&|| true), "SUCCESS");
    assert_eq!(cb.state(), "HALF_OPEN");
    assert_eq!(cb.call(&|| true), "SUCCESS");
    assert_eq!(cb.state(), "CLOSED");
}

#[test]
fn should_start_with_a_fresh_failure_count_after_closing() {
    let clock = FakeClock::new(10_000);
    let cb = sut!(half_open(&clock));
    cb.call(&|| true);
    cb.call(&|| true);

    cb.call(&|| false);
    cb.call(&|| false);

    assert_eq!(cb.state(), "CLOSED");
}

#[test]
fn should_trip_again_and_restart_the_cool_down_when_a_trial_fails() {
    let clock = FakeClock::new(10_000);
    let cb = sut!(half_open(&clock));

    assert_eq!(cb.call(&|| false), "FAILURE");
    assert_eq!(cb.state(), "OPEN");

    clock.advance(OPEN_MS - 1);
    assert_eq!(cb.state(), "OPEN");
    clock.advance(1);
    assert_eq!(cb.state(), "HALF_OPEN");
}

#[test]
fn should_require_the_full_number_of_trial_successes_again_after_a_trial_failure() {
    let clock = FakeClock::new(10_000);
    let cb = sut!(half_open(&clock));
    cb.call(&|| true);
    cb.call(&|| false);
    clock.advance(OPEN_MS);

    cb.call(&|| true);
    assert_eq!(cb.state(), "HALF_OPEN");
    cb.call(&|| true);
    assert_eq!(cb.state(), "CLOSED");
}

#[test]
fn should_allow_parallel_calls_while_closed() {
    let clock = FakeClock::new(10_000);
    let cb = sut!(system(&clock));
    let inside = AtomicUsize::new(0);

    let results = collect_concurrently(2, |_| {
        cb.call(&|| {
            inside.fetch_add(1, Ordering::SeqCst);
            eventually(10_000, || inside.load(Ordering::SeqCst) >= 2)
        })
    });

    assert!(results.iter().all(|r| r == "SUCCESS"));
}

#[test]
fn should_reject_every_concurrent_call_while_open_without_running_any() {
    let clock = FakeClock::new(10_000);
    let cb = sut!(system(&clock));
    trip(&cb);
    let runs = AtomicUsize::new(0);

    let results = collect_concurrently(100, |_| {
        cb.call(&|| {
            runs.fetch_add(1, Ordering::SeqCst);
            true
        })
    });

    assert_eq!(results.len(), 100);
    assert!(results.iter().all(|r| r == "REJECTED"));
    assert_eq!(runs.load(Ordering::SeqCst), 0);
}

#[test]
fn should_trip_exactly_once_under_concurrent_failures() {
    let clock = FakeClock::new(10_000);
    let cb = sut!(system(&clock));
    let runs = AtomicUsize::new(0);

    let results = collect_concurrently(100, |_| {
        cb.call(&|| {
            runs.fetch_add(1, Ordering::SeqCst);
            false
        })
    });

    let failures = results.iter().filter(|r| *r == "FAILURE").count();
    let rejected = results.iter().filter(|r| *r == "REJECTED").count();
    assert_eq!(failures + rejected, 100);
    assert!(failures >= 3);
    assert_eq!(runs.load(Ordering::SeqCst), failures);
    assert_eq!(cb.state(), "OPEN");
}

#[test]
fn should_admit_only_one_trial_at_a_time_when_half_open() {
    let clock = FakeClock::new(10_000);
    let cb = sut!(create(3, OPEN_MS, 3, clock.as_clock()));
    trip(&cb);
    clock.advance(OPEN_MS);
    let started = AtomicUsize::new(0);
    let finished = AtomicUsize::new(0);
    let release = AtomicBool::new(false);

    let results = std::thread::scope(|s| {
        let handles: Vec<_> = (0..20)
            .map(|_| {
                s.spawn(|| {
                    let r = cb.call(&|| {
                        started.fetch_add(1, Ordering::SeqCst);
                        eventually(20_000, || release.load(Ordering::SeqCst));
                        true
                    });
                    finished.fetch_add(1, Ordering::SeqCst);
                    r
                })
            })
            .collect();

        let settled = eventually(10_000, || {
            started.load(Ordering::SeqCst) == 1 && finished.load(Ordering::SeqCst) == 19
        });
        let single = stays(100, || started.load(Ordering::SeqCst) == 1);
        let state = cb.state();
        release.store(true, Ordering::SeqCst);
        let results: Vec<String> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        assert!(settled, "one trial running, the other 19 callers refused");
        assert!(single, "no second trial may start");
        assert_eq!(state, "HALF_OPEN");
        results
    });

    assert_eq!(results.iter().filter(|r| *r == "SUCCESS").count(), 1);
    assert_eq!(results.iter().filter(|r| *r == "REJECTED").count(), 19);
    assert_eq!(cb.state(), "HALF_OPEN");
}

#[test]
fn should_admit_the_next_trial_once_the_running_one_has_finished() {
    let clock = FakeClock::new(10_000);
    let cb = sut!(half_open(&clock));
    let inside = AtomicBool::new(false);
    let release = AtomicBool::new(false);

    std::thread::scope(|s| {
        let first = s.spawn(|| {
            cb.call(&|| {
                inside.store(true, Ordering::SeqCst);
                eventually(20_000, || release.load(Ordering::SeqCst));
                true
            })
        });
        assert!(eventually(10_000, || inside.load(Ordering::SeqCst)));

        assert_eq!(cb.call(&|| true), "REJECTED");
        assert_eq!(cb.state(), "HALF_OPEN");

        release.store(true, Ordering::SeqCst);
        assert_eq!(first.join().unwrap(), "SUCCESS");
    });

    assert_eq!(cb.call(&|| true), "SUCCESS");
    assert_eq!(cb.state(), "CLOSED");
}
