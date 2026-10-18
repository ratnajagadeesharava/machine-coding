#![allow(unused_imports, clippy::needless_range_loop)]
use p26_metrics::{contract::*, create};
use playground_support::{run_concurrently, sut, FakeClock};
use std::sync::Arc;
use std::time::{Duration, Instant};

const T0: u64 = 10_000;

fn metrics(clock: &FakeClock) -> Option<Arc<dyn MetricsApi>> {
    create(clock.as_clock())
}

fn observe_all(m: &Arc<dyn MetricsApi>, name: &str, values: &[i64]) {
    for v in values {
        m.observe(name, *v);
    }
}

#[test]
fn should_count_increments_per_tag_set() {
    let clock = FakeClock::new(T0);
    let m = sut!(metrics(&clock));

    assert!(m.increment("http", 1, &[("route", "/pay"), ("status", "200")]));
    assert!(m.increment("http", 2, &[("route", "/pay"), ("status", "500")]));
    assert!(m.increment("http", 4, &[("route", "/pay"), ("status", "200")]));

    assert_eq!(
        m.counter("http", &[("route", "/pay"), ("status", "200")]),
        5
    );
    assert_eq!(
        m.counter("http", &[("route", "/pay"), ("status", "500")]),
        2
    );
    assert_eq!(m.counter_total("http"), 7);
}

#[test]
fn should_match_tags_exactly() {
    let clock = FakeClock::new(T0);
    let m = sut!(metrics(&clock));
    m.increment("http", 1, &[("route", "/pay"), ("status", "200")]);
    m.increment("http", 10, &[("route", "/pay")]);
    m.increment("http", 100, &[]);

    assert_eq!(m.counter("http", &[("route", "/pay")]), 10);
    assert_eq!(m.counter("http", &[("status", "200")]), 0);
    assert_eq!(
        m.counter("http", &[("route", "/pay"), ("status", "404")]),
        0
    );
    assert_eq!(m.counter("http", &[]), 100);
    assert_eq!(m.counter_total("http"), 111);
}

#[test]
fn should_ignore_tag_order() {
    let clock = FakeClock::new(T0);
    let m = sut!(metrics(&clock));
    m.increment("x", 1, &[("a", "1"), ("b", "2")]);
    m.increment("x", 1, &[("b", "2"), ("a", "1")]);

    assert_eq!(m.counter("x", &[("b", "2"), ("a", "1")]), 2);
    assert_eq!(m.counter("x", &[("a", "1")]), 0);
}

#[test]
fn should_return_zero_for_unknown_counters() {
    let clock = FakeClock::new(T0);
    let m = sut!(metrics(&clock));

    assert_eq!(m.counter("nope", &[]), 0);
    assert_eq!(m.counter_total("nope"), 0);
    assert_eq!(m.count_in_window("nope", 1_000), 0);
}

#[test]
fn should_refuse_non_positive_deltas_and_blank_names() {
    let clock = FakeClock::new(T0);
    let m = sut!(metrics(&clock));

    assert!(!m.increment("x", 0, &[]));
    assert!(!m.increment("x", -5, &[]));
    assert!(!m.increment("  ", 1, &[]));
    assert!(!m.observe("", 1));

    assert_eq!(m.counter_total("x"), 0);
    assert_eq!(m.count_in_window("x", 1_000_000), 0);
}

#[test]
fn should_compute_nearest_rank_percentiles() {
    let clock = FakeClock::new(T0);
    let m = sut!(metrics(&clock));
    observe_all(&m, "latency", &[50, 15, 40, 20, 35]); // sorted: 15 20 35 40 50

    assert_eq!(m.percentile("latency", 1), Some(15));
    assert_eq!(m.percentile("latency", 20), Some(15)); // rank ceil(1.0) = 1
    assert_eq!(m.percentile("latency", 30), Some(20)); // rank ceil(1.5) = 2
    assert_eq!(m.percentile("latency", 40), Some(20)); // rank 2
    assert_eq!(m.percentile("latency", 50), Some(35)); // rank ceil(2.5) = 3
    assert_eq!(m.percentile("latency", 75), Some(40)); // rank ceil(3.75) = 4
    assert_eq!(m.percentile("latency", 95), Some(50)); // rank ceil(4.75) = 5
    assert_eq!(m.percentile("latency", 100), Some(50));
    assert_eq!(m.observation_count("latency"), 5);
}

