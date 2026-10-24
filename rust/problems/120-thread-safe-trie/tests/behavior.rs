#![allow(unused_imports, clippy::needless_range_loop)]
use p120_thread_safe_trie::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

fn with(t: &Arc<dyn WordSet>, words: &[&str]) {
    for w in words {
        t.insert(w).unwrap();
    }
}

/// A distinct word made only of allowed characters for every i.
fn word(i: usize) -> String {
    let digits = b"0123456789abcdefghijklmnopqrstuvwxyz";
    if i == 0 {
        return "0".into();
    }
    let mut n = i;
    let mut out = Vec::new();
    while n > 0 {
        out.push(digits[n % 36]);
        n /= 36;
    }
    out.reverse();
    String::from_utf8(out).unwrap()
}

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn should_report_whether_an_insert_added_the_word() {
    let t = sut!(create());

    assert_eq!(t.insert("apple"), Ok(true));
    assert_eq!(t.insert("apple"), Ok(false));
    assert_eq!(t.size(), 1);
}

#[test]
fn should_find_only_exact_words_with_contains() {
    let t = sut!(create());
    with(&t, &["apple", "app"]);

    assert_eq!(t.contains("apple"), Ok(true));
    assert_eq!(t.contains("app"), Ok(true));
    assert_eq!(t.contains("ap"), Ok(false));
    assert_eq!(t.contains("apples"), Ok(false));
    assert_eq!(t.contains("banana"), Ok(false));
}

#[test]
fn should_answer_starts_with_for_prefixes_and_whole_words() {
    let t = sut!(create());
    with(&t, &["apple"]);

    assert_eq!(t.starts_with("a"), Ok(true));
    assert_eq!(t.starts_with("appl"), Ok(true));
    assert_eq!(t.starts_with("apple"), Ok(true));
    assert_eq!(t.starts_with("apples"), Ok(false));
    assert_eq!(t.starts_with("b"), Ok(false));
}

#[test]
fn should_treat_the_empty_prefix_as_everything() {
    let e = sut!(create());
    assert_eq!(e.starts_with(""), Ok(false));
    assert_eq!(e.count_with_prefix(""), Ok(0));
    assert_eq!(e.words_with_prefix("", 10), Ok(vec![]));

    let t = sut!(create());
    with(&t, &["b", "a", "c"]);
    assert_eq!(t.starts_with(""), Ok(true));
    assert_eq!(t.starts_with("   "), Ok(true));
    assert_eq!(t.count_with_prefix(""), Ok(3));
    assert_eq!(t.words_with_prefix("", 10), Ok(strs(&["a", "b", "c"])));
}

#[test]
fn should_normalise_words_and_prefixes_by_trimming_and_lower_casing() {
    let t = sut!(create());

    assert_eq!(t.insert("  Hello "), Ok(true));
    assert_eq!(t.insert("HELLO"), Ok(false));
    assert_eq!(t.contains("hello"), Ok(true));
    assert_eq!(t.contains(" HeLLo  "), Ok(true));
    assert_eq!(t.starts_with(" HEL "), Ok(true));
    assert_eq!(t.count_with_prefix("HE"), Ok(1));
    assert_eq!(t.words_with_prefix(" H", 5), Ok(strs(&["hello"])));
    assert_eq!(t.delete(" HELLO "), Ok(true));
    assert_eq!(t.size(), 0);
}

#[test]
fn should_accept_digits_and_hyphens() {
    let t = sut!(create());
    with(&t, &["covid-19", "x-ray", "2024", "-"]);

    assert_eq!(t.contains("covid-19"), Ok(true));
    assert_eq!(t.words_with_prefix("covid-", 5), Ok(strs(&["covid-19"])));
    assert_eq!(
        t.words_with_prefix("", 10),
        Ok(strs(&["-", "2024", "covid-19", "x-ray"]))
    );
}

