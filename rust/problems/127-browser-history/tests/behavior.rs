#![allow(unused_imports, clippy::needless_range_loop)]
use p127_browser_history::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut, FakeClock};
use std::collections::HashSet;
use std::sync::Arc;

const BLANK: &str = "about:blank";

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

/// Visits the urls one after another, advancing the clock by 10 ms after each visit.
fn visit_all(b: &Arc<dyn BrowserHistory>, clock: &FakeClock, tab: u32, urls: &[&str]) {
    for u in urls {
        b.visit(tab, u).unwrap();
        clock.advance(10);
    }
}

#[test]
fn should_number_tabs_from_one_and_open_them_on_a_blank_page() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 1_000);

    assert_eq!(b.open_tab(), 1);
    assert_eq!(b.open_tab(), 2);
    assert_eq!(b.current_url(1), ok(BLANK));
    assert_eq!(b.can_go_back(1), Ok(false));
    assert_eq!(b.can_go_forward(1), Ok(false));
    assert_eq!(b.history(10), lines(&[]));
}

#[test]
fn should_not_reuse_ids_of_closed_tabs() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 1_000);
    let t1 = b.open_tab();
    b.close_tab(t1).unwrap();

    assert_eq!(b.open_tab(), 2);
    assert_eq!(b.current_url(t1), Err(BrowserError::UnknownTab));
    assert_eq!(b.close_tab(t1), Err(BrowserError::UnknownTab));
}

#[test]
fn should_navigate_and_show_the_latest_page() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 1_000);
    let t = b.open_tab();
    visit_all(&b, &clock, t, &["https://a.com", "https://b.com"]);

    assert_eq!(b.current_url(t), ok("https://b.com"));
    assert_eq!(b.can_go_back(t), Ok(true));
    assert_eq!(b.can_go_forward(t), Ok(false));
}

#[test]
fn should_go_back_and_forward_by_the_requested_steps() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 1_000);
    let t = b.open_tab();
    visit_all(&b, &clock, t, &["a", "b", "c", "d"]);

    assert_eq!(b.back(t, 1), ok("c"));
    assert_eq!(b.back(t, 2), ok("a"));
    assert_eq!(b.forward(t, 1), ok("b"));
    assert_eq!(b.forward(t, 2), ok("d"));
    assert_eq!(b.can_go_forward(t), Ok(false));
    assert_eq!(b.back(t, 0), ok("d"));
    assert_eq!(b.forward(t, 0), ok("d"));
}

#[test]
fn should_stop_at_the_ends_instead_of_failing() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 1_000);
    let t = b.open_tab();
    visit_all(&b, &clock, t, &["a", "b"]);

    assert_eq!(b.back(t, 99), ok(BLANK));
    assert_eq!(b.can_go_back(t), Ok(false));
    assert_eq!(b.back(t, 1), ok(BLANK));
    assert_eq!(b.forward(t, i32::MAX), ok("b"));
    assert_eq!(b.forward(t, 1), ok("b"));
    assert_eq!(b.back(t, i32::MAX), ok(BLANK));
}

#[test]
fn should_discard_forward_entries_when_visiting_from_the_middle() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 1_000);
    let t = b.open_tab();
    visit_all(&b, &clock, t, &["a", "b", "c"]);
    b.back(t, 2).unwrap();

    b.visit(t, "x").unwrap();

    assert_eq!(b.current_url(t), ok("x"));
    assert_eq!(b.can_go_forward(t), Ok(false));
    assert_eq!(b.forward(t, 5), ok("x"));
    assert_eq!(b.back(t, 1), ok("a"));
    assert_eq!(b.back(t, 1), ok(BLANK));
}

