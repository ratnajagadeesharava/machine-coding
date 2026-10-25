#![allow(unused_imports, clippy::needless_range_loop)]
use p137_hit_counter_window::{contract::*, create};
use playground_support::{run_concurrently, sut, FakeClock};
use std::sync::Arc;

#[test]
fn should_stay_bounded_when_hits_keep_arriving_and_nobody_reads() {
    let clock = FakeClock::new(0);
    let c = sut!(create(60_000, clock.as_clock()));

    for t in 0..300_000 {
        clock.set(t);
        c.hit("stream").unwrap();
    }

    assert!(c.memory_probe() <= 100_000, "got {}", c.memory_probe());
    assert_eq!(c.count("stream"), Ok(60_000));
    assert_eq!(c.count_in_last("stream", 1_000), Ok(1_000));
}

#[test]
fn should_release_the_memory_of_many_keys_that_went_quiet() {
    let clock = FakeClock::new(0);
    let c = sut!(create(1_000, clock.as_clock()));
    for k in 0..20_000 {
        c.hit_n(&format!("key-{k}"), 2).unwrap();
        if k % 100 == 99 {
            clock.advance(1);
        }
    }
    assert_eq!(c.total_in_window(), 40_000);

    clock.advance(10_000);

    assert_eq!(c.total_in_window(), 0);
    assert!(c.top_keys(5).unwrap().is_empty());
    assert!(c.memory_probe() <= 100_000, "got {}", c.memory_probe());
}

#[test]
fn should_layer_shrink_and_grow_across_several_windows() {
    let clock = FakeClock::new(0);
    let c = sut!(create(10_000, clock.as_clock()));
    for t in [0u64, 3_000, 6_000, 9_000] {
        clock.set(t);
        c.hit_n("a", if t == 0 { 1 } else { t }).unwrap();
    }
    assert_eq!(c.count("a"), Ok(1 + 3_000 + 6_000 + 9_000));

    c.set_window(5_000).unwrap();
    assert_eq!(c.count("a"), Ok(15_000));

    c.set_window(20_000).unwrap();
    assert_eq!(c.count("a"), Ok(15_000), "hits at 0 and 3000 stay dropped");
    clock.set(14_000);
    assert_eq!(c.count("a"), Ok(15_000));
    clock.set(15_001);
    assert_eq!(c.count("a"), Ok(15_000));
    clock.set(26_000);
    assert_eq!(c.count("a"), Ok(9_000), "6000 expired, 9000 live");
    clock.set(29_000);
    assert_eq!(c.count("a"), Ok(0));
}

#[test]
fn should_handle_the_clock_going_back_and_forth_around_expiry() {
    let clock = FakeClock::new(10_000);
    let c = sut!(create(1_000, clock.as_clock()));
    c.hit("a").unwrap();
    clock.set(0);
    c.hit_n("a", 3).unwrap();
    c.hit_n("a", 3).unwrap();
    clock.set(10_500);
    c.hit("a").unwrap();

    assert_eq!(c.count("a"), Ok(8));
    clock.set(10_700);
    assert_eq!(c.count("a"), Ok(8));
    clock.set(100);
    assert_eq!(c.count("a"), Ok(8), "now never went below 10700");
    clock.set(11_000);
    assert_eq!(c.count("a"), Ok(1));
    clock.set(11_500);
    assert_eq!(c.count("a"), Ok(0));
}

#[test]
fn should_keep_counts_exact_while_the_window_is_resized_concurrently() {
    let clock = FakeClock::new(1_000_000);
    let c = sut!(create(5_000, clock.as_clock()));

    run_concurrently(9, |t| {
        if t == 8 {
            for i in 0..20_000u64 {
                c.set_window(1_000 + (i % 2) * 4_000).unwrap();
            }
            return;
        }
        for i in 0..20_000 {
            c.hit(&format!("k{}", i % 5)).unwrap();
        }
    });

    assert_eq!(c.total_in_window(), 160_000);
}

#[test]
fn should_answer_top_keys_consistently_while_writers_are_running() {
    let clock = FakeClock::new(0);
    let c = sut!(create(60_000, clock.as_clock()));

    run_concurrently(16, |t| {
        if t >= 12 {
            for _ in 0..2_000 {
                assert!(c.top_keys(3).unwrap().len() <= 3);
                c.count_in_last("k0", 1).unwrap();
            }
            return;
        }
        for _ in 0..10_000 {
            c.hit(&format!("k{t}")).unwrap();
        }
    });

    assert_eq!(c.total_in_window(), 120_000);
    assert_eq!(c.top_keys(3).unwrap(), vec!["k0", "k1", "k10"]);
}

#[test]
fn should_keep_working_across_a_very_long_run_with_big_batches() {
    let clock = FakeClock::new(0);
    let c = sut!(create(3_600_000, clock.as_clock()));

    for minute in 0..600u64 {
        clock.set(minute * 60_000);
        c.hit_n("api", 1_000_000_000).unwrap();
    }
    clock.set(599 * 60_000);

    assert_eq!(c.count("api"), Ok(60_000_000_000));
    assert_eq!(c.count_in_last("api", 600_000), Ok(10_000_000_000));
    assert!(c.memory_probe() <= 100_000);
}
