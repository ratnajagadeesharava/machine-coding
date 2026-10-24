#![allow(unused_imports, clippy::needless_range_loop)]
use p121_concurrent_bloom_filter::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

type Filter = Arc<dyn ConcurrentBloomFilter>;

macro_rules! filter {
    ($n:expr, $p:expr) => {
        sut!(create($n, $p))
    };
}

/// Very low false positive rate: for a handful of items, "might contain" is effectively exact.
macro_rules! roomy {
    () => {
        filter!(1_000, 1e-9)
    };
}

fn items(prefix: &str, from: usize, to: usize) -> Vec<String> {
    (from..to).map(|i| format!("{prefix}{i}")).collect()
}

fn expected_size(m: u64, k: u32, x: u64) -> i64 {
    if x == 0 {
        return 0;
    }
    (-(m as f64 / k as f64) * (1.0 - x as f64 / m as f64).ln()).round() as i64
}

fn has(f: &Filter, s: &str) -> bool {
    f.might_contain(s).unwrap()
}

// ---------- sizing ----------

#[test]
fn should_size_a_thousand_items_at_one_percent() {
    let f = filter!(1_000, 0.01);

    assert_eq!(f.bit_count(), 9_586);
    assert_eq!(f.hash_count(), 7);
}

#[test]
fn should_size_other_parameter_sets_with_the_documented_formulas() {
    let f = filter!(10_000, 0.01);
    assert_eq!((f.bit_count(), f.hash_count()), (95_851, 7));
    let f = filter!(100, 0.001);
    assert_eq!((f.bit_count(), f.hash_count()), (1_438, 10));
    let f = filter!(500, 0.05);
    assert_eq!((f.bit_count(), f.hash_count()), (3_118, 4));
    let f = filter!(100, 1e-9);
    assert_eq!((f.bit_count(), f.hash_count()), (4_314, 30));
    let f = filter!(1_000_000, 0.001);
    assert_eq!((f.bit_count(), f.hash_count()), (14_377_588, 10));
}

#[test]
fn should_use_at_least_one_hash_even_for_a_very_loose_rate() {
    let f = filter!(10, 0.9);

    assert_eq!(f.bit_count(), 3);
    assert_eq!(f.hash_count(), 1);
}

#[test]
fn should_round_the_hash_count_to_the_nearest_integer() {
    // m/n*ln2 = 1.386 -> 1 ; 3.327 -> 3 ; 4.322 -> 4 ; 6.644 -> 7
    assert_eq!(filter!(1, 0.5).hash_count(), 1);
    assert_eq!(filter!(50, 0.1).hash_count(), 3);
    assert_eq!(filter!(500, 0.05).hash_count(), 4);
    assert_eq!(filter!(1_000, 0.01).hash_count(), 7);
}

#[test]
fn should_reject_invalid_expected_items() {
    let _valid = filter!(10, 0.01); // proves the factory is wired

    assert!(create(0, 0.01).is_none());
}

#[test]
fn should_reject_invalid_false_positive_rates() {
    let _valid = filter!(10, 0.01); // proves the factory is wired

    for bad in [0.0, -0.1, 1.0, 1.5, f64::NAN, f64::INFINITY] {
        assert!(create(10, bad).is_none(), "rate {bad}");
    }
}

// ---------- add / might_contain ----------

#[test]
fn should_start_empty() {
    let f = roomy!();

    assert_eq!(f.set_bits(), 0);
    assert_eq!(f.approximate_size(), 0);
    assert!(!has(&f, "anything"));
}

#[test]
fn should_report_a_new_item_as_changed_and_present() {
    let f = roomy!();

    assert_eq!(f.add("apple"), Ok(true));
    assert!(has(&f, "apple"));
}

#[test]
fn should_report_an_already_added_item_as_unchanged() {
    let f = roomy!();
    f.add("apple").unwrap();
    let bits = f.set_bits();

    assert_eq!(f.add("apple"), Ok(false));
    assert_eq!(f.add("apple"), Ok(false));
    assert_eq!(f.set_bits(), bits);
    assert!(has(&f, "apple"));
}