#[test]
fn should_not_add_a_session_entry_for_the_current_url_but_still_record_the_visit() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 1_000);
    let t = b.open_tab();
    visit_all(&b, &clock, t, &["a", "a", "a"]);

    assert_eq!(b.back(t, 1), ok(BLANK));
    b.forward(t, 1).unwrap();
    assert_eq!(b.current_url(t), ok("a"));
    assert_eq!(b.history(10).unwrap().len(), 3);
}

#[test]
fn should_keep_forward_entries_when_revisiting_the_current_url() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 1_000);
    let t = b.open_tab();
    visit_all(&b, &clock, t, &["a", "b"]);
    b.back(t, 1).unwrap();

    b.visit(t, "a").unwrap();

    assert_eq!(b.can_go_forward(t), Ok(true));
    assert_eq!(b.forward(t, 1), ok("b"));
}

#[test]
fn should_treat_the_blank_page_as_an_ordinary_url_when_visited() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 1_000);
    let t = b.open_tab();

    b.visit(t, BLANK).unwrap();

    assert_eq!(b.can_go_back(t), Ok(false));
    assert_eq!(b.history(5), lines(&["1000|about:blank|1"]));
}

#[test]
fn should_keep_tabs_independent() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 1_000);
    let t1 = b.open_tab();
    let t2 = b.open_tab();
    visit_all(&b, &clock, t1, &["a", "b"]);
    visit_all(&b, &clock, t2, &["x"]);

    b.back(t1, 1).unwrap();

    assert_eq!(b.current_url(t1), ok("a"));
    assert_eq!(b.current_url(t2), ok("x"));
    assert_eq!(b.can_go_back(t2), Ok(true));
    assert_eq!(b.can_go_forward(t2), Ok(false));
}

#[test]
fn should_record_global_visits_most_recent_first_with_time_url_and_tab() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 1_000);
    let t1 = b.open_tab();
    let t2 = b.open_tab();
    b.visit(t1, "https://a.com").unwrap();
    clock.advance(5);
    b.visit(t2, "https://b.com").unwrap();
    clock.advance(5);
    b.visit(t1, "https://c.com").unwrap();

    assert_eq!(
        b.history(10),
        lines(&[
            "1010|https://c.com|1",
            "1005|https://b.com|2",
            "1000|https://a.com|1"
        ])
    );
    assert_eq!(
        b.history(2),
        lines(&["1010|https://c.com|1", "1005|https://b.com|2"])
    );
    assert_eq!(b.history(0), lines(&[]));
}

#[test]
fn should_not_record_navigation_in_the_global_history() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 1_000);
    let t = b.open_tab();
    visit_all(&b, &clock, t, &["a", "b", "c"]);

    b.back(t, 2).unwrap();
    b.forward(t, 1).unwrap();

    assert_eq!(b.history(10).unwrap().len(), 3);
}

#[test]
fn should_return_entries_in_an_inclusive_time_range() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 1_000);
    let t = b.open_tab();
    visit_all(&b, &clock, t, &["a", "b", "c", "d"]); // 1000, 1010, 1020, 1030

    assert_eq!(
        b.history_between(1010, 1020),
        lines(&["1020|c|1", "1010|b|1"])
    );
    assert_eq!(b.history_between(1011, 1019), lines(&[]));
    assert_eq!(b.history_between(1030, 1030), lines(&["1030|d|1"]));
    assert_eq!(b.history_between(0, 5_000).unwrap().len(), 4);
}

#[test]
fn should_search_case_insensitively_listing_each_url_once_by_latest_visit() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 1_000);
    let t = b.open_tab();
    visit_all(
        &b,
        &clock,
        t,
        &[
            "https://Docs.example.com/a",
            "https://news.com",
            "https://example.org",
            "https://docs.example.com/b",
        ],
    );
    visit_all(&b, &clock, t, &["https://Docs.example.com/a"]);

    assert_eq!(
        b.search("EXAMPLE"),
        lines(&[
            "https://Docs.example.com/a",
            "https://docs.example.com/b",
            "https://example.org"
        ])
    );
    assert_eq!(
        b.search("docs.example.com/a"),
        lines(&["https://Docs.example.com/a"])
    );
    assert_eq!(b.search("zzz"), lines(&[]));
}

