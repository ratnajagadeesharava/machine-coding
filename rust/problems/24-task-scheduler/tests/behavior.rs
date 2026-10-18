#![allow(unused_imports, clippy::needless_range_loop)]
use p24_task_scheduler::{contract::*, create};
use playground_support::{collect_concurrently, eventually, sut, FakeClock};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const T0: u64 = 1_000;

type Log = Arc<Mutex<Vec<String>>>;

fn new_log() -> Log {
    Arc::new(Mutex::new(Vec::new()))
}

fn note(log: &Log, label: &str) -> Box<dyn FnOnce() + Send> {
    let (log, label) = (log.clone(), label.to_string());
    Box::new(move || log.lock().unwrap().push(label))
}

fn note_recurring(log: &Log, label: &str) -> Box<dyn FnMut() + Send> {
    let (log, label) = (log.clone(), label.to_string());
    Box::new(move || log.lock().unwrap().push(label.clone()))
}

fn entries(log: &Log) -> Vec<String> {
    log.lock().unwrap().clone()
}

fn scheduler(clock: &FakeClock) -> Option<Arc<dyn TaskSchedulerApi>> {
    create(clock.as_clock())
}

/// Runs `run_due` on another thread and fails (instead of hanging the suite) if it deadlocks.
fn run_due_without_deadlock(s: &Arc<dyn TaskSchedulerApi>) -> usize {
    let result = Arc::new(Mutex::new(None));
    let (s, r) = (s.clone(), result.clone());
    std::thread::spawn(move || {
        let n = s.run_due();
        *r.lock().unwrap() = Some(n);
    });
    assert!(
        eventually(10_000, || result.lock().unwrap().is_some()),
        "run_due did not return (deadlock?)"
    );
    let n = result.lock().unwrap().unwrap();
    n
}

#[test]
fn should_not_run_task_before_its_delay() {
    let clock = FakeClock::new(T0);
    let s = sut!(scheduler(&clock));
    let log = new_log();
    s.schedule_once(100, note(&log, "a"));

    assert_eq!(s.run_due(), 0);
    clock.advance(99);
    assert_eq!(s.run_due(), 0);
    clock.advance(1);
    assert_eq!(s.run_due(), 1);

    assert_eq!(entries(&log), vec!["a"]);
}

#[test]
fn should_run_zero_delay_task_immediately() {
    let clock = FakeClock::new(T0);
    let s = sut!(scheduler(&clock));
    let log = new_log();
    s.schedule_once(0, note(&log, "a"));

    assert_eq!(s.run_due(), 1);
    assert_eq!(entries(&log), vec!["a"]);
}

#[test]
fn should_run_one_shot_task_only_once() {
    let clock = FakeClock::new(T0);
    let s = sut!(scheduler(&clock));
    let log = new_log();
    s.schedule_once(10, note(&log, "a"));

    clock.advance(10);
    assert_eq!(s.run_due(), 1);
    clock.advance(1_000);
    assert_eq!(s.run_due(), 0);

    assert_eq!(entries(&log), vec!["a"]);
}

#[test]
fn should_run_due_tasks_in_due_time_then_submission_order() {
    let clock = FakeClock::new(T0);
    let s = sut!(scheduler(&clock));
    let log = new_log();
    s.schedule_once(50, note(&log, "a"));
    s.schedule_once(10, note(&log, "b"));
    s.schedule_once(50, note(&log, "c"));
    s.schedule_once(10, note(&log, "d"));
    s.schedule_once(500, note(&log, "late"));

    clock.advance(50);

    assert_eq!(s.run_due(), 4);
    assert_eq!(entries(&log), vec!["b", "d", "a", "c"]);
}

#[test]
fn should_return_unique_ids() {
    let clock = FakeClock::new(T0);
    let s = sut!(scheduler(&clock));
    let log = new_log();
    let a = s.schedule_once(10, note(&log, "a"));
    let b = s.schedule_once(10, note(&log, "b"));
    let c = s
        .schedule_recurring(10, 10, note_recurring(&log, "c"))
        .unwrap();

    let ids: HashSet<String> = [a, b, c].into_iter().collect();
    assert_eq!(ids.len(), 3);
}