#[test]
fn should_reject_blank_and_forbidden_characters() {
    let t = sut!(create());
    with(&t, &["ok"]);
    let bad = Some(TrieError::InvalidArgument);

    assert_eq!(t.insert("").err(), bad);
    assert_eq!(t.insert("   ").err(), bad);
    assert_eq!(t.insert("two words").err(), bad);
    assert_eq!(t.insert("under_score").err(), bad);
    assert_eq!(t.insert("caf\u{e9}").err(), bad);
    assert_eq!(t.insert("a.b").err(), bad);
    assert_eq!(t.contains(" ").err(), bad);
    assert_eq!(t.contains("a b").err(), bad);
    assert_eq!(t.delete("").err(), bad);
    assert_eq!(t.delete("a*").err(), bad);
    assert_eq!(t.starts_with("a b").err(), bad);
    assert_eq!(t.count_with_prefix("a!").err(), bad);
    assert_eq!(t.words_with_prefix("a/", 5).err(), bad);
}

#[test]
fn should_reject_a_zero_limit() {
    let t = sut!(create());
    with(&t, &["a"]);

    assert_eq!(
        t.words_with_prefix("a", 0),
        Err(TrieError::InvalidArgument)
    );
    assert_eq!(t.words_with_prefix("", 0), Err(TrieError::InvalidArgument));
}

#[test]
fn should_leave_the_trie_unchanged_when_a_call_is_rejected() {
    let t = sut!(create());
    with(&t, &["alpha"]);

    assert!(t.insert("al pha").is_err());
    assert!(t.insert("beta!").is_err());
    assert!(t.words_with_prefix("a", 0).is_err());

    assert_eq!(t.size(), 1);
    assert_eq!(t.starts_with("b"), Ok(false));
    assert_eq!(t.words_with_prefix("", 10), Ok(strs(&["alpha"])));
}

#[test]
fn should_count_words_with_a_prefix_including_the_word_itself() {
    let t = sut!(create());
    with(&t, &["car", "card", "care", "careful", "cat", "dog"]);

    assert_eq!(t.count_with_prefix("car"), Ok(4));
    assert_eq!(t.count_with_prefix("care"), Ok(2));
    assert_eq!(t.count_with_prefix("ca"), Ok(5));
    assert_eq!(t.count_with_prefix("c"), Ok(5));
    assert_eq!(t.count_with_prefix("dogs"), Ok(0));
    assert_eq!(t.count_with_prefix(""), Ok(6));
}

#[test]
fn should_list_words_in_lexicographic_order_regardless_of_insertion_order() {
    let t = sut!(create());
    with(
        &t,
        &["pear", "peach", "pea", "pecan", "apple", "pe", "p2", "p-x"],
    );

    // plain character order: '-' < digits < letters
    assert_eq!(
        t.words_with_prefix("p", 20),
        Ok(strs(&["p-x", "p2", "pe", "pea", "peach", "pear", "pecan"]))
    );
    assert_eq!(
        t.words_with_prefix("pea", 20),
        Ok(strs(&["pea", "peach", "pear"]))
    );
}

#[test]
fn should_apply_the_limit_to_the_smallest_words_first() {
    let t = sut!(create());
    with(&t, &["c", "a", "d", "b", "e"]);

    assert_eq!(t.words_with_prefix("", 3), Ok(strs(&["a", "b", "c"])));
    assert_eq!(t.words_with_prefix("", 1), Ok(strs(&["a"])));
    assert_eq!(
        t.words_with_prefix("", 100),
        Ok(strs(&["a", "b", "c", "d", "e"]))
    );
}

#[test]
fn should_return_an_empty_list_when_nothing_matches() {
    let t = sut!(create());
    with(&t, &["alpha"]);

    assert_eq!(t.words_with_prefix("b", 5), Ok(vec![]));
    assert_eq!(t.words_with_prefix("alphabet", 5), Ok(vec![]));
}

#[test]
fn should_return_a_fresh_list_the_caller_may_modify() {
    let t = sut!(create());
    with(&t, &["a", "b"]);

    let mut first = t.words_with_prefix("", 10).unwrap();
    first.push("zzz".into());
    first.retain(|w| w != "a");

    assert_eq!(t.words_with_prefix("", 10), Ok(strs(&["a", "b"])));
    assert_eq!(t.size(), 2);
}

