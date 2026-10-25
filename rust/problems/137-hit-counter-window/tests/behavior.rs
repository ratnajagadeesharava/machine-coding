#![allow(unused_imports, clippy::needless_range_loop)]
use p137_hit_counter_window::{contract::*, create};
use playground_support::{run_concurrently, sut, FakeClock};
use std::sync::Arc;

const BAD: Result<u64, CounterError> = Err(CounterError::InvalidArgument);

fn clock() -> FakeClock {
    FakeClock::new(10_000)
}

#[test]
fn should_count_hits_inside_the_window() {
    let clock = clock();
    let c = sut!(create(1_000, clock.as_clock()));

    c.hit("a").unwrap();
    clock.advance(100);
    c.hit("a").unwrap();
    c.hit("b").unwrap();

    assert_eq!(c.count("a"), Ok(2));
    assert_eq!(c.count("b"), Ok(1));
}

#[test]
fn should_count_an_unknown_key_as_zero() {
    let clock = clock();
    let c = sut!(create(1_000, clock.as_clock()));
    c.hit("a").unwrap();

    assert_eq!(c.count("nobody"), Ok(0));
    assert_eq!(c.count_in_last("nobody", 500), Ok(0));
    let empty = create(1_000, clock.as_clock()).unwrap();
    assert_eq!(empty.total_in_window(), 0);
}

#[test]
fn should_expire_a_hit_exactly_window_ms_old() {
    let clock = clock();
    let c = sut!(create(1_000, clock.as_clock()));
    c.hit("a").unwrap();

    clock.advance(999);
    assert_eq!(c.count("a"), Ok(1));
    clock.advance(1);
    assert_eq!(c.count("a"), Ok(0));
}

#[test]
fn should_record_several_hits_with_one_call() {
    let clock = clock();
    let c = sut!(create(1_000, clock.as_clock()));

    c.hit_n("a", 5).unwrap();
    c.hit("a").unwrap();
    c.hit_n("a", 1_000_000_000_000).unwrap();

    assert_eq!(c.count("a"), Ok(1_000_000_000_006));
    assert_eq!(c.total_in_window(), 1_000_000_000_006);
}

#[test]
fn should_expire_the_old_hits_and_keep_the_new_ones() {
    let clock = clock();
    let c = sut!(create(1_000, clock.as_clock()));
    c.hit_n("a", 3).unwrap();
    clock.advance(600);
    c.hit_n("a", 2).unwrap();

    clock.advance(500);

    assert_eq!(c.count("a"), Ok(2));
    clock.advance(500);
    assert_eq!(c.count("a"), Ok(0));
}

#[test]
fn should_reject_invalid_arguments() {
    let clock = clock();
    let c = sut!(create(1_000, clock.as_clock()));
    let bad = Err(CounterError::InvalidArgument);

    assert!(create(0, clock.as_clock()).is_none());
    assert_eq!(c.hit(" "), bad);
    assert_eq!(c.hit(""), bad);
    assert_eq!(c.hit_n("a", 0), bad);
    assert_eq!(c.hit_n("", 2), bad);
    assert_eq!(c.count("  "), BAD);
    assert_eq!(c.count_in_last(" ", 10), BAD);
    assert_eq!(c.set_window(0), bad);
    assert_eq!(c.top_keys(0), Err(CounterError::InvalidArgument));
}

#[test]
fn should_change_nothing_when_an_argument_is_rejected() {
    let clock = clock();
    let c = sut!(create(1_000, clock.as_clock()));
    c.hit("a").unwrap();

    assert!(c.hit_n("a", 0).is_err());
    assert!(c.set_window(0).is_err());

    assert_eq!(c.count("a"), Ok(1));
    assert_eq!(c.window(), 1_000);
}

#[test]
fn should_answer_shorter_queries_with_count_in_last() {
    let clock = clock();
    let c = sut!(create(60_000, clock.as_clock()));
    c.hit_n("a", 4).unwrap();
    clock.advance(30_000);
    c.hit_n("a", 2).unwrap();
    clock.advance(25_000);
    c.hit_n("a", 1).unwrap();

    assert_eq!(c.count_in_last("a", 5_000), Ok(1));
    assert_eq!(c.count_in_last("a", 25_000), Ok(1));
    assert_eq!(c.count_in_last("a", 25_001), Ok(3));
    assert_eq!(c.count_in_last("a", 60_000), Ok(7));
    assert_eq!(c.count("a"), Ok(7));
}