#[test]
fn should_set_between_one_and_k_bits_for_a_single_item() {
    let f = filter!(1_000, 0.01);

    f.add("apple").unwrap();

    assert!(f.set_bits() >= 1 && f.set_bits() <= f.hash_count() as u64);
}

#[test]
fn should_not_change_the_filter_when_probing() {
    let f = roomy!();
    f.add("apple").unwrap();
    let bits = f.set_bits();
    let size = f.approximate_size();

    for i in 0..200 {
        f.might_contain(&format!("probe-{i}")).unwrap();
    }

    assert_eq!(f.set_bits(), bits);
    assert_eq!(f.approximate_size(), size);
    assert_eq!(f.add("probe-0"), Ok(true)); // probing did not add it
}

#[test]
fn should_treat_items_as_case_sensitive_and_untrimmed() {
    let f = roomy!();
    f.add("Apple").unwrap();

    assert!(has(&f, "Apple"));
    assert!(!has(&f, "apple"));
    assert!(!has(&f, "APPLE"));
    assert_eq!(f.add(" Apple "), Ok(true));
    assert!(has(&f, " Apple "));
    assert_eq!(f.add("apple"), Ok(true));
}

#[test]
fn should_handle_non_ascii_items() {
    let f = roomy!();

    assert_eq!(f.add("école"), Ok(true));
    assert_eq!(f.add("日本語"), Ok(true));
    assert_eq!(f.add("\u{1F600} smile"), Ok(true));
    assert!(has(&f, "école"));
    assert!(has(&f, "日本語"));
    assert!(has(&f, "\u{1F600} smile"));
    assert!(!has(&f, "Ecole"));
    assert!(!has(&f, "ecole"));
}

#[test]
fn should_reject_blank_items() {
    let f = roomy!();

    for bad in ["", "   ", "\t\n "] {
        assert_eq!(f.add(bad), Err(BloomError::InvalidItem), "add {bad:?}");
        assert_eq!(f.might_contain(bad), Err(BloomError::InvalidItem), "might_contain {bad:?}");
    }
}

#[test]
fn should_change_nothing_when_an_item_is_rejected() {
    let f = roomy!();
    f.add("apple").unwrap();
    let bits = f.set_bits();

    assert_eq!(f.add("  "), Err(BloomError::InvalidItem));

    assert_eq!(f.set_bits(), bits);
    assert_eq!(f.approximate_size(), 1);
}

#[test]
fn should_report_every_new_item_as_changed_in_a_roomy_filter() {
    let f = roomy!();

    let changed = items("n-", 0, 300).iter().filter(|s| f.add(s).unwrap()).count();

    assert_eq!(changed, 300);
    assert_eq!(f.add("n-0"), Ok(false));
    assert_eq!(f.add("n-299"), Ok(false));
}

#[test]
fn should_keep_two_filters_independent() {
    let a = roomy!();
    let b = roomy!();

    a.add("apple").unwrap();

    assert!(has(&a, "apple"));
    assert!(!has(&b, "apple"));
    assert_eq!(b.set_bits(), 0);
}

// ---------- add_all ----------

#[test]
fn should_count_the_items_that_changed_the_filter_in_add_all() {
    let f = roomy!();
    f.add("a").unwrap();
    let batch: Vec<String> = ["a", "b", "c", "b", "d"].iter().map(|s| s.to_string()).collect();

    // b, c, d; the repeated b and the earlier a changed nothing
    assert_eq!(f.add_all(&batch), Ok(3));
    for s in ["a", "b", "c", "d"] {
        assert!(has(&f, s));
    }
}

#[test]
fn should_add_nothing_when_any_element_of_add_all_is_invalid() {
    let f = roomy!();
    let batch: Vec<String> = ["a", "b", " "].iter().map(|s| s.to_string()).collect();

    assert_eq!(f.add_all(&batch), Err(BloomError::InvalidItem));

    assert_eq!(f.set_bits(), 0);
    assert!(!has(&f, "a"));
}

#[test]
fn should_accept_an_empty_add_all() {
    let f = roomy!();

    assert_eq!(f.add_all(&[]), Ok(0));
    assert_eq!(f.set_bits(), 0);
}

// ---------- approximate_size ----------