#[test]
fn should_delete_only_the_given_word_and_keep_words_sharing_its_prefix() {
    let t = sut!(create());
    with(&t, &["app", "apple", "apply", "ape"]);

    assert_eq!(t.delete("app"), Ok(true));

    assert_eq!(t.contains("app"), Ok(false));
    assert_eq!(t.contains("apple"), Ok(true));
    assert_eq!(t.contains("apply"), Ok(true));
    assert_eq!(t.starts_with("app"), Ok(true));
    assert_eq!(
        t.words_with_prefix("ap", 10),
        Ok(strs(&["ape", "apple", "apply"]))
    );
    assert_eq!(t.size(), 3);
}

#[test]
fn should_keep_the_shorter_word_when_a_longer_one_is_deleted() {
    let t = sut!(create());
    with(&t, &["app", "apple"]);

    assert_eq!(t.delete("apple"), Ok(true));

    assert_eq!(t.contains("app"), Ok(true));
    assert_eq!(t.starts_with("appl"), Ok(false));
    assert_eq!(t.count_with_prefix("app"), Ok(1));
    assert_eq!(t.words_with_prefix("", 10), Ok(strs(&["app"])));
}

#[test]
fn should_return_false_when_deleting_a_missing_word_or_only_a_prefix() {
    let t = sut!(create());
    with(&t, &["apple"]);

    assert_eq!(t.delete("banana"), Ok(false));
    assert_eq!(t.delete("app"), Ok(false));
    assert_eq!(t.delete("apple"), Ok(true));
    assert_eq!(t.delete("apple"), Ok(false));
    assert_eq!(t.size(), 0);
    assert_eq!(t.starts_with("a"), Ok(false));
}

#[test]
fn should_allow_a_word_to_be_reinserted_after_deletion() {
    let t = sut!(create());
    with(&t, &["tea"]);

    assert_eq!(t.delete("tea"), Ok(true));
    assert_eq!(t.insert("tea"), Ok(true));

    assert_eq!(t.contains("tea"), Ok(true));
    assert_eq!(t.size(), 1);
}

#[test]
fn should_count_distinct_words_in_size() {
    let t = sut!(create());
    assert_eq!(t.size(), 0);

    with(&t, &["a", "b", "a", "ab", "B", " ab "]);
    assert_eq!(t.size(), 3);

    t.delete("a").unwrap();
    t.delete("zzz").unwrap();
    assert_eq!(t.size(), 2);
}

#[test]
fn should_handle_very_long_words() {
    let t = sut!(create());
    let long = format!("{}-9", "a".repeat(1_000));

    assert_eq!(t.insert(&long), Ok(true));
    assert_eq!(t.contains(&long), Ok(true));
    assert_eq!(t.starts_with(&"a".repeat(999)), Ok(true));
    assert_eq!(t.count_with_prefix("a"), Ok(1));
    assert_eq!(t.delete(&long), Ok(true));
    assert_eq!(t.starts_with("a"), Ok(false));
}

// ---------------------------------------------------------------- concurrency

#[test]
fn should_lose_no_words_when_many_threads_insert_disjoint_words() {
    let t = sut!(create());
    let threads = 8;
    let per_thread = 5_000;

    run_concurrently(threads, |id| {
        for i in 0..per_thread {
            assert_eq!(t.insert(&word(id * per_thread + i)), Ok(true));
        }
    });

    assert_eq!(t.size(), threads * per_thread);
    for i in (0..threads * per_thread).step_by(7) {
        assert_eq!(t.contains(&word(i)), Ok(true));
    }
}

#[test]
fn should_return_true_for_exactly_one_thread_when_the_same_word_is_inserted_concurrently() {
    let t = sut!(create());

    let wins = collect_concurrently(16, |_| {
        (0..500)
            .filter(|i| t.insert(&format!("shared-{i}")).unwrap())
            .count()
    });

    assert_eq!(wins.iter().sum::<usize>(), 500);
    assert_eq!(t.size(), 500);
}