#[test]
fn should_treat_a_hit_exactly_duration_old_as_outside_count_in_last() {
    let clock = clock();
    let c = sut!(create(10_000, clock.as_clock()));
    c.hit("a").unwrap();
    clock.advance(5_000);

    assert_eq!(c.count_in_last("a", 5_000), Ok(0));
    assert_eq!(c.count_in_last("a", 5_001), Ok(1));
}

#[test]
fn should_reject_durations_outside_zero_to_window() {
    let clock = clock();
    let c = sut!(create(1_000, clock.as_clock()));
    c.hit("a").unwrap();

    assert_eq!(c.count_in_last("a", 0), BAD);
    assert_eq!(c.count_in_last("a", 1_001), BAD);
    assert_eq!(c.count_in_last("unknown", 1_001), BAD);
    assert_eq!(c.count_in_last("a", 1_000), Ok(1));
    assert_eq!(c.count_in_last("a", 1), Ok(1));
}

#[test]
fn should_sum_the_windowed_hits_of_all_keys() {
    let clock = clock();
    let c = sut!(create(1_000, clock.as_clock()));
    c.hit_n("a", 2).unwrap();
    c.hit_n("b", 3).unwrap();
    clock.advance(600);
    c.hit_n("c", 4).unwrap();

    assert_eq!(c.total_in_window(), 9);
    clock.advance(500);
    assert_eq!(c.total_in_window(), 4);
    clock.advance(500);
    assert_eq!(c.total_in_window(), 0);
}

#[test]
fn should_work_with_a_one_millisecond_window() {
    let clock = clock();
    let c = sut!(create(1, clock.as_clock()));

    c.hit_n("a", 2).unwrap();
    assert_eq!(c.count("a"), Ok(2));
    assert_eq!(c.count_in_last("a", 1), Ok(2));
    clock.advance(1);
    assert_eq!(c.count("a"), Ok(0));
}

#[test]
fn should_drop_hits_outside_the_new_window_when_shrinking_and_not_bring_them_back() {
    let clock = clock();
    let c = sut!(create(10_000, clock.as_clock()));
    c.hit_n("a", 1).unwrap();
    clock.advance(3_000);
    c.hit_n("a", 10).unwrap();
    clock.advance(3_000);
    c.hit_n("a", 100).unwrap();
    assert_eq!(c.count("a"), Ok(111));

    c.set_window(4_000).unwrap();
    assert_eq!(c.window(), 4_000);
    assert_eq!(c.count("a"), Ok(110));

    c.set_window(10_000).unwrap();
    assert_eq!(c.window(), 10_000);
    assert_eq!(c.count("a"), Ok(110), "the dropped hit stays dropped");
    assert_eq!(c.total_in_window(), 110);
}

#[test]
fn should_keep_the_live_hits_and_accept_longer_queries_when_growing() {
    let clock = clock();
    let c = sut!(create(1_000, clock.as_clock()));
    c.hit_n("a", 2).unwrap();
    clock.advance(500);

    c.set_window(5_000).unwrap();

    assert_eq!(c.count("a"), Ok(2));
    clock.advance(3_000);
    assert_eq!(c.count_in_last("a", 4_000), Ok(2));
    assert_eq!(c.count("a"), Ok(2));
    clock.advance(1_500);
    assert_eq!(c.count("a"), Ok(0));
}

#[test]
fn should_not_resurrect_hits_that_expired_before_the_window_grew() {
    let clock = clock();
    let c = sut!(create(1_000, clock.as_clock()));
    c.hit_n("a", 7).unwrap();
    clock.advance(1_500);

    c.set_window(5_000).unwrap();

    assert_eq!(c.count("a"), Ok(0));
    assert_eq!(c.total_in_window(), 0);
}

#[test]
fn should_apply_the_new_window_limit_to_count_in_last() {
    let clock = clock();
    let c = sut!(create(1_000, clock.as_clock()));
    c.hit("a").unwrap();
    c.set_window(500).unwrap();

    assert_eq!(c.count_in_last("a", 501), BAD);
    assert_eq!(c.count_in_last("a", 500), Ok(1));
    c.set_window(2_000).unwrap();
    assert_eq!(c.count_in_last("a", 2_000), Ok(1));
}

#[test]
fn should_rank_top_keys_by_count_then_by_key() {
    let clock = clock();
    let c = sut!(create(1_000, clock.as_clock()));
    c.hit_n("pear", 3).unwrap();
    c.hit_n("apple", 3).unwrap();
    c.hit_n("zebra", 9).unwrap();
    c.hit_n("mango", 1).unwrap();
    c.hit_n("fig", 3).unwrap();

    assert_eq!(
        c.top_keys(10).unwrap(),
        vec!["zebra", "apple", "fig", "pear", "mango"]
    );
    assert_eq!(c.top_keys(2).unwrap(), vec!["zebra", "apple"]);
    assert_eq!(c.top_keys(1).unwrap(), vec!["zebra"]);
}

