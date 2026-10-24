#![allow(unused_imports, clippy::needless_range_loop)]
use p121_concurrent_bloom_filter::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

type Filter = Arc<dyn ConcurrentBloomFilter>;

macro_rules! filter {
    ($n:expr, $p:expr) => {
        sut!(create($n, $p))
    };
}

fn items(prefix: &str, from: usize, to: usize) -> Vec<String> {
    (from..to).map(|i| format!("{prefix}{i}")).collect()
}

fn has(f: &Filter, s: &str) -> bool {
    f.might_contain(s).unwrap()
}

#[test]
fn should_report_a_saturated_filter_as_everything_possibly_present() {
    let f = filter!(1, 0.5); // m = 2, k = 1
    assert_eq!(f.bit_count(), 2);
    assert_eq!(f.hash_count(), 1);

    f.add_all(&items("fill-", 0, 200)).unwrap();

    assert_eq!(f.set_bits(), 2);
    assert_eq!(f.approximate_size(), u64::MAX);
    assert!(has(&f, "never-added"));
    assert_eq!(f.add("one-more"), Ok(false));
}

#[test]
fn should_stay_consistent_when_many_threads_add_overlapping_batches() {
    let threads = 16;
    let pool = items("o-", 0, 6_000);
    let concurrent = filter!(6_000, 0.01);
    let sequential = filter!(6_000, 0.01);

    run_concurrently(threads, |t| {
        let mut mine = pool.clone();
        let n = mine.len();
        for i in 0..n {
            mine.swap(i, (i * 31 + t * 977 + 7) % n);
        }
        concurrent.add_all(&mine).unwrap();
    });
    sequential.add_all(&pool).unwrap();

    assert_eq!(concurrent.set_bits(), sequential.set_bits());
    for s in &pool {
        assert!(has(&concurrent, s));
    }
}

#[test]
fn should_keep_the_rate_for_several_parameter_sets() {
    for (n, percent) in [(2_000usize, 1usize), (5_000, 5), (1_000, 10)] {
        let rate = percent as f64 / 100.0;
        let f = filter!(n, rate);
        f.add_all(&items("in-", 0, n)).unwrap();

        let fp = (0..100_000).filter(|i| has(&f, &format!("out-{i}"))).count();

        assert!(fp <= (3.0 * rate * 100_000.0) as usize, "n={n} rate={rate}: {fp}");
    }
}

#[test]
fn should_estimate_the_size_across_fill_levels() {
    let f = filter!(20_000, 0.01);
    let mut added = 0;
    for target in [100usize, 1_000, 5_000, 10_000, 20_000] {
        f.add_all(&items("g-", added, target)).unwrap();
        added = target;

        let est = f.approximate_size() as f64;
        assert!(est >= target as f64 * 0.95 && est <= target as f64 * 1.05, "after {target}: {est}");
    }
}

#[test]
fn should_handle_very_long_and_odd_items() {
    let f = filter!(1_000, 1e-9);
    let long_item = "x".repeat(200_000);
    let long_other = format!("{}y", "x".repeat(199_999));

    assert_eq!(f.add(&long_item), Ok(true));
    assert!(has(&f, &long_item));
    assert!(!has(&f, &long_other));
    assert_eq!(f.add("a\u{0}b"), Ok(true));
    assert!(!has(&f, "a"));
    assert_eq!(f.add("\n x \n"), Ok(true));
    assert!(!has(&f, "x"));
}

#[test]
fn should_keep_the_estimate_monotone_while_it_is_read_concurrently() {
    let f = filter!(30_000, 0.01);
    let decreased = AtomicBool::new(false);

    run_concurrently(5, |t| {
        if t < 4 {
            for i in 0..7_500 {
                f.add(&format!("e-{t}-{i}")).unwrap();
            }
        } else {
            let mut previous = 0;
            for _ in 0..2_000 {
                let now = f.approximate_size();
                if now < previous {
                    decreased.store(true, Ordering::SeqCst);
                }
                previous = now;
            }
        }
    });

    assert!(!decreased.load(Ordering::SeqCst), "estimate went down while only adding");
    let est = f.approximate_size();
    assert!((28_000..=32_000).contains(&est), "{est}");
}

#[test]
fn should_report_each_distinct_item_once_across_threads_in_a_roomy_filter() {
    let threads = 8;
    let f = filter!(4_000, 1e-9);
    let changed = AtomicUsize::new(0);

    run_concurrently(threads, |t| {
        for i in 0..500 {
            if f.add(&format!("r-{t}-{i}")).unwrap() {
                changed.fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    assert_eq!(changed.load(Ordering::SeqCst), threads * 500);
}

#[test]
fn should_work_again_after_clear_under_concurrent_use() {
    let f = filter!(10_000, 0.01);
    f.add_all(&items("old-", 0, 5_000)).unwrap();
    f.clear();

    run_concurrently(8, |t| {
        for i in 0..1_000 {
            f.add(&format!("new-{t}-{i}")).unwrap();
        }
    });

    for t in 0..8 {
        for i in 0..1_000 {
            assert!(has(&f, &format!("new-{t}-{i}")));
        }
    }
    let est = f.approximate_size();
    assert!((7_600..=8_400).contains(&est), "{est}");
    let still_there = items("old-", 0, 5_000).iter().filter(|s| has(&f, s)).count();
    assert!(still_there < 300, "only false positives expected, got {still_there}"); 
}
