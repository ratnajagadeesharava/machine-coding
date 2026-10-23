#![allow(unused_imports, clippy::needless_range_loop)]
use p109_document_line_search::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

fn large_lines(n: usize) -> Vec<String> {
    (1..=n)
        .map(|i| format!("line {i} group{} common Item{}", i % 100, i % 7))
        .collect()
}

fn is_strictly_increasing(v: &[u32]) -> bool {
    v.windows(2).all(|w| w[0] < w[1])
}

#[test]
fn should_find_lines_containing_a_single_word() {
    let d = sut!(create(&["The cat sat", "a dog barked", "cat and dog"]));

    assert_eq!(d.search("cat"), vec![1, 3]);
    assert_eq!(d.search("dog"), vec![2, 3]);
}

#[test]
fn should_assign_one_based_ids_in_document_order() {
    let d = sut!(create(&["x", "y", "x", "y", "x"]));

    assert_eq!(d.search("x"), vec![1, 3, 5]);
    assert_eq!(d.search("y"), vec![2, 4]);
}

#[test]
fn should_require_every_word_of_the_query() {
    let d = sut!(create(&[
        "red green",
        "green blue",
        "red green blue",
        "red"
    ]));

    assert_eq!(d.search("red green"), vec![1, 3]);
    assert_eq!(d.search("red green blue"), vec![3]);
    assert!(d.search("red blue yellow").is_empty());
}

#[test]
fn should_ignore_word_order_and_position() {
    let d = sut!(create(&[
        "alpha beta gamma",
        "gamma x y z beta alpha",
        "beta alpha"
    ]));

    assert_eq!(d.search("gamma alpha"), vec![1, 2]);
    assert_eq!(d.search("alpha gamma"), vec![1, 2]);
}

#[test]
fn should_match_case_insensitively() {
    let d = sut!(create(&["Hello World", "hello there", "HELLO"]));

    assert_eq!(d.search("hello"), vec![1, 2, 3]);
    assert_eq!(d.search("HeLLo wORLD"), vec![1]);
}

#[test]
fn should_match_whole_words_only() {
    let d = sut!(create(&[
        "A Category of cats",
        "concat bobcat",
        "the cat",
        "cat"
    ]));

    assert_eq!(d.search("cat"), vec![3, 4]);
    assert_eq!(d.search("cats"), vec![1]);
    assert!(d.search("cate").is_empty());
}

#[test]
fn should_treat_punctuation_and_underscores_as_separators() {
    let d = sut!(create(&[
        "cat, dog.",
        "(cat)",
        "the-cat",
        "cat_food",
        "tabs\tand\nnewline cat",
        "concat"
    ]));

    assert_eq!(d.search("cat"), vec![1, 2, 3, 4, 5]);
    assert_eq!(d.search("food cat"), vec![4]);
    assert_eq!(d.search("dog"), vec![1]);
}

#[test]
fn should_treat_letters_and_digits_as_one_word() {
    let d = sut!(create(&["v2 release notes", "release 2 notes", "version2"]));

    assert_eq!(d.search("v2"), vec![1]);
    assert_eq!(d.search("2"), vec![2]);
    assert!(d.search("v").is_empty());
}

#[test]
fn should_split_the_query_with_the_same_rule_as_the_lines() {
    let d = sut!(create(&["snake_case name", "snake only", "case only"]));

    assert_eq!(d.search("snake_case"), vec![1]);
    assert_eq!(d.search("  snake,   case!! "), vec![1]);
}

#[test]
fn should_ignore_duplicate_query_words_and_repeated_words_in_a_line() {
    let d = sut!(create(&["cat cat cat", "cat dog", "dog"]));

    assert_eq!(d.search("cat cat"), vec![1, 2]);
    assert_eq!(d.search("cat dog cat dog"), vec![2]);
    assert_eq!(d.search("CAT cat Cat"), vec![1, 2]);
}

#[test]
fn should_return_an_empty_list_for_a_query_without_words() {
    let d = sut!(create(&["some text", "", "more text"]));

    assert!(d.search("").is_empty());
    assert!(d.search("   ").is_empty());
    assert!(d.search("?! ,.").is_empty());
    assert!(d.search("\t\n").is_empty());
}

#[test]
fn should_return_an_empty_list_when_nothing_matches() {
    let d = sut!(create(&["one two", "three"]));

    assert!(d.search("four").is_empty());
    assert!(d.search("one four").is_empty());
}

#[test]
fn should_handle_an_empty_document_and_blank_lines() {
    let none: [&str; 0] = [];
    let empty = sut!(create(&none));
    assert!(empty.search("a").is_empty());
    assert!(!empty.delete_line(1));

    let d = sut!(create(&["", "   ", "word", ""]));
    assert_eq!(d.search("word"), vec![3]);
    assert!(d.delete_line(1));
    assert!(d.delete_line(4));
    assert_eq!(d.search("word"), vec![3]);
}

#[test]
fn should_remove_a_deleted_line_from_every_search() {
    let d = sut!(create(&["cat dog", "cat", "dog cat bird"]));

    assert!(d.delete_line(1));

    assert_eq!(d.search("cat"), vec![2, 3]);
    assert_eq!(d.search("cat dog"), vec![3]);
    assert_eq!(d.search("dog"), vec![3]);
}

#[test]
fn should_keep_the_ids_of_the_other_lines_after_a_deletion() {
    let d = sut!(create(&["keep", "drop", "keep", "keep"]));

    assert!(d.delete_line(1));
    assert_eq!(d.search("keep"), vec![3, 4]);
    assert!(d.delete_line(4));
    assert_eq!(d.search("keep"), vec![3]);
    assert_eq!(d.search("drop"), vec![2]);
    assert!(d.delete_line(2));
    assert!(d.search("drop").is_empty());
}

