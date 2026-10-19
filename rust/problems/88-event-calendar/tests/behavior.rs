#![allow(unused_imports, clippy::needless_range_loop)]
use p88_event_calendar::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::sync::atomic::{AtomicUsize, Ordering};

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn should_schedule_and_list_an_agenda_sorted_by_start() {
    let c = sut!(create());
    c.schedule("ann", "standup", 600, 630, &["bob"]).unwrap();
    c.schedule("ann", "lunch", 720, 780, &[]).unwrap();
    c.schedule("bob", "1:1", 540, 570, &["ann"]).unwrap();

    assert_eq!(
        c.agenda("ann", 0, 2_000),
        strs(&["540-570:1:1", "600-630:standup", "720-780:lunch"])
    );
    assert_eq!(
        c.agenda("bob", 0, 2_000),
        strs(&["540-570:1:1", "600-630:standup"])
    );
    assert!(c.agenda("cy", 0, 2_000).is_empty());
}

#[test]
fn should_include_events_that_overlap_the_agenda_window() {
    let c = sut!(create());
    c.schedule("ann", "a", 100, 200, &[]).unwrap();
    c.schedule("ann", "b", 200, 300, &[]).unwrap();

    assert_eq!(c.agenda("ann", 150, 200), strs(&["100-200:a"]));
    assert_eq!(c.agenda("ann", 200, 250), strs(&["200-300:b"]));
    assert!(c.agenda("ann", 300, 400).is_empty());
}

#[test]
fn should_allow_events_that_only_touch() {
    let c = sut!(create());
    c.schedule("ann", "a", 100, 200, &["bob"]).unwrap();

    c.schedule("bob", "b", 200, 300, &["ann"]).unwrap();
    c.schedule("ann", "c", 0, 100, &[]).unwrap();

    assert_eq!(c.agenda("ann", 0, 1_000).len(), 3);
}

#[test]
fn should_refuse_an_event_that_conflicts_for_any_participant_and_create_nothing() {
    let c = sut!(create());
    c.schedule("bob", "busy", 100, 200, &[]).unwrap();

    assert_eq!(
        c.schedule("ann", "meeting", 150, 250, &["bob", "cy"]),
        Err(CalError::Conflict)
    );

    assert!(c.agenda("ann", 0, 1_000).is_empty());
    assert!(c.agenda("cy", 0, 1_000).is_empty());
}

#[test]
fn should_reject_invalid_events() {
    let c = sut!(create());

    assert_eq!(
        c.schedule("ann", "x", 100, 100, &[]),
        Err(CalError::InvalidInput)
    );
    assert_eq!(
        c.schedule("ann", "x", 200, 100, &[]),
        Err(CalError::InvalidInput)
    );
    assert_eq!(
        c.schedule("ann", " ", 100, 200, &[]),
        Err(CalError::InvalidInput)
    );
    assert_eq!(
        c.schedule("ann", "x", 100, 200, &["bob", "bob"]),
        Err(CalError::InvalidInput)
    );
    assert_eq!(
        c.schedule("ann", "x", 100, 200, &["ann"]),
        Err(CalError::InvalidInput)
    );
}

#[test]
fn should_cancel_only_as_the_organizer_and_free_everybodys_slot() {
    let c = sut!(create());
    let id = c.schedule("ann", "sync", 100, 200, &["bob"]).unwrap();

    assert_eq!(c.cancel("bob", &id), Err(CalError::PermissionDenied));
    assert_eq!(c.cancel("ann", "no-such-id"), Err(CalError::UnknownEvent));
    c.cancel("ann", &id).unwrap();

    assert!(c.agenda("bob", 0, 1_000).is_empty());
    c.schedule("bob", "other", 100, 200, &["ann"]).unwrap();
}

#[test]
fn should_let_an_attendee_decline_without_affecting_others() {
    let c = sut!(create());
    let id = c.schedule("ann", "sync", 100, 200, &["bob", "cy"]).unwrap();

    c.decline("bob", &id).unwrap();

    assert!(c.agenda("bob", 0, 1_000).is_empty());
    assert_eq!(c.agenda("cy", 0, 1_000), strs(&["100-200:sync"]));
    assert_eq!(c.decline("ann", &id), Err(CalError::PermissionDenied));
    c.schedule("bob", "free now", 100, 200, &[]).unwrap();
}