#[test]
fn should_list_only_keys_with_hits_in_the_window_in_top_keys() {
    let clock = clock();
    let c = sut!(create(1_000, clock.as_clock()));
    assert!(c.top_keys(5).unwrap().is_empty());

    c.hit_n("old", 100).unwrap();
    clock.advance(600);
    c.hit("new").unwrap();
    assert_eq!(c.top_keys(5).unwrap(), vec!["old", "new"]);

    clock.advance(500);
    assert_eq!(c.top_keys(5).unwrap(), vec!["new"]);
    clock.advance(500);
    assert!(c.top_keys(5).unwrap().is_empty());
}

#[test]
fn should_record_hits_at_the_last_seen_time_when_the_clock_moves_backwards() {
    let clock = clock();
    let c = sut!(create(1_000, clock.as_clock()));
    c.hit("a").unwrap();

    clock.set(9_000);
    c.hit("a").unwrap();
    c.hit_n("a", 2).unwrap();

    assert_eq!(c.count("a"), Ok(4), "all hits were recorded at 10000");
    clock.set(10_999);
    assert_eq!(c.count("a"), Ok(4));
    clock.set(11_000);
    assert_eq!(c.count("a"), Ok(0));
}

#[test]
fn should_answer_queries_at_the_last_seen_time_when_the_clock_moves_backwards() {
    let clock = clock();
    let c = sut!(create(1_000, clock.as_clock()));
    clock.set(20_000);
    c.hit("a").unwrap();

    clock.set(5_000);

    assert_eq!(c.count("a"), Ok(1));
    assert_eq!(c.total_in_window(), 1);
    assert_eq!(c.top_keys(1).unwrap(), vec!["a"]);
    clock.set(20_500);
    assert_eq!(c.count("a"), Ok(1));
    clock.set(5_000);
    c.hit("a").unwrap();
    assert_eq!(c.count("a"), Ok(2), "now stayed at 20500");
    clock.set(21_000);
    assert_eq!(c.count("a"), Ok(1), "only the hit at 20500 is left");
}

#[test]
fn should_have_nothing_stored_before_the_first_hit() {
    let clock = clock();
    let c = sut!(create(1_000, clock.as_clock()));
    assert_eq!(c.memory_probe(), 0);
}

#[test]
fn should_not_keep_one_record_per_hit() {
    let clock = clock();
    let c = sut!(create(2_000, clock.as_clock()));
    for _ in 0..1_000 {
        for _ in 0..5_000 {
            c.hit("busy").unwrap();
        }
        clock.advance(1);
    }

    assert_eq!(c.count("busy"), Ok(5_000_000));
    assert!(c.memory_probe() <= 100_000, "got {}", c.memory_probe());

    clock.advance(5_000);

    assert_eq!(c.count("busy"), Ok(0));
    assert!(c.memory_probe() <= 100_000, "got {}", c.memory_probe());
}

#[test]
fn should_lose_no_hit_when_many_threads_hit_the_same_key() {
    let clock = clock();
    let c = sut!(create(60_000, clock.as_clock()));

    run_concurrently(16, |_| {
        for _ in 0..10_000 {
            c.hit("shared").unwrap();
        }
    });

    assert_eq!(c.count("shared"), Ok(160_000));
    assert_eq!(c.total_in_window(), 160_000);
}

#[test]
fn should_lose_no_hit_when_many_threads_hit_many_keys_with_batches() {
    let clock = clock();
    let c = sut!(create(60_000, clock.as_clock()));

    run_concurrently(16, |t| {
        for i in 0..2_000 {
            c.hit_n(&format!("k{}", i % 10), 1 + (t as u64 % 3)).unwrap();
        }
    });

    let expected: u64 = (0..16u64).map(|t| 2_000 * (1 + t % 3)).sum();
    assert_eq!(c.total_in_window(), expected);
    let sum: u64 = (0..10).map(|k| c.count(&format!("k{k}")).unwrap()).sum();
    assert_eq!(sum, expected);
}

#[test]
fn should_stay_correct_over_50_runs() {
    for run in 0..50 {
        let clock = FakeClock::new(0);
        let c = sut!(create(1_000, clock.as_clock()));

        run_concurrently(16, |t| {
            for _ in 0..200 {
                c.hit(&format!("k{}", t % 4)).unwrap();
            }
        });

        assert_eq!(c.total_in_window(), 3_200, "run {run}");
        assert_eq!(c.count("k0"), Ok(800), "run {run}");
    }
}