#[test]
fn should_estimate_the_size_from_the_number_of_set_bits() {
    let f = filter!(1_000, 0.01);
    for step in 0..5 {
        f.add_all(&items(&format!("e-{step}-"), 0, 100)).unwrap();

        let expected = expected_size(f.bit_count(), f.hash_count(), f.set_bits());
        let got = f.approximate_size() as i64;
        assert!((expected - 1..=expected + 1).contains(&got), "got {got}, expected {expected}");
    }
}

#[test]
fn should_estimate_the_number_of_distinct_items() {
    let f = filter!(1_000, 0.01);
    f.add_all(&items("d-", 0, 1_000)).unwrap();
    f.add_all(&items("d-", 0, 1_000)).unwrap(); // duplicates do not count

    assert!((950..=1_050).contains(&f.approximate_size()), "got {}", f.approximate_size());
}

#[test]
fn should_estimate_one_for_a_single_item() {
    let f = filter!(1_000, 0.01);

    f.add("only").unwrap();

    assert_eq!(f.approximate_size(), 1);
}

#[test]
fn should_never_decrease_the_estimate_while_adding() {
    let f = filter!(500, 0.01);
    let mut previous = 0;
    for i in 0..500 {
        f.add(&format!("m-{i}")).unwrap();
        let now = f.approximate_size();
        assert!(now >= previous);
        previous = now;
    }
}

// ---------- clear ----------

#[test]
fn should_forget_everything_on_clear() {
    let f = roomy!();
    f.add_all(&items("c-", 0, 50)).unwrap();

    f.clear();

    assert_eq!(f.set_bits(), 0);
    assert_eq!(f.approximate_size(), 0);
    for s in items("c-", 0, 50) {
        assert!(!has(&f, &s));
    }
    assert_eq!(f.bit_count(), 43_133);
    assert_eq!(f.hash_count(), 30);
}

#[test]
fn should_be_usable_again_after_clear() {
    let f = roomy!();
    f.add("apple").unwrap();
    f.clear();

    assert_eq!(f.add("apple"), Ok(true));
    assert!(has(&f, "apple"));
    assert_eq!(f.approximate_size(), 1);
    f.clear();
    f.clear(); // clearing an empty filter is fine
    assert_eq!(f.set_bits(), 0);
}

// ---------- hashing ----------

#[test]
fn should_give_the_same_bits_regardless_of_insertion_order_and_instance() {
    let all = items("h-", 0, 2_000);
    let mut shuffled = all.clone();
    // deterministic permutation: stride walk over a prime-length-free index space
    let n = shuffled.len();
    for i in 0..n {
        shuffled.swap(i, (i * 7919 + 13) % n);
    }
    let a = filter!(2_000, 0.01);
    let b = filter!(2_000, 0.01);

    a.add_all(&all).unwrap();
    b.add_all(&shuffled).unwrap();

    assert_eq!(b.set_bits(), a.set_bits());
    assert_eq!(b.approximate_size(), a.approximate_size());
    for i in 0..5_000 {
        let probe = format!("probe-{i}");
        assert_eq!(has(&b, &probe), has(&a, &probe), "{probe}");
    }
}

// ---------- accuracy ----------

#[test]
fn should_have_no_false_negatives() {
    let f = filter!(100_000, 0.01);
    let all = items("item-", 0, 100_000);

    f.add_all(&all).unwrap();

    for s in &all {
        assert!(has(&f, s), "{s}");
    }
}

#[test]
fn should_keep_the_false_positive_rate_within_three_times_the_configured_rate() {
    let f = filter!(10_000, 0.01);
    f.add_all(&items("item-", 0, 10_000)).unwrap();

    let false_positives = (0..100_000).filter(|i| has(&f, &format!("probe-{i}"))).count();

    assert!(false_positives <= 3_000, "{false_positives} false positives");
}

#[test]
fn should_keep_the_false_positive_rate_for_a_tighter_rate() {
    let f = filter!(20_000, 0.001);
    f.add_all(&items("user:", 0, 20_000)).unwrap();

    let false_positives = (0..100_000).filter(|i| has(&f, &format!("other:{i}"))).count();

    assert!(false_positives <= 300, "{false_positives} false positives");
}

// ---------- concurrency ----------