#[test]
fn should_return_true_for_exactly_one_thread_when_the_same_word_is_deleted_concurrently() {
    let t = sut!(create());
    for i in 0..500 {
        t.insert(&format!("shared-{i}")).unwrap();
    }

    let wins = collect_concurrently(16, |_| {
        (0..500)
            .filter(|i| t.delete(&format!("shared-{i}")).unwrap())
            .count()
    });

    assert_eq!(wins.iter().sum::<usize>(), 500);
    assert_eq!(t.size(), 0);
    assert_eq!(t.starts_with(""), Ok(false));
}

#[test]
fn should_count_each_distinct_word_once_when_writers_insert_overlapping_ranges() {
    let t = sut!(create());

    run_concurrently(8, |id| {
        for i in id * 500..id * 500 + 2_000 {
            t.insert(&word(i)).unwrap();
        }
    });

    // ranges [0,2000) .. [3500,5500) overlap: union is [0, 5500)
    assert_eq!(t.size(), 5_500);
    assert_eq!(t.count_with_prefix(""), Ok(5_500));
    let all = t.words_with_prefix("", 10_000).unwrap();
    assert_eq!(all.len(), 5_500);
    assert_eq!(all.iter().collect::<HashSet<_>>().len(), 5_500);
}

#[test]
fn should_not_lose_either_side_when_inserts_and_deletes_of_disjoint_words_run_together() {
    let t = sut!(create());
    for i in 0..4_000 {
        t.insert(&format!("old{}", word(i))).unwrap();
    }

    run_concurrently(8, |id| {
        if id < 4 {
            for i in (id..4_000).step_by(4) {
                assert_eq!(t.delete(&format!("old{}", word(i))), Ok(true));
            }
        } else {
            for i in ((id - 4)..4_000).step_by(4) {
                assert_eq!(t.insert(&format!("new{}", word(i))), Ok(true));
            }
        }
    });

    assert_eq!(t.size(), 4_000);
    assert_eq!(t.starts_with("old"), Ok(false));
    assert_eq!(t.count_with_prefix("new"), Ok(4_000));
}

#[test]
fn should_end_with_the_right_words_when_every_thread_inserts_and_deletes_its_own_words() {
    let t = sut!(create());

    run_concurrently(8, |id| {
        for _ in 0..200 {
            for i in 0..20 {
                assert_eq!(t.insert(&format!("t{id}-{i}")), Ok(true));
            }
            for i in (0..20).step_by(2) {
                assert_eq!(t.delete(&format!("t{id}-{i}")), Ok(true));
            }
            for i in (1..20).step_by(2) {
                assert_eq!(t.delete(&format!("t{id}-{i}")), Ok(true));
            }
        }
        t.insert(&format!("t{id}-keep")).unwrap();
    });

    assert_eq!(t.size(), 8);
    let expected: Vec<String> = (0..8).map(|i| format!("t{i}-keep")).collect();
    assert_eq!(t.words_with_prefix("", 100), Ok(expected));
}

#[test]
fn should_never_show_a_half_inserted_word_to_readers() {
    let t = sut!(create());
    let writers = 4;
    let per_writer = 2_000;
    let done = AtomicBool::new(false);
    let writers_left = AtomicUsize::new(writers);

    run_concurrently(writers + 4, |id| {
        if id < writers {
            for i in 0..per_writer {
                t.insert(&format!("w{id}-{}", word(i))).unwrap();
            }
            if writers_left.fetch_sub(1, Ordering::SeqCst) == 1 {
                done.store(true, Ordering::SeqCst);
            }
            return;
        }
        // Words are only ever added here, so once a word is seen it must stay fully visible.
        while !done.load(Ordering::SeqCst) {
            for w in 0..writers {
                for i in (0..per_writer).step_by(37) {
                    let full = format!("w{w}-{}", word(i));
                    if t.contains(&full).unwrap() {
                        assert_eq!(t.starts_with(&full), Ok(true));
                        assert_eq!(t.starts_with(&full[..full.len() - 1]), Ok(true));
                        assert!(t.count_with_prefix(&full).unwrap() >= 1);
                        assert_eq!(t.words_with_prefix(&full, 1), Ok(vec![full.clone()]));
                    }
                }
            }
        }
    });

    assert_eq!(t.size(), writers * per_writer);
}

