#![allow(unused_imports, clippy::needless_range_loop)]
use p20_rate_limiter::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut, FakeClock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

const START: u64 = 1_000_000;

fn limiter(limit: u32, window_millis: u64, clock: &FakeClock) -> Option<Arc<dyn RateLimiterApi>> {
    create(limit, window_millis, clock.as_clock())
}

#[test]
fn should_allow_up_to_the_limit_and_then_deny() {
    let clock = FakeClock::new(START);
    let rl = sut!(limiter(3, 1000, &clock));

    assert!(rl.allow("a"));
    assert!(rl.allow("a"));
    assert!(rl.allow("a"));
    assert!(!rl.allow("a"));
    assert!(!rl.allow("a"));
}

#[test]
fn should_keep_keys_independent() {
    let clock = FakeClock::new(START);
    let rl = sut!(limiter(1, 1000, &clock));

    assert!(rl.allow("a"));
    assert!(!rl.allow("a"));

    assert!(rl.allow("b"));
    assert!(!rl.allow("b"));
}

#[test]
fn should_treat_keys_as_case_sensitive_and_accept_any_string() {
    let clock = FakeClock::new(START);
    let rl = sut!(limiter(1, 1000, &clock));

    assert!(rl.allow("user"));
    assert!(rl.allow("USER"));
    assert!(rl.allow(""));
    assert!(rl.allow("\u{e9}\u{4e2d}"));
    assert!(!rl.allow(""));
    assert!(!rl.allow("user"));
}

#[test]
fn should_allow_again_when_the_oldest_request_leaves_the_window() {
    let clock = FakeClock::new(START);
    let rl = sut!(limiter(2, 1000, &clock));

    assert!(rl.allow("a")); // t = 0
    clock.advance(400);
    assert!(rl.allow("a")); // t = 400
    clock.advance(599);
    assert!(!rl.allow("a")); // t = 999: window (-1, 999] holds both
    clock.advance(1);
    assert!(rl.allow("a")); // t = 1000: window (0, 1000] holds only t = 400
    assert!(!rl.allow("a"));
}

#[test]
fn should_expire_a_request_exactly_one_window_after_it_was_allowed() {
    let clock = FakeClock::new(START);
    let rl = sut!(limiter(1, 1000, &clock));

    assert!(rl.allow("a"));
    clock.advance(999);
    assert!(!rl.allow("a"));
    clock.advance(1);
    assert!(rl.allow("a"));
}

#[test]
fn should_not_count_denied_requests() {
    let clock = FakeClock::new(START);
    let rl = sut!(limiter(2, 1000, &clock));

    assert!(rl.allow("a")); // t = 0
    clock.advance(400);
    assert!(rl.allow("a")); // t = 400
    clock.advance(100);
    assert!(!rl.allow("a")); // t = 500, denied
    clock.advance(499);
    assert!(!rl.allow("a")); // t = 999, denied
    clock.advance(1);
    assert!(rl.allow("a")); // t = 1000: only t = 400 counts
    assert!(!rl.allow("a"));
    clock.advance(400);
    assert!(rl.allow("a")); // t = 1400: t = 400 left, t = 1000 counts
    assert!(!rl.allow("a"));
}

#[test]
fn should_slide_instead_of_resetting_on_fixed_boundaries() {
    let clock = FakeClock::new(START);
    let rl = sut!(limiter(2, 1000, &clock));

    clock.advance(900);
    assert!(rl.allow("a")); // t = 900
    clock.advance(50);
    assert!(rl.allow("a")); // t = 950
    clock.advance(50);
    assert!(!rl.allow("a")); // t = 1000: a fixed window would have reset here
    clock.advance(100);
    assert!(!rl.allow("a")); // t = 1100
    clock.advance(800);
    assert!(rl.allow("a")); // t = 1900: only t = 950 counts
    assert!(!rl.allow("a"));
}

#[test]
fn should_restore_the_full_budget_after_a_full_window_of_silence() {
    let clock = FakeClock::new(START);
    let rl = sut!(limiter(5, 1000, &clock));
    for _ in 0..5 {
        assert!(rl.allow("a"));
    }
    assert!(!rl.allow("a"));

    clock.advance(1000);

    for _ in 0..5 {
        assert!(rl.allow("a"));
    }
    assert!(!rl.allow("a"));
}

#[test]
fn should_allow_exactly_the_limit_when_many_threads_hammer_one_key_at_the_same_instant() {
    let clock = FakeClock::new(START);
    let rl = sut!(limiter(100, 60_000, &clock));

    let allowed_per_thread =
        collect_concurrently(32, |_| (0..50).filter(|_| rl.allow("hot")).count());

    assert_eq!(allowed_per_thread.iter().sum::<usize>(), 100);
    assert!(!rl.allow("hot"));
}

#[test]
fn should_hold_the_limit_per_key_when_many_keys_are_hammered_concurrently() {
    let clock = FakeClock::new(START);
    let rl = sut!(limiter(3, 60_000, &clock));
    let allowed: Vec<AtomicUsize> = (0..100).map(|_| AtomicUsize::new(0)).collect();

    run_concurrently(8, |_| {
        for _round in 0..5 {
            for (k, counter) in allowed.iter().enumerate() {
                if rl.allow(&format!("key{k}")) {
                    counter.fetch_add(1, Ordering::SeqCst);
                }
            }
        }
    });

    for (k, counter) in allowed.iter().enumerate() {
        assert_eq!(counter.load(Ordering::SeqCst), 3, "key{k}");
    }
}

#[test]
fn should_allow_exactly_the_limit_again_in_each_new_window_under_concurrency() {
    let clock = FakeClock::new(START);
    let rl = sut!(limiter(20, 1000, &clock));

    for phase in 0..3 {
        let counts = collect_concurrently(16, |_| (0..10).filter(|_| rl.allow("a")).count());
        assert_eq!(counts.iter().sum::<usize>(), 20, "phase {phase}");
        clock.advance(1000);
    }
}

#[test]
fn should_stay_correct_after_many_one_off_keys_and_passing_time() {
    let clock = FakeClock::new(START);
    let rl = sut!(limiter(2, 1000, &clock));
    let start = Instant::now();

    for i in 0..200_000 {
        let key = format!("k{i}");
        assert!(rl.allow(&key));
        assert!(rl.allow(&key));
        assert!(!rl.allow(&key));
        clock.advance(10);
    }
    // key i was used at START + 10 * i; now is START + 2_000_000
    assert!(rl.allow("k0"));
    assert!(rl.allow("k0"));
    assert!(!rl.allow("k0"));
    assert!(rl.allow("k199900")); // used exactly one window ago: expired
    assert!(!rl.allow("k199901")); // used 990 ms ago: still counts
    assert!(!rl.allow("k199999"));

    assert!(start.elapsed() < Duration::from_secs(10));
}

#[test]
fn should_decide_quickly_even_with_a_very_large_limit() {
    let clock = FakeClock::new(START);
    let rl = sut!(limiter(200_000, 10_000_000, &clock));
    let start = Instant::now();

    for i in 0..200_000 {
        assert!(rl.allow("hot"));
        if i % 10 == 0 {
            clock.advance(1);
        }
    }
    for _ in 0..200_000 {
        assert!(!rl.allow("hot"));
    }

    assert!(start.elapsed() < Duration::from_secs(10));
}