#[test]
fn should_have_no_false_negatives_when_threads_add_disjoint_ranges() {
    let threads = 8;
    let per_thread = 12_500;
    let f = filter!(threads * per_thread, 0.01);

    run_concurrently(threads, |t| {
        for i in 0..per_thread {
            f.add(&format!("t{t}-{i}")).unwrap();
        }
    });

    for t in 0..threads {
        for i in 0..per_thread {
            assert!(has(&f, &format!("t{t}-{i}")), "t{t}-{i}");
        }
    }
}

#[test]
fn should_end_with_the_same_bits_as_a_sequential_run_after_concurrent_adds() {
    let threads = 8;
    let per_thread = 5_000;
    let concurrent = filter!(threads * per_thread, 0.01);
    let sequential = filter!(threads * per_thread, 0.01);

    run_concurrently(threads, |t| {
        concurrent.add_all(&items(&format!("t{t}-"), 0, per_thread)).unwrap();
    });
    for t in 0..threads {
        sequential.add_all(&items(&format!("t{t}-"), 0, per_thread)).unwrap();
    }

    assert_eq!(concurrent.set_bits(), sequential.set_bits());
    assert_eq!(concurrent.approximate_size(), sequential.approximate_size());
}

#[test]
fn should_end_with_the_same_bits_when_threads_add_the_same_items() {
    let shared = items("s-", 0, 5_000);
    let concurrent = filter!(5_000, 0.01);
    let sequential = filter!(5_000, 0.01);

    run_concurrently(8, |_| {
        for s in &shared {
            concurrent.add(s).unwrap();
        }
    });
    sequential.add_all(&shared).unwrap();

    assert_eq!(concurrent.set_bits(), sequential.set_bits());
}

#[test]
fn should_report_at_least_one_change_and_at_most_k_when_threads_add_the_same_new_item() {
    for _ in 0..20 {
        let f = roomy!();
        let changed = AtomicUsize::new(0);

        run_concurrently(16, |_| {
            if f.add("contested").unwrap() {
                changed.fetch_add(1, Ordering::SeqCst);
            }
        });

        let k = f.hash_count() as usize;
        assert!((1..=k).contains(&changed.load(Ordering::SeqCst)));
        assert!(f.set_bits() >= 1 && f.set_bits() <= k as u64);
        assert_eq!(f.add("contested"), Ok(false));
    }
}

#[test]
fn should_never_show_an_added_item_as_absent_once_add_has_returned() {
    let total = 20_000usize;
    let f = filter!(total, 0.01);
    let published = AtomicUsize::new(0); // items [0, published) have been added and add() returned
    let missing = AtomicU64::new(0);

    run_concurrently(4, |t| {
        if t == 0 {
            for i in 0..total {
                f.add(&format!("w-{i}")).unwrap();
                published.store(i + 1, Ordering::SeqCst);
            }
        } else {
            let mut checked = 0;
            loop {
                let bound = published.load(Ordering::SeqCst);
                while checked < bound {
                    if !has(&f, &format!("w-{checked}")) {
                        missing.fetch_add(1, Ordering::SeqCst);
                    }
                    checked += 1;
                }
                // re-check the newest published item right away, it is the most likely to be racing
                if bound > 0 && !has(&f, &format!("w-{}", bound - 1)) {
                    missing.fetch_add(1, Ordering::SeqCst);
                }
                if bound >= total {
                    break;
                }
            }
        }
    });

    assert_eq!(missing.load(Ordering::SeqCst), 0);
}

#[test]
fn should_answer_probes_while_other_threads_add() {
    let f = filter!(40_000, 0.01);
    f.add_all(&items("base-", 0, 10_000)).unwrap();
    let missing = AtomicU64::new(0);

    run_concurrently(8, |t| {
        if t < 4 {
            for i in 0..7_500 {
                f.add(&format!("late-{t}-{i}")).unwrap();
            }
        } else {
            for _ in 0..3 {
                for i in 0..10_000 {
                    if !has(&f, &format!("base-{i}")) {
                        missing.fetch_add(1, Ordering::SeqCst);
                    }
                }
            }
        }
    });

    assert_eq!(missing.load(Ordering::SeqCst), 0);
    assert!(has(&f, "late-3-7499"));
}
