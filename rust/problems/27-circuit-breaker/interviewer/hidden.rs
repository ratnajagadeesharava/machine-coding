#![allow(unused_imports, clippy::needless_range_loop)]
use p27_circuit_breaker::create;
use playground_support::{sut, FakeClock};

#[test]
fn threshold_of_one_should_trip_on_the_first_failure_and_one_probe_should_close() {
    let clock = FakeClock::new(5_000);
    let cb = sut!(create(1, 200, 1, clock.as_clock()));

    assert_eq!(cb.call(&|| true), "SUCCESS");
    assert_eq!(cb.call(&|| false), "FAILURE");
    assert_eq!(cb.state(), "OPEN");

    clock.advance(200);
    assert_eq!(cb.call(&|| true), "SUCCESS");
    assert_eq!(cb.state(), "CLOSED");
    assert_eq!(cb.call(&|| false), "FAILURE");
    assert_eq!(cb.state(), "OPEN");
}

#[test]
fn rejected_calls_should_not_extend_the_cool_down() {
    let clock = FakeClock::new(5_000);
    let cb = sut!(create(2, 1000, 1, clock.as_clock()));
    cb.call(&|| false);
    cb.call(&|| false);

    clock.advance(600);
    assert_eq!(cb.call(&|| true), "REJECTED");
    clock.advance(400);

    assert_eq!(cb.state(), "HALF_OPEN");
}

#[test]
fn a_trial_failure_should_discard_earlier_trial_successes() {
    let clock = FakeClock::new(5_000);
    let cb = sut!(create(2, 100, 3, clock.as_clock()));
    cb.call(&|| false);
    cb.call(&|| false);
    clock.advance(100);
    cb.call(&|| true);
    cb.call(&|| true);
    assert_eq!(cb.state(), "HALF_OPEN");

    assert_eq!(cb.call(&|| false), "FAILURE");
    assert_eq!(cb.state(), "OPEN");

    clock.advance(100);
    cb.call(&|| true);
    cb.call(&|| true);
    assert_eq!(cb.state(), "HALF_OPEN");
    cb.call(&|| true);
    assert_eq!(cb.state(), "CLOSED");
}

#[test]
fn should_stay_half_open_indefinitely_until_a_trial_decides_and_need_no_call_to_observe_it() {
    let clock = FakeClock::new(5_000);
    let cb = sut!(create(1, 50, 1, clock.as_clock()));
    cb.call(&|| false);

    clock.advance(1_000_000);

    assert_eq!(cb.state(), "HALF_OPEN");
    assert_eq!(cb.state(), "HALF_OPEN");
}

#[test]
fn should_keep_independent_guards_independent() {
    let clock = FakeClock::new(5_000);
    let a = sut!(create(1, 100, 1, clock.as_clock()));
    let b = sut!(create(1, 100, 1, clock.as_clock()));

    a.call(&|| false);

    assert_eq!(a.state(), "OPEN");
    assert_eq!(b.state(), "CLOSED");
    assert_eq!(b.call(&|| true), "SUCCESS");
}

#[test]
fn should_count_only_consecutive_failures_across_a_closing_cycle() {
    let clock = FakeClock::new(5_000);
    let cb = sut!(create(3, 100, 1, clock.as_clock()));
    cb.call(&|| false);
    cb.call(&|| false);
    cb.call(&|| true);
    cb.call(&|| false);
    cb.call(&|| false);
    assert_eq!(cb.state(), "CLOSED");

    cb.call(&|| false);
    clock.advance(100);
    cb.call(&|| true);

    cb.call(&|| false);
    cb.call(&|| false);
    assert_eq!(cb.state(), "CLOSED");
}
