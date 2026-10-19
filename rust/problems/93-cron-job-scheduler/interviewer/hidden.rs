#![allow(unused_imports, clippy::needless_range_loop)]
use p93_cron_job_scheduler::{contract::*, create};
use playground_support::{sut, FakeClock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[test]
fn should_handle_a_thousand_jobs_over_a_day() {
    let clock = FakeClock::new(0);
    let s = sut!(create(clock.as_clock()));
    let runs = Arc::new(AtomicUsize::new(0));
    for i in 0..1_000 {
        let runs = runs.clone();
        s.define(&format!("job{i}"), &format!("*/{} * *", 1 + i % 60), Arc::new(move || { runs.fetch_add(1, Ordering::SeqCst); Ok(()) }), 1, 1_000, 1_000).unwrap();
    }

    for _ in 1..=1_440 {
        clock.advance(60_000);
        s.run_due();
    }

    assert!(runs.load(Ordering::SeqCst) > 20_000);
}

#[test]
fn should_jump_over_weeks_of_downtime_with_one_run_per_job() {
    let clock = FakeClock::new(0);
    let s = sut!(create(clock.as_clock()));
    let runs = Arc::new(AtomicUsize::new(0));
    let r = runs.clone();
    s.define("j", "* * *", Arc::new(move || { r.fetch_add(1, Ordering::SeqCst); Ok(()) }), 1, 1_000, 1_000).unwrap();

    clock.advance(21 * 1_440 * 60_000);

    assert_eq!(s.run_due(), 1);
    assert_eq!(runs.load(Ordering::SeqCst), 1);
}