#[test]
fn should_return_false_for_unknown_and_already_deleted_ids() {
    let d = sut!(create(&["a", "b", "c"]));

    assert!(!d.delete_line(0));
    assert!(!d.delete_line(4));
    assert!(!d.delete_line(u32::MAX));
    assert!(d.delete_line(2));
    assert!(!d.delete_line(2));
    assert_eq!(d.search("a"), vec![1]);
    assert_eq!(d.search("c"), vec![3]);
}

#[test]
fn should_never_reuse_the_id_of_a_deleted_last_line() {
    let d = sut!(create(&["a", "b"]));

    assert!(d.delete_line(2));
    assert!(!d.delete_line(2));
    assert!(!d.delete_line(3));
    assert!(d.search("b").is_empty());
}

#[test]
fn should_return_ids_in_increasing_order_without_duplicates() {
    let d = sut!(create(&large_lines(2_000)));

    let ids = d.search("common item3 group3");
    assert!(!ids.is_empty());
    assert!(is_strictly_increasing(&ids));
    assert!(ids.iter().all(|id| id % 7 == 3 && id % 100 == 3));
}

#[test]
fn should_return_a_copy_that_the_caller_may_modify() {
    let d = sut!(create(&["x", "x", "x"]));

    let mut first = d.search("x");
    first.clear();
    first.push(99);

    assert_eq!(d.search("x"), vec![1, 2, 3]);
}

#[test]
fn should_not_be_affected_by_later_changes_to_the_input() {
    let mut source = vec!["first line".to_string(), "second line".to_string()];
    let d = sut!(create(&source));

    source[0] = "changed".to_string();
    source.push("third line".to_string());

    assert_eq!(d.search("first"), vec![1]);
    assert!(d.search("changed").is_empty());
    assert_eq!(d.search("line"), vec![1, 2]);
}

#[test]
fn should_answer_queries_on_a_large_document() {
    let d = sut!(create(&large_lines(20_000)));

    assert_eq!(d.search("common").len(), 20_000);
    assert_eq!(d.search("group7").len(), 200);
    let both = d.search("group7 item3");
    assert!(!both.is_empty());
    assert!(both.iter().all(|id| id % 100 == 7 && id % 7 == 3));
    assert_eq!(d.search("line 20000"), vec![20_000]);
    assert!(d.search("group7 group8").is_empty());
    assert!(d.search("group7 nosuchword").is_empty());
}

#[test]
fn should_answer_thousands_of_searches_on_a_large_document_quickly() {
    let d = sut!(create(&large_lines(20_000)));

    let start = Instant::now();
    let mut total = 0usize;
    for i in 0..2_000 {
        total += d.search(&format!("group{} item{}", i % 100, i % 7)).len();
    }
    let elapsed_ms = start.elapsed().as_millis();

    assert!(total > 0);
    assert!(
        elapsed_ms < 10_000,
        "2,000 searches over 20,000 lines took {elapsed_ms} ms"
    );
}

#[test]
fn should_not_count_deleted_lines_in_a_large_document() {
    let d = sut!(create(&large_lines(20_000)));

    for id in (2..=20_000).step_by(2) {
        assert!(d.delete_line(id));
    }

    let ids = d.search("common");
    assert_eq!(ids.len(), 10_000);
    assert!(ids.iter().all(|id| id % 2 == 1));
    assert_eq!(d.search("line 3"), vec![3]);
    assert!(d.search("line 4").is_empty());
}

#[test]
fn should_let_eight_threads_delete_different_lines_without_losing_any() {
    let d = sut!(create(&large_lines(8_000)));

    let counts = collect_concurrently(8, |t| {
        let mut deleted = 0;
        let mut id = t as u32 + 1;
        while id <= 8_000 {
            if d.delete_line(id) {
                deleted += 1;
            }
            id += 8;
        }
        deleted
    });

    assert!(counts.iter().all(|&c| c == 1_000));
    assert!(d.search("common").is_empty());
}

#[test]
fn should_tell_exactly_one_of_many_racing_threads_that_it_deleted_the_line() {
    let d = sut!(create(&["target", "other"]));
    let winners = AtomicUsize::new(0);

    run_concurrently(16, |_| {
        if d.delete_line(1) {
            winners.fetch_add(1, Ordering::SeqCst);
        }
    });

    assert_eq!(winners.load(Ordering::SeqCst), 1);
    assert!(d.search("target").is_empty());
    assert_eq!(d.search("other"), vec![2]);
}

#[test]
fn should_search_while_lines_are_being_deleted() {
    let n = 4_000u32;
    let d = sut!(create(&large_lines(n as usize)));
    let deleted_up_to = AtomicU32::new(0);
    let problem: Mutex<Option<String>> = Mutex::new(None);

    run_concurrently(5, |t| {
        if t == 0 {
            for id in 1..=n {
                d.delete_line(id);
                deleted_up_to.store(id, Ordering::SeqCst);
            }
            return;
        }
        for _ in 0..300 {
            let floor = deleted_up_to.load(Ordering::SeqCst);
            let ids = d.search("common");
            if let Some(&bad) = ids.iter().find(|&&id| id <= floor) {
                *problem.lock().unwrap() =
                    Some(format!("deleted id {bad} returned, floor {floor}"));
            }
            if !is_strictly_increasing(&ids) {
                *problem.lock().unwrap() = Some("result not increasing".to_string());
            }
        }
    });

    assert_eq!(*problem.lock().unwrap(), None);
    assert!(d.search("common").is_empty());
}
