#![allow(unused_imports, clippy::needless_range_loop)]
use p127_browser_history::{contract::*, create};
use playground_support::{run_concurrently, sut, FakeClock};
use std::sync::Arc;

fn ok(s: &str) -> Result<String, BrowserError> {
    Ok(s.to_string())
}

fn lines(v: &[&str]) -> Result<Vec<String>, BrowserError> {
    Ok(v.iter().map(|s| s.to_string()).collect())
}

macro_rules! browser {
    ($clock:expr, $cap:expr) => {
        sut!(create($cap, $clock.as_clock()))
    };
}

#[test]
fn should_truncate_forward_entries_even_when_the_new_url_equals_a_forward_entry() {
    let clock = FakeClock::new(5_000);
    let b = browser!(clock, 100);
    let t = b.open_tab();
    b.visit(t, "a").unwrap();
    b.visit(t, "b").unwrap();
    b.visit(t, "c").unwrap();
    b.back(t, 2).unwrap();

    b.visit(t, "b").unwrap(); // not the current url ("a"): a brand new entry, "b" and "c" ahead of it are gone

    assert_eq!(b.current_url(t), ok("b"));
    assert_eq!(b.can_go_forward(t), Ok(false));
    assert_eq!(b.back(t, 1), ok("a"));
    assert_eq!(b.back(t, 1), ok("about:blank"));
    assert_eq!(b.forward(t, 10), ok("b"));
}

#[test]
fn should_only_suppress_duplicates_against_the_current_entry_not_against_earlier_ones() {
    let clock = FakeClock::new(5_000);
    let b = browser!(clock, 100);
    let t = b.open_tab();
    b.visit(t, "a").unwrap();
    b.visit(t, "b").unwrap();
    b.visit(t, "a").unwrap(); // "a" is not the current page: a third entry

    assert_eq!(b.back(t, 1), ok("b"));
    assert_eq!(b.back(t, 1), ok("a"));
    assert_eq!(b.back(t, 1), ok("about:blank"));
    assert_eq!(b.can_go_back(t), Ok(false));
}

#[test]
fn should_keep_insertion_order_for_entries_with_the_same_timestamp() {
    let clock = FakeClock::new(5_000);
    let b = browser!(clock, 100);
    let t1 = b.open_tab();
    let t2 = b.open_tab();
    b.visit(t1, "x").unwrap();
    b.visit(t2, "y").unwrap();
    b.visit(t1, "z").unwrap();

    let expected = ["5000|z|1", "5000|y|2", "5000|x|1"];
    assert_eq!(b.history(10), lines(&expected));
    assert_eq!(b.history_between(5000, 5000), lines(&expected));
}

#[test]
fn should_make_counts_reflect_only_retained_entries() {
    let clock = FakeClock::new(5_000);
    let b = browser!(clock, 5);
    let t = b.open_tab();
    for u in ["a", "a", "a", "b", "c"] {
        b.visit(t, u).unwrap();
        clock.advance(1);
    }
    assert_eq!(b.most_visited(3), lines(&["a", "c", "b"]));

    b.visit(t, "b").unwrap(); // pushes out the first "a": a=2, b=2, c=1
    clock.advance(1);
    assert_eq!(b.most_visited(3), lines(&["b", "a", "c"]));

    b.visit(t, "d").unwrap(); // pushes out the second "a": a=1
    assert_eq!(b.most_visited(10), lines(&["b", "d", "c", "a"]));
    assert_eq!(b.delete_from_history("b"), Ok(2));
    assert_eq!(b.most_visited(10), lines(&["d", "c", "a"]));
}