#[test]
fn should_run_recurring_task_every_period() {
    let clock = FakeClock::new(T0);
    let s = sut!(scheduler(&clock));
    let log = new_log();
    s.schedule_recurring(10, 10, note_recurring(&log, "r"));

    clock.advance(10);
    assert_eq!(s.run_due(), 1);
    clock.advance(5);
    assert_eq!(s.run_due(), 0);
    clock.advance(5);
    assert_eq!(s.run_due(), 1);

    assert_eq!(entries(&log), vec!["r", "r"]);
}

#[test]
fn should_refuse_recurring_task_with_non_positive_period() {
    let clock = FakeClock::new(T0);
    let s = sut!(scheduler(&clock));
    let log = new_log();

    assert!(s
        .schedule_recurring(0, 0, note_recurring(&log, "r"))
        .is_none());

    assert_eq!(s.run_due(), 0);
}

#[test]
fn should_catch_up_every_missed_recurring_occurrence() {
    let clock = FakeClock::new(T0);
    let s = sut!(scheduler(&clock));
    let log = new_log();
    s.schedule_recurring(100, 10, note_recurring(&log, "r")); // due 1100, 1110, 1120, ...

    clock.advance(135); // now 1135: 1100, 1110, 1120, 1130 are missed
    assert_eq!(s.run_due(), 4);
    assert_eq!(s.run_due(), 0);
    clock.advance(5); // now 1140
    assert_eq!(s.run_due(), 1);
}

#[test]
fn should_keep_fixed_rate_when_the_host_is_late() {
    let clock = FakeClock::new(T0);
    let s = sut!(scheduler(&clock));
    let log = new_log();
    s.schedule_recurring(0, 100, note_recurring(&log, "r")); // due 1000, 1100, 1200, 1300, ...

    assert_eq!(s.run_due(), 1); // 1000
    clock.advance(250); // now 1250: 1100, 1200
    assert_eq!(s.run_due(), 2);
    clock.advance(49); // now 1299
    assert_eq!(s.run_due(), 0);
    clock.advance(1); // now 1300, not 1350: next due = previous due + period
    assert_eq!(s.run_due(), 1);
}

#[test]
fn should_interleave_missed_occurrences_with_other_tasks_by_due_time() {
    let clock = FakeClock::new(T0);
    let s = sut!(scheduler(&clock));
    let log = new_log();
    s.schedule_recurring(10, 10, note_recurring(&log, "r")); // 1010, 1020, 1030
    s.schedule_once(25, note(&log, "o")); // 1025

    clock.advance(30);

    assert_eq!(s.run_due(), 4);
    assert_eq!(entries(&log), vec!["r", "r", "o", "r"]);
}

#[test]
fn should_not_run_cancelled_one_shot_task() {
    let clock = FakeClock::new(T0);
    let s = sut!(scheduler(&clock));
    let log = new_log();
    let id = s.schedule_once(10, note(&log, "a"));

    assert!(s.cancel(&id));
    clock.advance(10);

    assert_eq!(s.run_due(), 0);
    assert!(entries(&log).is_empty());
}

#[test]
fn should_stop_recurring_task_when_cancelled() {
    let clock = FakeClock::new(T0);
    let s = sut!(scheduler(&clock));
    let log = new_log();
    let id = s
        .schedule_recurring(10, 10, note_recurring(&log, "r"))
        .unwrap();
    clock.advance(10);
    s.run_due();

    assert!(s.cancel(&id));
    clock.advance(100);

    assert_eq!(s.run_due(), 0);
    assert_eq!(entries(&log), vec!["r"]);
}

#[test]
fn should_refuse_to_cancel_unknown_finished_or_already_cancelled_tasks() {
    let clock = FakeClock::new(T0);
    let s = sut!(scheduler(&clock));
    let log = new_log();
    let done = s.schedule_once(0, note(&log, "done"));
    s.run_due();
    let cancelled = s.schedule_once(10, note(&log, "x"));
    s.cancel(&cancelled);

    assert!(!s.cancel("nope"));
    assert!(!s.cancel(&done));
    assert!(!s.cancel(&cancelled));
}

#[test]
fn should_prevent_run_of_a_task_that_is_due_but_not_run_yet() {
    let clock = FakeClock::new(T0);
    let s = sut!(scheduler(&clock));
    let log = new_log();
    let a = s.schedule_once(10, note(&log, "a"));
    s.schedule_once(10, note(&log, "b"));
    clock.advance(50); // both are overdue, run_due has not been called

    assert!(s.cancel(&a));

    assert_eq!(s.run_due(), 1);
    assert_eq!(entries(&log), vec!["b"]);
}

