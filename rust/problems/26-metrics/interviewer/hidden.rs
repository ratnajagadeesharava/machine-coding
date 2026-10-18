#![allow(unused_imports, clippy::needless_range_loop)]
use p26_metrics::create;
use playground_support::{sut, FakeClock};

#[test]
fn an_event_exactly_at_the_left_edge_should_be_excluded_and_one_at_now_included() {
    let clock = FakeClock::new(1_000_000);
    let m = sut!(create(clock.as_clock()));
    m.increment("c", 1, &[]); // t = 1_000_000
    clock.advance(60_000);
    m.increment("c", 10, &[]); // t = 1_060_000 = now

    assert_eq!(m.count_in_window("c", 60_000), 10);
    assert_eq!(m.count_in_window("c", 60_001), 11);
}

#[test]
fn percentiles_on_rank_boundaries_should_use_ceiling_ranks() {
    let clock = FakeClock::new(1_000_000);
    let m = sut!(create(clock.as_clock()));
    for v in 1..=7 {
        m.observe("h", v * 10); // 10..=70, n = 7
    }

    assert_eq!(m.percentile("h", 14), Some(10)); // 14*7/100 = 0.98 -> rank 1
    assert_eq!(m.percentile("h", 15), Some(20)); // 1.05 -> rank 2
    assert_eq!(m.percentile("h", 29), Some(30)); // 2.03 -> rank 3
    assert_eq!(m.percentile("h", 57), Some(40)); // 3.99 -> rank 4
    assert_eq!(m.percentile("h", 58), Some(50)); // 4.06 -> rank 5
    assert_eq!(m.percentile("h", 100), Some(70));
}

#[test]
fn a_single_value_should_answer_every_percentile() {
    let clock = FakeClock::new(1_000_000);
    let m = sut!(create(clock.as_clock()));
    m.observe("h", 42);

    for p in 1..=100 {
        assert_eq!(m.percentile("h", p), Some(42));
    }
}

#[test]
fn the_empty_tag_set_should_be_independent_of_tagged_counters_but_included_in_the_total() {
    let clock = FakeClock::new(1_000_000);
    let m = sut!(create(clock.as_clock()));
    m.increment("c", 2, &[]);
    m.increment("c", 3, &[("a", "1")]);

    assert_eq!(m.counter("c", &[]), 2);
    assert_eq!(m.counter("c", &[("a", "1")]), 3);
    assert_eq!(m.counter_total("c"), 5);
}

#[test]
fn refused_increments_should_leave_no_trace_in_totals_or_windows() {
    let clock = FakeClock::new(1_000_000);
    let m = sut!(create(clock.as_clock()));
    m.increment("c", 5, &[]);
    m.increment("c", 0, &[("a", "1")]);
    m.increment("c", -9, &[]);

    assert_eq!(m.counter_total("c"), 5);
    assert_eq!(m.counter("c", &[("a", "1")]), 0);
    assert_eq!(m.count_in_window("c", 10_000), 5);
}
