#![allow(unused_imports, clippy::needless_range_loop)]
use p88_event_calendar::{contract::*, create};
use playground_support::{run_concurrently, sut};

#[test]
fn should_handle_a_year_of_daily_meetings() {
    let c = sut!(create());

    let ids = c.schedule_recurring("ann", "daily", 0, 30, &["bob"], 1_440, 365).unwrap();

    assert_eq!(ids.len(), 365);
    assert_eq!(c.agenda("bob", 0, 365 * 1_440).len(), 365);
}

#[test]
fn should_not_deadlock_with_large_crossing_groups() {
    let c = sut!(create());
    let everyone: Vec<String> = (0..30).map(|i| format!("u{i}")).collect();

    run_concurrently(30, |t| {
        let others: Vec<&str> = everyone.iter().enumerate().filter(|(i, _)| *i != t).map(|(_, s)| s.as_str()).collect();
        for i in 0..100u64 {
            let _ = c.schedule(&everyone[t], "all", i * 60, i * 60 + 30, &others);
        }
    });

    assert_eq!(c.agenda("u0", 0, 10_000).len(), 100);
}