#[test]
fn should_continue_with_other_tasks_when_a_task_panics() {
    let clock = FakeClock::new(T0);
    let s = sut!(scheduler(&clock));
    let log = new_log();
    s.schedule_once(0, Box::new(|| panic!("boom")));
    s.schedule_once(0, note(&log, "after"));

    assert_eq!(s.run_due(), 2);
    assert_eq!(entries(&log), vec!["after"]);
}

#[test]
fn should_keep_a_recurring_task_recurring_after_it_panics() {
    let clock = FakeClock::new(T0);
    let s = sut!(scheduler(&clock));
    let attempts = Arc::new(AtomicUsize::new(0));
    let a = attempts.clone();
    s.schedule_recurring(
        0,
        10,
        Box::new(move || {
            a.fetch_add(1, Ordering::SeqCst);
            panic!("boom");
        }),
    );

    assert_eq!(s.run_due(), 1);
    clock.advance(20);
    assert_eq!(s.run_due(), 2);

    assert_eq!(attempts.load(Ordering::SeqCst), 3);
}

#[test]
fn should_let_a_task_schedule_further_work() {
    let clock = FakeClock::new(T0);
    let s = sut!(scheduler(&clock));
    let log = new_log();
    {
        let (s2, log2) = (s.clone(), log.clone());
        s.schedule_once(
            0,
            Box::new(move || {
                log2.lock().unwrap().push("outer".to_string());
                s2.schedule_once(100, note(&log2, "later"));
            }),
        );
    }

    assert_eq!(run_due_without_deadlock(&s), 1);
    assert_eq!(s.run_due(), 0);
    clock.advance(100);
    assert_eq!(s.run_due(), 1);

    assert_eq!(entries(&log), vec!["outer", "later"]);
}

#[test]
fn should_run_work_scheduled_from_inside_a_task_in_the_same_call_when_already_due() {
    let clock = FakeClock::new(T0);
    let s = sut!(scheduler(&clock));
    let log = new_log();
    {
        let (s2, log2) = (s.clone(), log.clone());
        s.schedule_once(
            0,
            Box::new(move || {
                log2.lock().unwrap().push("outer".to_string());
                s2.schedule_once(0, note(&log2, "inner"));
            }),
        );
    }
    s.schedule_once(0, note(&log, "sibling"));

    assert_eq!(run_due_without_deadlock(&s), 3);
    assert_eq!(entries(&log), vec!["outer", "sibling", "inner"]);
}

#[test]
fn should_let_a_task_cancel_another_due_task() {
    let clock = FakeClock::new(T0);
    let s = sut!(scheduler(&clock));
    let log = new_log();
    let victim: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let cancelled = Arc::new(AtomicBool::new(false));
    {
        let (s2, v, c) = (s.clone(), victim.clone(), cancelled.clone());
        s.schedule_once(
            10,
            Box::new(move || {
                let id = v.lock().unwrap().clone().unwrap();
                c.store(s2.cancel(&id), Ordering::SeqCst);
            }),
        );
    }
    *victim.lock().unwrap() = Some(s.schedule_once(20, note(&log, "victim")));
    clock.advance(30);

    assert_eq!(run_due_without_deadlock(&s), 1);

    assert!(cancelled.load(Ordering::SeqCst));
    assert!(entries(&log).is_empty());
}

#[test]
fn should_let_a_recurring_task_cancel_itself_and_stop_the_catch_up() {
    let clock = FakeClock::new(T0);
    let s = sut!(scheduler(&clock));
    let me: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let runs = Arc::new(AtomicUsize::new(0));
    let cancel_result = Arc::new(AtomicBool::new(false));
    {
        let (s2, me2, r, c) = (s.clone(), me.clone(), runs.clone(), cancel_result.clone());
        let id = s.schedule_recurring(
            10,
            10,
            Box::new(move || {
                if r.fetch_add(1, Ordering::SeqCst) + 1 == 3 {
                    let id = me2.lock().unwrap().clone().unwrap();
                    c.store(s2.cancel(&id), Ordering::SeqCst);
                }
            }),
        );
        *me.lock().unwrap() = id;
    }
    clock.advance(100); // ten occurrences are missed

    assert_eq!(run_due_without_deadlock(&s), 3);

    assert!(cancel_result.load(Ordering::SeqCst));
    clock.advance(100);
    assert_eq!(s.run_due(), 0);
    assert_eq!(runs.load(Ordering::SeqCst), 3);
}