#[test]
fn should_return_the_value_with_the_same_percentile_over_a_hundred_distinct_values() {
    let clock = FakeClock::new(T0);
    let m = sut!(metrics(&clock));
    // 37 is coprime with 100, so this visits 1..=100 in a scrambled order.
    for i in 0..100i64 {
        m.observe("h", (i * 37) % 100 + 1);
    }

    for p in 1..=100u32 {
        assert_eq!(m.percentile("h", p), Some(p as i64), "p{p}");
    }
}

#[test]
fn should_handle_duplicates_and_negative_values() {
    let clock = FakeClock::new(T0);
    let m = sut!(metrics(&clock));
    observe_all(&m, "h", &[5, 5, 5, -3, 5, 5, 5, 5, 5, 100]); // n=10: -3, 5 x8, 100

    assert_eq!(m.percentile("h", 10), Some(-3));
    assert_eq!(m.percentile("h", 11), Some(5));
    assert_eq!(m.percentile("h", 90), Some(5));
    assert_eq!(m.percentile("h", 91), Some(100));
}

#[test]
fn should_keep_percentiles_correct_when_values_arrive_between_queries() {
    let clock = FakeClock::new(T0);
    let m = sut!(metrics(&clock));
    observe_all(&m, "h", &[10, 20, 30]);
    assert_eq!(m.percentile("h", 100), Some(30));

    m.observe("h", 5);
    m.observe("h", 40);

    assert_eq!(m.percentile("h", 100), Some(40));
    assert_eq!(m.percentile("h", 20), Some(5)); // n=5, rank 1
    assert_eq!(m.percentile("h", 60), Some(20)); // rank 3
}

#[test]
fn should_return_none_for_unknown_histograms_or_bad_ranks() {
    let clock = FakeClock::new(T0);
    let m = sut!(metrics(&clock));
    m.observe("h", 1);

    assert_eq!(m.percentile("nope", 50), None);
    assert_eq!(m.percentile("h", 0), None);
    assert_eq!(m.percentile("h", 101), None);
    assert_eq!(m.observation_count("nope"), 0);
}

#[test]
fn should_keep_counters_and_histograms_in_separate_namespaces() {
    let clock = FakeClock::new(T0);
    let m = sut!(metrics(&clock));
    m.increment("x", 3, &[]);
    m.observe("x", 99);

    assert_eq!(m.counter_total("x"), 3);
    assert_eq!(m.observation_count("x"), 1);
    assert_eq!(m.percentile("x", 100), Some(99));
}

#[test]
fn should_count_only_what_falls_in_the_half_open_window() {
    let clock = FakeClock::new(T0);
    let m = sut!(metrics(&clock));
    m.increment("c", 5, &[]); // at 10_000
    clock.advance(1_000);
    m.increment("c", 3, &[("k", "v")]); // at 11_000

    assert_eq!(m.count_in_window("c", 1_000), 3); // (10_000, 11_000]: the first is excluded
    assert_eq!(m.count_in_window("c", 1_001), 8); // (9_999, 11_000]
    assert_eq!(m.count_in_window("c", 1), 3);
    assert_eq!(m.count_in_window("c", 0), 0);
    assert_eq!(m.count_in_window("c", u64::MAX), 8);
}

#[test]
fn should_let_events_leave_the_window_as_time_passes() {
    let clock = FakeClock::new(T0);
    let m = sut!(metrics(&clock));
    m.increment("c", 1, &[]); // 10_000
    clock.advance(400);
    m.increment("c", 2, &[]); // 10_400
    clock.advance(400);
    m.increment("c", 4, &[]); // 10_800

    assert_eq!(m.count_in_window("c", 1_000), 7);
    clock.advance(300); // now 11_100
    assert_eq!(m.count_in_window("c", 1_000), 6); // (10_100, 11_100]: drops the first
    clock.advance(700); // now 11_800
    assert_eq!(m.count_in_window("c", 1_000), 0); // (10_800, 11_800]: the last one is at 10_800, excluded
    assert_eq!(m.count_in_window("c", 1_001), 4); // (10_799, 11_800]
}

