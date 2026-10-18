#![allow(unused_imports, clippy::needless_range_loop)]
use p20_rate_limiter::{contract::*, create};
use playground_support::{sut, FakeClock};

#[test]
fn should_work_when_the_clock_starts_at_zero() {
    let clock = FakeClock::new(0);
    let rl = sut!(create(1, 1000, clock.as_clock()));
    assert!(rl.allow("a"));
    assert!(!rl.allow("a"));
    clock.advance(999);
    assert!(!rl.allow("a"));
    clock.advance(1);
    assert!(rl.allow("a"));
}

#[test]
fn should_support_a_one_millisecond_window() {
    let clock = FakeClock::new(5);
    let rl = sut!(create(1, 1, clock.as_clock()));
    assert!(rl.allow("a"));
    assert!(!rl.allow("a"));
    clock.advance(1);
    assert!(rl.allow("a"));
    assert!(!rl.allow("a"));
}

#[test]
fn should_expire_staggered_requests_one_by_one() {
    let clock = FakeClock::new(0);
    let rl = sut!(create(3, 1000, clock.as_clock()));
    assert!(rl.allow("a")); // t = 0
    clock.advance(100);
    assert!(rl.allow("a")); // t = 100
    clock.advance(100);
    assert!(rl.allow("a")); // t = 200
    assert!(!rl.allow("a"));
    clock.advance(800);
    assert!(rl.allow("a")); // t = 1000: t = 0 left
    assert!(!rl.allow("a"));
    clock.advance(100);
    assert!(rl.allow("a")); // t = 1100: t = 100 left
    assert!(!rl.allow("a"));
    clock.advance(100);
    assert!(rl.allow("a")); // t = 1200: t = 200 left
    assert!(!rl.allow("a"));
}

#[test]
fn persistent_retries_should_not_delay_the_next_allowed_request() {
    let clock = FakeClock::new(10_000);
    let rl = sut!(create(1, 1000, clock.as_clock()));
    assert!(rl.allow("a"));
    for _ in 0..9 {
        clock.advance(100);
        assert!(!rl.allow("a"));
    }
    clock.advance(100);
    assert!(rl.allow("a"));
}

#[test]
fn one_keys_expiry_should_not_affect_another_key() {
    let clock = FakeClock::new(0);
    let rl = sut!(create(1, 1000, clock.as_clock()));
    assert!(rl.allow("A")); // t = 0
    clock.advance(500);
    assert!(rl.allow("B")); // t = 500
    clock.advance(500);
    assert!(rl.allow("A")); // t = 1000: A's request expired
    assert!(!rl.allow("B")); // B's has not
    clock.advance(500);
    assert!(rl.allow("B")); // t = 1500
}
