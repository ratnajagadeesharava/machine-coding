#![allow(unused_imports, clippy::needless_range_loop)]
use p24_task_scheduler::create;
use playground_support::{sut, FakeClock};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

fn push(log: &Arc<Mutex<Vec<String>>>, label: &str) -> Box<dyn FnOnce() + Send> {
    let (log, label) = (log.clone(), label.to_string());
    Box::new(move || log.lock().unwrap().push(label))
}

#[test]
fn same_due_time_should_follow_submission_order_across_task_kinds() {
    let clock = FakeClock::new(5_000);
    let s = sut!(create(clock.as_clock()));
    let log = Arc::new(Mutex::new(Vec::new()));
    s.schedule_once(20, push(&log, "once-1"));
    let l = log.clone();
    s.schedule_recurring(
        20,
        100,
        Box::new(move || l.lock().unwrap().push("rec".to_string())),
    );
    s.schedule_once(20, push(&log, "once-2"));

    clock.advance(20);

    assert_eq!(s.run_due(), 3);
    assert_eq!(*log.lock().unwrap(), vec!["once-1", "rec", "once-2"]);
}

#[test]
fn should_catch_up_a_thousand_occurrences_of_a_one_millisecond_period() {
    let clock = FakeClock::new(5_000);
    let s = sut!(create(clock.as_clock()));
    let runs = Arc::new(AtomicUsize::new(0));
    let r = runs.clone();
    s.schedule_recurring(
        0,
        1,
        Box::new(move || {
            r.fetch_add(1, Ordering::SeqCst);
        }),
    ); // due at 5000, 5001, ...

    clock.advance(999); // now 5999 -> occurrences 5000..=5999

    assert_eq!(s.run_due(), 1_000);
    assert_eq!(runs.load(Ordering::SeqCst), 1_000);
}

#[test]
fn a_one_shot_cancelling_itself_should_be_refused_because_it_already_started() {
    let clock = FakeClock::new(5_000);
    let s = sut!(create(clock.as_clock()));
    let me: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let result = Arc::new(AtomicBool::new(true));
    let (s2, me2, r) = (s.clone(), me.clone(), result.clone());
    let id = s.schedule_once(
        0,
        Box::new(move || {
            let id = me2.lock().unwrap().clone().unwrap();
            r.store(s2.cancel(&id), Ordering::SeqCst);
        }),
    );
    *me.lock().unwrap() = Some(id);

    assert_eq!(s.run_due(), 1);

    assert!(!result.load(Ordering::SeqCst));
}

#[test]
fn calling_run_due_twice_in_a_row_should_run_nothing_the_second_time() {
    let clock = FakeClock::new(5_000);
    let s = sut!(create(clock.as_clock()));
    let log = Arc::new(Mutex::new(Vec::new()));
    s.schedule_once(0, push(&log, "a"));
    let l = log.clone();
    s.schedule_recurring(
        0,
        50,
        Box::new(move || l.lock().unwrap().push("r".to_string())),
    );

    assert_eq!(s.run_due(), 2);
    assert_eq!(s.run_due(), 0);
    assert_eq!(*log.lock().unwrap(), vec!["a", "r"]);
}

#[test]
fn a_panicking_one_shot_should_still_count_as_an_execution_and_not_run_again() {
    let clock = FakeClock::new(5_000);
    let s = sut!(create(clock.as_clock()));
    let attempts = Arc::new(AtomicUsize::new(0));
    let a = attempts.clone();
    s.schedule_once(
        0,
        Box::new(move || {
            a.fetch_add(1, Ordering::SeqCst);
            panic!("boom");
        }),
    );

    assert_eq!(s.run_due(), 1);
    clock.advance(1_000);
    assert_eq!(s.run_due(), 0);
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
}