#[test]
fn should_rank_most_visited_by_count_then_latest_visit_then_url() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 1_000);
    let t = b.open_tab();
    visit_all(&b, &clock, t, &["p", "q", "r"]); // 1000, 1010, 1020
    visit_all(&b, &clock, t, &["q", "p"]); // 1030, 1040
    visit_all(&b, &clock, t, &["z"]); // 1050
    visit_all(&b, &clock, t, &["y"]); // 1060

    assert_eq!(b.most_visited(10), lines(&["p", "q", "y", "z", "r"]));
    assert_eq!(b.most_visited(2), lines(&["p", "q"]));
    assert_eq!(b.most_visited(0), lines(&[]));
}

#[test]
fn should_break_most_visited_ties_by_url_when_latest_visit_times_are_equal() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 1_000);
    let t1 = b.open_tab();
    let t2 = b.open_tab();
    b.visit(t1, "b.com").unwrap();
    b.visit(t2, "a.com").unwrap();
    b.visit(t1, "c.com").unwrap();

    assert_eq!(b.most_visited(3), lines(&["a.com", "b.com", "c.com"]));
}

#[test]
fn should_delete_a_url_from_the_global_history_but_not_from_open_tabs() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 1_000);
    let t1 = b.open_tab();
    let t2 = b.open_tab();
    visit_all(&b, &clock, t1, &["a", "b"]);
    visit_all(&b, &clock, t2, &["a"]);

    assert_eq!(b.delete_from_history("a"), Ok(2));
    assert_eq!(b.delete_from_history("a"), Ok(0));
    assert_eq!(b.delete_from_history("never"), Ok(0));

    assert_eq!(b.history(10), lines(&["1010|b|1"]));
    assert_eq!(b.search("a"), lines(&[]));
    assert_eq!(b.back(t1, 1), ok("a"));
    assert_eq!(b.current_url(t2), ok("a"));
}

#[test]
fn should_clear_entries_from_a_given_time_onwards() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 1_000);
    let t = b.open_tab();
    visit_all(&b, &clock, t, &["a", "b", "c", "d"]); // 1000, 1010, 1020, 1030

    assert_eq!(b.clear_history(1020), 2);
    assert_eq!(b.history(10), lines(&["1010|b|1", "1000|a|1"]));
    assert_eq!(b.clear_history(5_000), 0);
    assert_eq!(b.current_url(t), ok("d"));
    assert_eq!(b.clear_history(0), 2);
    assert_eq!(b.history(10), lines(&[]));
}

#[test]
fn should_drop_the_oldest_entries_beyond_the_capacity() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 3);
    let t = b.open_tab();
    visit_all(&b, &clock, t, &["a", "b", "c", "d", "e"]);

    assert_eq!(b.history(10), lines(&["1040|e|1", "1030|d|1", "1020|c|1"]));
    assert_eq!(b.search("a"), lines(&[]));
    assert_eq!(b.most_visited(10), lines(&["e", "d", "c"]));
    assert_eq!(b.back(t, 4), ok("a")); // tabs are not limited by the capacity
}

#[test]
fn should_free_capacity_when_entries_are_deleted() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 4);
    let t = b.open_tab();
    visit_all(&b, &clock, t, &["a", "a", "b", "a"]); // 4 entries
    assert_eq!(b.most_visited(1), lines(&["a"]));

    b.delete_from_history("a").unwrap();
    visit_all(&b, &clock, t, &["c", "d", "e", "f"]); // "b" is pushed out

    assert_eq!(b.history(10).unwrap().len(), 4);
    assert_eq!(b.search("b"), lines(&[]));
}