#[test]
fn should_schedule_recurring_events_all_or_nothing() {
    let c = sut!(create());
    c.schedule(
        "bob",
        "blocker",
        10_000 + 2 * 1_440,
        10_060 + 2 * 1_440,
        &[],
    )
    .unwrap(); // hits the 3rd occurrence

    assert_eq!(
        c.schedule_recurring("ann", "daily", 10_000, 10_030, &["bob"], 1_440, 5),
        Err(CalError::Conflict)
    );
    assert!(c.agenda("ann", 0, 100_000).is_empty());

    let ids = c
        .schedule_recurring("ann", "daily", 10_000, 10_030, &["cy"], 1_440, 5)
        .unwrap();
    let mut unique = ids.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(ids.len(), 5);
    assert_eq!(unique.len(), 5);
    assert_eq!(c.agenda("cy", 0, 100_000).len(), 5);
    assert_eq!(
        c.agenda("ann", 0, 100_000)[1],
        format!("{}-{}:daily", 10_000 + 1_440, 10_030 + 1_440)
    );
}

#[test]
fn should_find_common_free_slots() {
    let c = sut!(create());
    c.schedule("ann", "a", 60, 120, &[]).unwrap();
    c.schedule("bob", "b", 100, 180, &[]).unwrap();
    c.schedule("bob", "c", 240, 260, &[]).unwrap();

    assert_eq!(
        c.common_free_slots(&["ann", "bob"], 0, 300, 30),
        strs(&["0-60", "180-240", "260-300"])
    );
    assert_eq!(
        c.common_free_slots(&["ann", "bob"], 0, 300, 60),
        strs(&["0-60", "180-240"])
    );
    assert_eq!(
        c.common_free_slots(&["ann"], 0, 300, 100),
        strs(&["120-300"])
    );
    assert!(c.common_free_slots(&["ann", "bob"], 0, 300, 200).is_empty());
}

#[test]
fn should_let_exactly_one_of_two_crossing_meetings_win() {
    for round in 0..50 {
        let c = sut!(create());

        let results = collect_concurrently(2, |t| {
            let (org, att) = if t == 0 {
                ("ann", "bob")
            } else {
                ("bob", "ann")
            };
            c.schedule(org, &format!("m{t}"), 100, 200, &[att]).is_ok()
        });

        assert_eq!(results.iter().filter(|r| **r).count(), 1, "round {round}");
    }
}

#[test]
fn should_never_double_book_anyone_under_concurrent_scheduling() {
    let c = sut!(create());
    let people = ["p0", "p1", "p2", "p3", "p4", "p5"];
    let accepted = AtomicUsize::new(0);

    run_concurrently(12, |t| {
        let mut s = (t as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        for _ in 0..200 {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            let a = (s % 6) as usize;
            let b = (a + 1 + (s >> 8) as usize % 5) % 6;
            let start = ((s >> 16) % 20) * 30;
            if c.schedule(people[a], "m", start, start + 30, &[people[b]])
                .is_ok()
            {
                accepted.fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    let mut entries = 0;
    for p in people {
        let agenda = c.agenda(p, 0, 10_000);
        entries += agenda.len();
        for w in agenda.windows(2) {
            let prev_end: u64 = w[0]
                .split(':')
                .next()
                .unwrap()
                .split('-')
                .nth(1)
                .unwrap()
                .parse()
                .unwrap();
            let start: u64 = w[1]
                .split(':')
                .next()
                .unwrap()
                .split('-')
                .next()
                .unwrap()
                .parse()
                .unwrap();
            assert!(start >= prev_end, "{p} double booked");
        }
    }
    assert_eq!(
        entries,
        accepted.load(Ordering::SeqCst) * 2,
        "each accepted meeting has exactly two participants"
    );
}

#[test]
fn should_stay_correct_over_100_runs() {
    for run in 0..100 {
        let c = sut!(create());
        let wins = AtomicUsize::new(0);

        run_concurrently(6, |t| {
            let next = format!("p{}", (t + 1) % 6);
            if c.schedule(&format!("p{t}"), "m", 0, 60, &[&next]).is_ok() {
                wins.fetch_add(1, Ordering::SeqCst);
            }
        });

        let w = wins.load(Ordering::SeqCst);
        assert!(
            (1..=3).contains(&w),
            "run {run}: at most 3 disjoint pairs fit in a ring of 6, got {w}"
        );
        let total: usize = (0..6)
            .map(|t| c.agenda(&format!("p{t}"), 0, 100).len())
            .sum();
        assert_eq!(total, w * 2, "run {run}");
    }
}