#[test]
fn should_sum_all_tag_sets_and_ignore_other_counters_in_the_window() {
    let clock = FakeClock::new(T0);
    let m = sut!(metrics(&clock));
    m.increment("a", 1, &[("t", "1")]);
    m.increment("a", 2, &[("t", "2")]);
    m.increment("b", 100, &[]);

    assert_eq!(m.count_in_window("a", 5_000), 3);
    assert_eq!(m.count_in_window("b", 5_000), 100);
}

#[test]
fn should_not_lose_increments_from_many_threads() {
    let clock = FakeClock::new(T0);
    let m = sut!(metrics(&clock));

    run_concurrently(100, |i| {
        let shard = (i % 4).to_string();
        for _ in 0..1_000 {
            m.increment("hits", 1, &[("shard", shard.as_str())]);
        }
    });

    assert_eq!(m.counter_total("hits"), 100_000);
    for shard in 0..4 {
        assert_eq!(
            m.counter("hits", &[("shard", shard.to_string().as_str())]),
            25_000
        );
    }
    assert_eq!(m.count_in_window("hits", 1), 100_000);
}

#[test]
fn should_not_lose_increments_on_a_single_counter_from_many_threads() {
    let clock = FakeClock::new(T0);
    let m = sut!(metrics(&clock));

    run_concurrently(100, |_| {
        for _ in 0..1_000 {
            m.increment("single", 1, &[]);
        }
    });

    assert_eq!(m.counter("single", &[]), 100_000);
}

#[test]
fn should_observe_and_query_percentiles_concurrently() {
    let clock = FakeClock::new(T0);
    let m = sut!(metrics(&clock));

    run_concurrently(12, |t| {
        if t < 8 {
            for v in 1..=1_000 {
                m.observe("lat", v);
            }
        } else {
            for _ in 0..300 {
                if let Some(p) = m.percentile("lat", 50) {
                    assert!((1..=1_000).contains(&p));
                }
                m.observation_count("lat");
            }
        }
    });

    assert_eq!(m.observation_count("lat"), 8_000);
    assert_eq!(m.percentile("lat", 50), Some(500));
    assert_eq!(m.percentile("lat", 1), Some(10));
    assert_eq!(m.percentile("lat", 100), Some(1_000));
}

#[test]
fn should_stay_fast_with_a_million_observations_and_interleaved_queries() {
    let clock = FakeClock::new(T0);
    let m = sut!(metrics(&clock));
    let start = Instant::now();

    let mut seed = 12345u64;
    for _ in 0..1_000_000 {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        m.observe("big", ((seed >> 33) % 100_000) as i64);
    }
    assert_eq!(m.observation_count("big"), 1_000_000);
    for q in 0..1_000u32 {
        assert!(m.percentile("big", 1 + q % 100).is_some());
    }
    for round in 0..300i64 {
        m.observe("big", 100_000 + round);
        assert!(m.percentile("big", 99).is_some());
    }

    assert_eq!(m.percentile("big", 100), Some(100_299));
    assert_eq!(m.observation_count("big"), 1_000_300);
    assert!(start.elapsed() < Duration::from_secs(60));
}

#[test]
fn should_answer_window_queries_quickly_over_many_increments() {
    let clock = FakeClock::new(T0);
    let m = sut!(metrics(&clock));
    let start = Instant::now();

    for i in 0..300_000u64 {
        let tag = (i % 10).to_string();
        m.increment("events", 1, &[("n", tag.as_str())]);
        clock.advance(1);
    }
    let mut total = 0;
    for q in 1..=5_000u64 {
        total += m.count_in_window("events", q * 50);
    }

    assert_eq!(m.count_in_window("events", 1_000), 999); // events at t = now-999 .. now-1
    assert_eq!(m.count_in_window("events", 10_000_000), 300_000);
    assert!(total > 0);
    assert!(start.elapsed() < Duration::from_secs(60));
}