#[test]
fn should_accept_new_entries_after_clearing_everything() {
    let clock = FakeClock::new(5_000);
    let b = browser!(clock, 3);
    let t = b.open_tab();
    for u in ["a", "b", "c", "d"] {
        b.visit(t, u).unwrap();
        clock.advance(1);
    }
    assert_eq!(b.clear_history(0), 3);
    assert_eq!(b.history(10), lines(&[]));
    assert_eq!(b.most_visited(5), lines(&[]));

    b.visit(t, "e").unwrap();
    b.visit(t, "f").unwrap();

    assert_eq!(b.history(10), lines(&["5004|f|1", "5004|e|1"]));
    assert_eq!(b.back(t, 100), ok("about:blank"));
}

#[test]
fn should_behave_with_capacity_one() {
    let clock = FakeClock::new(5_000);
    let b = browser!(clock, 1);
    let t = b.open_tab();
    b.visit(t, "a").unwrap();
    b.visit(t, "b").unwrap();

    assert_eq!(b.history(10), lines(&["5000|b|1"]));
    assert_eq!(b.most_visited(5), lines(&["b"]));
    assert_eq!(b.back(t, 1), ok("a"));
}

#[test]
fn should_handle_long_sessions_and_huge_step_counts() {
    let clock = FakeClock::new(5_000);
    let b = browser!(clock, 10);
    let t = b.open_tab();
    for i in 0..100_000 {
        b.visit(t, &format!("https://example.com/{i}")).unwrap();
    }

    assert_eq!(b.history(100).unwrap().len(), 10);
    assert_eq!(b.back(t, 99_999), ok("https://example.com/0"));
    assert_eq!(b.back(t, i32::MAX), ok("about:blank"));
    assert_eq!(b.forward(t, i32::MAX), ok("https://example.com/99999"));
    assert_eq!(b.forward(t, 1), ok("https://example.com/99999"));
}

#[test]
fn should_drop_the_oldest_entries_quickly_when_the_capacity_is_large() {
    let clock = FakeClock::new(5_000);
    let b = browser!(clock, 50_000);
    let t = b.open_tab();

    for i in 0..300_000 {
        b.visit(t, &format!("u{i}")).unwrap();
    }

    let recent = b.history(3).unwrap();
    assert!(recent[0].ends_with("|u299999|1"));
    assert_eq!(b.search("u250000"), lines(&["u250000"]));
    assert_eq!(b.search("u249999"), lines(&[]));
    assert_eq!(b.history(100_000).unwrap().len(), 50_000);
}

#[test]
fn should_survive_tabs_being_closed_while_others_browse() {
    let clock = FakeClock::new(5_000);
    let b = browser!(clock, 100_000);

    run_concurrently(8, |t| {
        for i in 0..300 {
            let tab = b.open_tab();
            b.visit(tab, &format!("https://t{t}.com/{i}")).unwrap();
            b.visit(tab, &format!("https://t{t}.com/{i}/next")).unwrap();
            b.back(tab, 1).unwrap();
            b.close_tab(tab).unwrap();
            assert_eq!(b.visit(tab, "x"), Err(BrowserError::UnknownTab));
        }
    });

    assert_eq!(b.history(100_000).unwrap().len(), 8 * 300 * 2);
    assert_eq!(b.open_tab(), 8 * 300 + 1);
}

#[test]
fn should_keep_every_reader_view_consistent_while_writers_and_clears_run() {
    let clock = FakeClock::new(5_000);
    let b = browser!(clock, 500);
    let tab = b.open_tab();

    run_concurrently(8, |t| {
        for i in 0..2_000 {
            if t < 4 {
                b.visit(tab, &format!("u{t}-{i}")).unwrap();
            } else if t == 4 && i % 100 == 0 {
                b.clear_history(0);
            } else {
                for line in b.history(20).unwrap() {
                    assert_eq!(line.split('|').count(), 3);
                }
                assert!(b.search("u1-").unwrap().len() <= 500);
                assert!(b.most_visited(5).unwrap().len() <= 5);
            }
        }
    });

    assert!(b.history(10_000).unwrap().len() <= 500);
    assert!(b.current_url(tab).unwrap().starts_with('u'));
}