#[test]
fn should_return_consistent_sorted_snapshots_while_words_are_inserted_and_deleted() {
    let t = sut!(create());
    let mut universe: HashSet<String> = HashSet::new();
    let stable: Vec<String> = (0..200).map(|i| format!("s{}", word(i))).collect();
    for s in &stable {
        t.insert(s).unwrap();
        universe.insert(s.clone());
    }
    for i in 0..1_000 {
        universe.insert(format!("s-churn{}", word(i)));
    }
    let writers_left = AtomicUsize::new(2);
    let done = AtomicBool::new(false);

    run_concurrently(6, |id| {
        if id < 2 {
            for _ in 0..20 {
                for i in (id..1_000).step_by(2) {
                    t.insert(&format!("s-churn{}", word(i))).unwrap();
                }
                for i in (id..1_000).step_by(2) {
                    t.delete(&format!("s-churn{}", word(i))).unwrap();
                }
            }
            if writers_left.fetch_sub(1, Ordering::SeqCst) == 1 {
                done.store(true, Ordering::SeqCst);
            }
            return;
        }
        while !done.load(Ordering::SeqCst) {
            let snapshot = t.words_with_prefix("s", 100_000).unwrap();
            assert!(snapshot.windows(2).all(|w| w[0] < w[1]), "sorted, no duplicates");
            assert!(snapshot.iter().all(|w| universe.contains(w)));
            let set: HashSet<&String> = snapshot.iter().collect();
            assert!(stable.iter().all(|s| set.contains(s)));
            let c = t.count_with_prefix("s").unwrap();
            assert!((200..=1_200).contains(&c));
        }
    });

    assert_eq!(t.size(), 200);
    assert_eq!(t.words_with_prefix("", 1_000).unwrap().len(), 200);
}

#[test]
fn should_keep_size_equal_to_the_number_of_listed_words_after_mixed_concurrent_writes() {
    let t = sut!(create());

    run_concurrently(8, |id| {
        for i in 0..3_000 {
            let k = (i * 7 + id * 13) % 1_500;
            if (i + id) % 3 == 0 {
                t.delete(&word(k)).unwrap();
            } else {
                t.insert(&word(k)).unwrap();
            }
        }
    });

    let all = t.words_with_prefix("", 100_000).unwrap();
    assert!(all.windows(2).all(|w| w[0] < w[1]), "sorted, no duplicates");
    assert_eq!(t.size(), all.len());
    assert_eq!(t.count_with_prefix(""), Ok(all.len()));
    for w in &all {
        assert_eq!(t.contains(w), Ok(true));
    }
}

// ---------------------------------------------------------------- scale

#[test]
fn should_handle_hundred_thousand_words_and_ten_thousand_prefix_queries_in_reasonable_time() {
    let t = sut!(create());
    let start = Instant::now();

    for i in 0..100_000 {
        t.insert(&word(i)).unwrap();
    }
    assert_eq!(t.size(), 100_000);

    for i in 0..10_000 {
        let p = word(i * 7);
        assert_eq!(t.starts_with(&p), Ok(true));
        assert!(t.count_with_prefix(&p).unwrap() >= 1);
        assert!(!t.words_with_prefix(&p, 5).unwrap().is_empty());
    }
    for i in (0..100_000).step_by(2) {
        assert_eq!(t.delete(&word(i)), Ok(true));
    }
    assert_eq!(t.size(), 50_000);

    let elapsed = start.elapsed().as_millis();
    assert!(
        elapsed < 20_000,
        "100k inserts, 10k prefix queries and 50k deletes took {elapsed} ms"
    );
}
