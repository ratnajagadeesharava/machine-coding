#![allow(unused_imports, clippy::needless_range_loop)]
use p93_cron_job_scheduler::{contract::*, create};
use playground_support::{run_concurrently, sut, FakeClock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

const MIN: u64 = 60_000;

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

fn setup() -> Option<(Arc<dyn JobService>, FakeClock)> {
    let clock = FakeClock::new(0);
    Some((create(clock.as_clock())?, clock))
}

fn minutes(clock: &FakeClock, m: u64) {
    clock.advance(m * MIN);
}

fn counting(runs: &Arc<AtomicUsize>) -> JobFn {
    let runs = runs.clone();
    Arc::new(move || {
        runs.fetch_add(1, Ordering::SeqCst);
        Ok(())
    })
}

/// Fails the first `failures` calls, then succeeds.
fn flaky(failures: usize, calls: &Arc<AtomicUsize>) -> JobFn {
    let calls = calls.clone();
    Arc::new(move || {
        if calls.fetch_add(1, Ordering::SeqCst) < failures {
            Err("boom".to_string())
        } else {
            Ok(())
        }
    })
}

fn noop() -> JobFn {
    Arc::new(|| Ok(()))
}

fn always_fail() -> JobFn {
    Arc::new(|| Err("always".to_string()))
}

#[test]
fn should_run_an_every_five_minutes_job_on_time() {
    let (s, clock) = sut!(setup());
    let runs = Arc::new(AtomicUsize::new(0));
    s.define("j", "*/5 * *", counting(&runs), 1, 1_000, 1_000).unwrap();

    assert_eq!(s.run_due(), 0);
    minutes(&clock, 4);
    assert_eq!(s.run_due(), 0);
    minutes(&clock, 1);
    assert_eq!(s.run_due(), 1);
    assert_eq!(s.run_due(), 0); // nothing twice
    minutes(&clock, 5);
    assert_eq!(s.run_due(), 1);

    assert_eq!(runs.load(Ordering::SeqCst), 2);
    assert_eq!(s.history("j"), Ok(strs(&["5:1:OK", "10:1:OK"])));
    assert_eq!(s.next_slot("j"), Ok(Some(15)));
}

#[test]
fn should_not_run_in_the_minute_of_the_definition() {
    let (s, _) = sut!(setup());
    s.define("j", "* * *", noop(), 1, 1_000, 1_000).unwrap();

    assert_eq!(s.next_slot("j"), Ok(Some(1)));
    assert_eq!(s.run_due(), 0);
}

#[test]
fn should_run_a_daily_job_at_the_right_minute_of_the_day() {
    let (s, clock) = sut!(setup());
    s.define("daily", "30 9 *", noop(), 1, 1_000, 1_000).unwrap();

    assert_eq!(s.next_slot("daily"), Ok(Some(9 * 60 + 30)));
    minutes(&clock, 9 * 60 + 30);
    s.run_due();
    assert_eq!(s.next_slot("daily"), Ok(Some(1_440 + 9 * 60 + 30)));
}

#[test]
fn should_use_the_weekday_field_with_sunday_as_zero() {
    let (s, _) = sut!(setup());
    s.define("monday", "0 12 1", noop(), 1, 1_000, 1_000).unwrap(); // epoch day 0 is a Thursday
    s.define("sunday", "0 0 0", noop(), 1, 1_000, 1_000).unwrap();

    assert_eq!(s.next_slot("monday"), Ok(Some(4 * 1_440 + 12 * 60))); // Mon 1970-01-05 12:00
    assert_eq!(s.next_slot("sunday"), Ok(Some(3 * 1_440))); // Sun 1970-01-04 00:00
}

#[test]
fn should_understand_lists_and_ranges() {
    let (s, clock) = sut!(setup());
    s.define("j", "0,30 8-9 *", noop(), 1, 1_000, 1_000).unwrap();

    assert_eq!(s.next_slot("j"), Ok(Some(480))); // 08:00
    minutes(&clock, 480);
    s.run_due();
    assert_eq!(s.next_slot("j"), Ok(Some(510))); // 08:30
    minutes(&clock, 30);
    s.run_due();
    assert_eq!(s.next_slot("j"), Ok(Some(540))); // 09:00
    minutes(&clock, 30);
    s.run_due();
    assert_eq!(s.next_slot("j"), Ok(Some(570))); // 09:30
    minutes(&clock, 30);
    s.run_due();

    assert_eq!(s.history("j"), Ok(strs(&["480:1:OK", "510:1:OK", "540:1:OK", "570:1:OK"])));
    assert_eq!(s.next_slot("j"), Ok(Some(1_440 + 480))); // 10:00 is outside 8-9, so it wraps to the next day
}

#[test]
fn should_reject_bad_definitions() {
    let (s, _) = sut!(setup());
    s.define("ok", "* * *", noop(), 1, 1, 1).unwrap();
    let bad = JobError::InvalidDefinition;

    assert_eq!(s.define("ok", "* * *", noop(), 1, 1, 1), Err(JobError::InvalidDefinition));
    assert_eq!(s.define("a", "* *", noop(), 1, 1, 1), Err(JobError::InvalidDefinition));
    assert_eq!(s.define("b", "60 * *", noop(), 1, 1, 1), Err(JobError::InvalidDefinition));
    assert_eq!(s.define("c", "* 24 *", noop(), 1, 1, 1), Err(JobError::InvalidDefinition));
    assert_eq!(s.define("d", "* * 7", noop(), 1, 1, 1), Err(JobError::InvalidDefinition));
    assert_eq!(s.define("e", "*/0 * *", noop(), 1, 1, 1), Err(JobError::InvalidDefinition));
    assert_eq!(s.define("f", "5-1 * *", noop(), 1, 1, 1), Err(bad));
    assert_eq!(s.define("g", "* * *", noop(), 0, 1, 1), Err(JobError::InvalidDefinition));
    assert_eq!(s.define("h", "* * *", noop(), 1, 0, 1), Err(JobError::InvalidDefinition));
    assert_eq!(s.next_slot("nope"), Err(JobError::UnknownJob));
}

#[test]
fn should_run_only_once_for_many_missed_slots_and_forget_the_others() {
    let (s, clock) = sut!(setup());
    let runs = Arc::new(AtomicUsize::new(0));
    s.define("j", "*/5 * *", counting(&runs), 1, 1_000, 1_000).unwrap();

    minutes(&clock, 17);

    assert_eq!(s.run_due(), 1);
    assert_eq!(runs.load(Ordering::SeqCst), 1);
    assert_eq!(s.history("j"), Ok(strs(&["15:1:OK"]))); // latest matching minute <= 17
    assert_eq!(s.next_slot("j"), Ok(Some(20)));
}

#[test]
fn should_retry_with_exponential_backoff_until_it_succeeds() {
    let (s, clock) = sut!(setup());
    let calls = Arc::new(AtomicUsize::new(0));
    s.define("j", "*/10 * *", flaky(2, &calls), 5, 1_000, 60_000).unwrap();
    minutes(&clock, 10);

    assert_eq!(s.run_due(), 1); // attempt 1 fails
    clock.advance(999);
    assert_eq!(s.run_due(), 0); // retry due 1s after the failure
    clock.advance(1);
    assert_eq!(s.run_due(), 1); // attempt 2 fails
    clock.advance(1_999);
    assert_eq!(s.run_due(), 0); // second backoff is 2s
    clock.advance(1);
    assert_eq!(s.run_due(), 1); // attempt 3 succeeds

    assert_eq!(s.history("j"), Ok(strs(&["10:1:FAILED", "10:2:FAILED", "10:3:OK"])));
    assert!(s.dead_letters().is_empty());
    assert_eq!(s.next_slot("j"), Ok(Some(20)));
}

#[test]
fn should_cap_the_backoff() {
    let (s, clock) = sut!(setup());
    s.define("j", "*/60 * *", always_fail(), 6, 1_000, 3_000).unwrap();
    minutes(&clock, 60);

    s.run_due();
    for gap in [1_000u64, 2_000, 3_000, 3_000, 3_000] {
        clock.advance(gap - 1);
        assert_eq!(s.run_due(), 0, "too early for gap {gap}");
        clock.advance(1);
        assert_eq!(s.run_due(), 1);
    }

    let h = s.history("j").unwrap();
    assert_eq!(h.len(), 6);
    assert_eq!(h.last().unwrap(), "60:6:DEAD");
}

#[test]
fn should_give_up_after_the_last_attempt_and_keep_scheduling_the_next_slot() {
    let (s, clock) = sut!(setup());
    let calls = Arc::new(AtomicUsize::new(0));
    s.define("j", "*/10 * *", flaky(3, &calls), 3, 1_000, 1_000).unwrap();
    minutes(&clock, 10);

    s.run_due();
    clock.advance(1_000);
    s.run_due();
    clock.advance(1_000);
    s.run_due();

    assert_eq!(s.history("j"), Ok(strs(&["10:1:FAILED", "10:2:FAILED", "10:3:DEAD"])));
    assert_eq!(s.dead_letters(), strs(&["j:10"]));
    clock.advance(10 * MIN - 2_000);
    assert_eq!(s.run_due(), 1); // the next slot runs again (the job works now)
    assert_eq!(s.history("j").unwrap().last().unwrap(), "20:1:OK");
}

#[test]
fn should_skip_a_slot_that_arrives_while_the_job_is_still_retrying() {
    let (s, clock) = sut!(setup());
    let calls = Arc::new(AtomicUsize::new(0));
    s.define("j", "*/5 * *", flaky(1, &calls), 3, 8 * MIN, 8 * MIN).unwrap();
    minutes(&clock, 5);
    s.run_due(); // slot 5 fails, retry due at minute 13

    minutes(&clock, 5); // slot 10 arrives
    assert_eq!(s.run_due(), 0);
    minutes(&clock, 3); // minute 13: the retry runs
    assert_eq!(s.run_due(), 1);

    assert_eq!(s.history("j"), Ok(strs(&["5:1:FAILED", "10:0:SKIPPED", "5:2:OK"])));
    assert_eq!(s.next_slot("j"), Ok(Some(15)));
}

#[test]
fn should_run_due_jobs_in_due_time_then_definition_order() {
    let (s, clock) = sut!(setup());
    let order = Arc::new(Mutex::new(Vec::new()));
    for (name, cron) in [("b", "*/10 * *"), ("a", "*/5 * *"), ("c", "*/10 * *")] {
        let order = order.clone();
        let tag = name.to_string();
        s.define(name, cron, Arc::new(move || { order.lock().unwrap().push(tag.clone()); Ok(()) }), 1, 1, 1).unwrap();
    }

    minutes(&clock, 10);
    s.run_due();

    assert_eq!(*order.lock().unwrap(), strs(&["a", "b", "c"])); // a is due since minute 5; b and c since 10
}

#[test]
fn should_not_let_a_failing_job_stop_the_others() {
    let (s, clock) = sut!(setup());
    let runs = Arc::new(AtomicUsize::new(0));
    s.define("bad", "* * *", always_fail(), 1, 1_000, 1_000).unwrap();
    s.define("good", "* * *", counting(&runs), 1, 1_000, 1_000).unwrap();
    minutes(&clock, 1);

    assert_eq!(s.run_due(), 2);
    assert_eq!(runs.load(Ordering::SeqCst), 1);
}

#[test]
fn should_pause_resume_and_remove() {
    let (s, clock) = sut!(setup());
    let runs = Arc::new(AtomicUsize::new(0));
    s.define("j", "* * *", counting(&runs), 1, 1_000, 1_000).unwrap();

    s.pause("j").unwrap();
    assert_eq!(s.next_slot("j"), Ok(None));
    minutes(&clock, 5);
    assert_eq!(s.run_due(), 0);
    s.resume("j").unwrap();
    assert_eq!(s.next_slot("j"), Ok(Some(6)));
    minutes(&clock, 1);
    assert_eq!(s.run_due(), 1);
    s.remove("j").unwrap();
    minutes(&clock, 1);
    assert_eq!(s.run_due(), 0);
    assert_eq!(s.history("j"), Err(JobError::UnknownJob));
}

#[test]
fn should_execute_each_attempt_exactly_once_when_many_threads_call_run_due() {
    let (s, clock) = sut!(setup());
    let runs = Arc::new(AtomicUsize::new(0));
    s.define("j", "* * *", counting(&runs), 1, 1_000, 1_000).unwrap();
    let executions = AtomicUsize::new(0);

    for _ in 0..50 {
        minutes(&clock, 1);
        run_concurrently(8, |_| {
            executions.fetch_add(s.run_due(), Ordering::SeqCst);
        });
    }

    assert_eq!(runs.load(Ordering::SeqCst), 50);
    assert_eq!(executions.load(Ordering::SeqCst), 50);
    assert_eq!(s.history("j").unwrap().len(), 50);
}

#[test]
fn should_stay_correct_over_100_runs() {
    for run in 0..100 {
        let (s, clock) = sut!(setup());
        let calls = Arc::new(AtomicUsize::new(0));
        s.define("j", "* * *", flaky(1, &calls), 3, 1_000, 1_000).unwrap();
        minutes(&clock, 1);

        run_concurrently(6, |_| {
            s.run_due();
        });
        clock.advance(1_000);
        run_concurrently(6, |_| {
            s.run_due();
        });

        assert_eq!(calls.load(Ordering::SeqCst), 2, "run {run}");
        assert_eq!(s.history("j"), Ok(strs(&["1:1:FAILED", "1:2:OK"])), "run {run}");
    }
}