#[test]
fn should_run_every_task_exactly_once_when_many_threads_submit() {
    let clock = FakeClock::new(T0);
    let s = sut!(scheduler(&clock));
    let (threads, per_thread) = (16usize, 100usize);
    let runs: Arc<Vec<AtomicUsize>> = Arc::new(
        (0..threads * per_thread)
            .map(|_| AtomicUsize::new(0))
            .collect(),
    );

    let ids = collect_concurrently(threads, |t| {
        (0..per_thread)
            .map(|k| {
                let n = t * per_thread + k;
                let runs = runs.clone();
                s.schedule_once(
                    (k % 3) as u64,
                    Box::new(move || {
                        runs[n].fetch_add(1, Ordering::SeqCst);
                    }),
                )
            })
            .collect::<Vec<String>>()
    });
    clock.advance(10);

    let distinct: HashSet<String> = ids.into_iter().flatten().collect();
    assert_eq!(distinct.len(), threads * per_thread);
    assert_eq!(s.run_due(), threads * per_thread);
    for (n, r) in runs.iter().enumerate() {
        assert_eq!(r.load(Ordering::SeqCst), 1, "task {n}");
    }
}

#[test]
fn should_run_each_occurrence_once_when_many_threads_call_run_due() {
    let clock = FakeClock::new(T0);
    let s = sut!(scheduler(&clock));
    let (threads, per_thread) = (8usize, 50usize);
    let runs: Arc<Vec<AtomicUsize>> = Arc::new(
        (0..threads * per_thread)
            .map(|_| AtomicUsize::new(0))
            .collect(),
    );

    let executed = collect_concurrently(threads, |t| {
        let mut mine = 0;
        for k in 0..per_thread {
            let n = t * per_thread + k;
            let runs = runs.clone();
            s.schedule_once(
                0,
                Box::new(move || {
                    runs[n].fetch_add(1, Ordering::SeqCst);
                }),
            );
            mine += s.run_due();
        }
        mine
    });

    let total: usize = executed.iter().sum::<usize>() + s.run_due();
    assert_eq!(total, threads * per_thread);
    for (n, r) in runs.iter().enumerate() {
        assert_eq!(r.load(Ordering::SeqCst), 1, "task {n}");
    }
}

#[test]
fn should_not_run_a_task_that_was_cancelled_concurrently_after_cancel_returned_true() {
    let clock = FakeClock::new(T0);
    let s = sut!(scheduler(&clock));
    let n = 400usize;
    let runs: Arc<Vec<AtomicUsize>> = Arc::new((0..n).map(|_| AtomicUsize::new(0)).collect());
    let ids: Vec<String> = (0..n)
        .map(|i| {
            let runs = runs.clone();
            s.schedule_once(
                0,
                Box::new(move || {
                    runs[i].fetch_add(1, Ordering::SeqCst);
                }),
            )
        })
        .collect();

    let results = collect_concurrently(2, |t| {
        if t == 0 {
            s.run_due();
            return true;
        }
        // Cancels in reverse so that it contends with the runner.
        let mut ok = true;
        for i in (n / 2..n).rev() {
            if s.cancel(&ids[i]) && runs[i].load(Ordering::SeqCst) != 0 {
                ok = false; // cancelled yet it had already run
            }
        }
        ok
    });

    assert!(results[1]);
    for r in runs.iter() {
        assert!(r.load(Ordering::SeqCst) <= 1);
    }
}

#[test]
fn should_handle_many_pending_tasks_without_scanning_them_all_on_every_call() {
    let clock = FakeClock::new(T0);
    let s = sut!(scheduler(&clock));
    let n = 50_000u64;
    let executed = Arc::new(AtomicUsize::new(0));
    let start = Instant::now();

    for i in 1..=n {
        let e = executed.clone();
        s.schedule_once(
            i,
            Box::new(move || {
                e.fetch_add(1, Ordering::SeqCst);
            }),
        );
    }
    for _ in 1..=n {
        clock.advance(1);
        assert_eq!(s.run_due(), 1);
    }

    assert_eq!(executed.load(Ordering::SeqCst), n as usize);
    assert!(start.elapsed() < Duration::from_secs(30));
}