#[test]
fn should_keep_history_entries_of_closed_tabs() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 1_000);
    let t = b.open_tab();
    visit_all(&b, &clock, t, &["a"]);

    b.close_tab(t).unwrap();

    assert_eq!(b.history(10), lines(&["1000|a|1"]));
}

#[test]
fn should_reject_invalid_input() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 1_000);
    let t = b.open_tab();
    b.visit(t, "a").unwrap();
    let bad = BrowserError::InvalidArgument;
    let unknown = BrowserError::UnknownTab;

    assert_eq!(b.visit(t, "").err(), Some(bad));
    assert_eq!(b.visit(t, "  \t").err(), Some(bad));
    assert_eq!(b.visit(99, "x").err(), Some(unknown));
    assert_eq!(b.visit(0, "x").err(), Some(unknown));
    assert_eq!(b.back(t, -1).err(), Some(bad));
    assert_eq!(b.forward(t, -1).err(), Some(bad));
    assert_eq!(b.back(99, 1).err(), Some(unknown));
    assert_eq!(b.forward(99, 1).err(), Some(unknown));
    assert_eq!(b.can_go_back(99).err(), Some(unknown));
    assert_eq!(b.can_go_forward(99).err(), Some(unknown));
    assert_eq!(b.close_tab(99).err(), Some(unknown));
    assert_eq!(b.history(-1).err(), Some(bad));
    assert_eq!(b.history_between(10, 9).err(), Some(bad));
    assert_eq!(b.search(" ").err(), Some(bad));
    assert_eq!(b.most_visited(-1).err(), Some(bad));
    assert_eq!(b.delete_from_history(" ").err(), Some(bad));

    assert_eq!(b.history(10).unwrap().len(), 1);
    assert_eq!(b.current_url(t), ok("a"));
}

#[test]
fn should_reject_a_zero_capacity() {
    let clock = FakeClock::new(1_000);
    let _wired = browser!(clock, 10); // proves the crate is wired: from here on None means "rejected"

    assert!(create(0, clock.as_clock()).is_none());
}

#[test]
fn should_hand_out_unique_tab_ids_to_concurrent_callers() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 1_000);

    let ids: Vec<u32> =
        collect_concurrently(16, |_| (0..100).map(|_| b.open_tab()).collect::<Vec<_>>())
            .into_iter()
            .flatten()
            .collect();

    assert_eq!(ids.iter().collect::<HashSet<_>>().len(), 1_600);
    assert!(ids.iter().all(|id| (1..=1_600).contains(id)));
}

#[test]
fn should_lose_no_visits_when_many_threads_browse_at_once() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 100_000);
    let tabs: Vec<u32> = (0..8).map(|_| b.open_tab()).collect();

    run_concurrently(8, |t| {
        for i in 0..500 {
            b.visit(tabs[t], &format!("https://site{t}.com/{i}"))
                .unwrap();
            if i % 5 == 4 {
                b.back(tabs[t], 2).unwrap();
            }
        }
    });

    let all = b.history(100_000).unwrap();
    assert_eq!(all.len(), 4_000);
    assert_eq!(all.iter().collect::<HashSet<_>>().len(), 4_000);
    for t in 0..8 {
        assert_eq!(b.search(&format!("site{t}.com/")).unwrap().len(), 500);
    }
}

#[test]
fn should_keep_the_capacity_under_concurrent_visits_and_deletes() {
    let clock = FakeClock::new(1_000);
    let b = browser!(clock, 50);
    let t0 = b.open_tab();
    let t1 = b.open_tab();

    run_concurrently(8, |t| {
        for i in 0..500 {
            b.visit(if t % 2 == 0 { t0 } else { t1 }, &format!("u{}", i % 20))
                .unwrap();
            if t == 7 && i % 50 == 0 {
                b.delete_from_history("u3").unwrap();
            }
            b.history(10).unwrap();
        }
    });

    let n = b.history(1_000).unwrap().len();
    assert!(n <= 50 && n > 40, "history size {n}");
}
