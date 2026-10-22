#![allow(unused_imports, clippy::needless_range_loop)]
use p109_document_line_search::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

fn big(n: usize) -> Vec<String> {
    (1..=n)
        .map(|i| {
            let mut s = format!("row{i} common shared bucket{}", i % 50);
            if i % 10_000 == 0 {
                s.push_str(" rare");
            }
            s
        })
        .collect()
}

#[test]
fn should_return_sorted_ids_when_a_rare_word_is_intersected_with_a_common_one() {
    let d = sut!(create(&big(100_000)));

    let expected: Vec<u32> = (1..=10).map(|k| k * 10_000).collect();
    assert_eq!(d.search("common rare"), expected);
    assert_eq!(d.search("rare common shared"), expected);
    assert!(d.search("rare nosuchword common").is_empty());
}

#[test]
fn should_answer_five_thousand_searches_on_a_hundred_thousand_lines() {
    let d = sut!(create(&big(100_000)));

    let start = Instant::now();
    let mut total = 0usize;
    for i in 0..5_000usize {
        let row = 1 + (i * 37) % 100_000;
        total += d
            .search(&format!("bucket{} row{row} common", row % 50))
            .len();
    }
    let elapsed_ms = start.elapsed().as_millis();

    assert!(total > 0);
    assert!(
        elapsed_ms < 15_000,
        "5,000 searches over 100,000 lines took {elapsed_ms} ms"
    );
}

#[test]
fn should_keep_ids_stable_while_deleting_and_searching_interleaved() {
    let d = sut!(create(&big(5_000)));

    let mut live = 5_000usize;
    for id in (1..=5_000u32).step_by(3) {
        assert!(d.delete_line(id));
        live -= 1;
        if id % 300 == 1 {
            let ids = d.search("common");
            assert_eq!(ids.len(), live);
            assert!(ids.windows(2).all(|w| w[0] < w[1]));
            assert!(!ids.contains(&id));
        }
    }
    assert_eq!(d.search("row2"), vec![2]);
    assert!(d.search("row1").is_empty());
    assert_eq!(d.search("row5000"), vec![5_000]);
    assert!(!d.delete_line(5_001));
}

#[test]
fn should_return_nothing_after_every_line_is_deleted() {
    let d = sut!(create(&["a b", "b c", "c a"]));

    assert!(d.delete_line(3));
    assert!(d.delete_line(1));
    assert!(d.delete_line(2));

    assert!(d.search("a").is_empty());
    assert!(d.search("b c").is_empty());
    for id in 1..=4 {
        assert!(!d.delete_line(id));
    }
}

#[test]
fn should_fold_case_of_non_ascii_letters_and_split_on_any_whitespace() {
    let d = sut!(create(&[
        "\u{c9}COLE normale",
        "caf\u{e9}\tcr\u{e8}me\nbr\u{fb}l\u{e9}e",
        "Stra\u{df}e 7",
        "tab\u{2003}separated"
    ]));

    assert_eq!(d.search("\u{e9}cole"), vec![1]);
    assert_eq!(d.search("\u{c9}COLE"), vec![1]);
    assert_eq!(d.search("CAF\u{c9} BR\u{db}L\u{c9}E"), vec![2]);
    assert_eq!(d.search("cr\u{e8}me"), vec![2]);
    assert_eq!(d.search("tab separated"), vec![4]);
    assert_eq!(d.search("7"), vec![3]);
}

#[test]
fn should_handle_a_very_long_line_and_a_query_with_many_words() {
    let mut long_line = String::new();
    for i in 0..50_000 {
        long_line.push_str(&format!("w{i}{}", if i % 2 == 0 { ", " } else { " - " }));
    }
    let mut query = String::new();
    for i in 0..200 {
        query.push_str(&format!(" w{}", i * 7));
    }
    let d = sut!(create(&["w1 w2", long_line.as_str(), "w7 w14"]));

    assert_eq!(d.search(&query), vec![2]);
    assert!(d.search(&format!("{query} w50000")).is_empty());
    assert_eq!(d.search("w7 w14"), vec![2, 3]);
}

#[test]
fn should_tell_exactly_one_thread_for_every_raced_deletion() {
    let lines = 200usize;
    let text: Vec<String> = (0..lines).map(|i| format!("shared line{i}")).collect();
    let d = sut!(create(&text));
    let wins: Vec<AtomicUsize> = (0..=lines).map(|_| AtomicUsize::new(0)).collect();

    run_concurrently(16, |_| {
        for id in 1..=lines {
            if d.delete_line(id as u32) {
                wins[id].fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    for id in 1..=lines {
        assert_eq!(wins[id].load(Ordering::SeqCst), 1, "winners for id {id}");
    }
    assert!(d.search("shared").is_empty());
}

#[test]
fn should_never_show_a_deleted_line_to_a_search_that_starts_after_the_deletion_returned() {
    let d = sut!(create(&big(20_000)));
    let deleted_up_to = AtomicU32::new(0);
    let problem: Mutex<Option<String>> = Mutex::new(None);

    run_concurrently(9, |t| {
        if t == 0 {
            for id in 1..=20_000u32 {
                d.delete_line(id);
                deleted_up_to.store(id, Ordering::SeqCst);
            }
            return;
        }
        for i in 0..100 {
            let floor = deleted_up_to.load(Ordering::SeqCst);
            let ids = d.search(&format!("shared bucket{}", i % 50));
            let mut prev = 0;
            for id in ids {
                if id <= floor {
                    *problem.lock().unwrap() =
                        Some(format!("deleted id {id} visible, floor {floor}"));
                }
                if id <= prev {
                    *problem.lock().unwrap() = Some("not strictly increasing".into());
                }
                prev = id;
            }
        }
    });

    assert_eq!(*problem.lock().unwrap(), None);
    assert!(d.search("shared").is_empty());
}
