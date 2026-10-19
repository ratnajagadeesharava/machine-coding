#![allow(unused_imports, clippy::needless_range_loop)]
use p77_dining_philosophers::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Every philosopher eats `meals` times. Returns (times two neighbours ate together, meals per philosopher).
fn run_meals(t: &Arc<dyn Table>, seats: usize, meals: usize, inside_meal: &(dyn Fn() + Sync)) -> (usize, Vec<usize>) {
    let eating: Vec<AtomicUsize> = (0..seats).map(|_| AtomicUsize::new(0)).collect();
    let eaten: Vec<AtomicUsize> = (0..seats).map(|_| AtomicUsize::new(0)).collect();
    let violations = AtomicUsize::new(0);
    run_concurrently(seats, |p| {
        for _ in 0..meals {
            t.dine(p, &mut || {
                eating[p].store(1, Ordering::SeqCst);
                if eating[(p + 1) % seats].load(Ordering::SeqCst) == 1
                    || eating[(p + seats - 1) % seats].load(Ordering::SeqCst) == 1
                {
                    violations.fetch_add(1, Ordering::SeqCst);
                }
                inside_meal();
                eating[p].store(0, Ordering::SeqCst);
                eaten[p].fetch_add(1, Ordering::SeqCst);
            })
            .unwrap();
        }
    });
    (
        violations.load(Ordering::SeqCst),
        eaten.iter().map(|e| e.load(Ordering::SeqCst)).collect(),
    )
}

#[test]
fn should_let_a_single_philosopher_eat() {
    let t = sut!(create(5));
    let mut meals = 0;

    t.dine(2, &mut || meals += 1).unwrap();

    assert_eq!(meals, 1);
}

#[test]
fn should_never_let_neighbours_eat_at_the_same_time() {
    let t = sut!(create(5));

    let (violations, _) = run_meals(&t, 5, 500, &thread::yield_now);

    assert_eq!(violations, 0, "neighbours eating simultaneously");
}

#[test]
fn should_feed_everyone_every_time_without_deadlock() {
    let t = sut!(create(5));

    let (_, eaten) = run_meals(&t, 5, 2_000, &|| {});

    assert_eq!(eaten, vec![2_000; 5]);
}

#[test]
fn should_work_for_two_seats_where_both_forks_are_shared() {
    let t = sut!(create(2));

    let (violations, eaten) = run_meals(&t, 2, 2_000, &thread::yield_now);

    assert_eq!(violations, 0);
    assert_eq!(eaten, vec![2_000, 2_000]);
}

#[test]
fn should_work_for_other_table_sizes() {
    for seats in [3, 4, 7, 12] {
        let t = sut!(create(seats));

        let (violations, eaten) = run_meals(&t, seats, 300, &thread::yield_now);

        assert_eq!(violations, 0, "{seats} seats");
        assert_eq!(eaten, vec![300; seats]);
    }
}

#[test]
fn should_let_non_neighbours_eat_in_parallel() {
    let seats = 5;
    let t = sut!(create(seats));
    let eating_now = AtomicUsize::new(0);
    let max_eating = AtomicUsize::new(0);

    run_concurrently(seats, |p| {
        for _ in 0..20 {
            t.dine(p, &mut || {
                max_eating.fetch_max(eating_now.fetch_add(1, Ordering::SeqCst) + 1, Ordering::SeqCst);
                thread::sleep(Duration::from_millis(10));
                eating_now.fetch_sub(1, Ordering::SeqCst);
            })
            .unwrap();
        }
    });

    let max = max_eating.load(Ordering::SeqCst);
    assert!(max >= 2, "a global lock would make this 1, saw {max}");
    assert!(max <= seats / 2, "at most floor(seats / 2) can eat at once, saw {max}");
}

#[test]
fn should_put_the_forks_down_when_eating_panics() {
    let t = sut!(create(3));

    let result = catch_unwind(AssertUnwindSafe(|| {
        let _ = t.dine(0, &mut || panic!("choked"));
    }));
    assert!(result.is_err(), "the panic propagates to the caller");

    let meals = AtomicUsize::new(0);
    run_concurrently(3, |p| {
        t.dine(p, &mut || {
            meals.fetch_add(1, Ordering::SeqCst);
        })
        .unwrap();
    });
    assert_eq!(meals.load(Ordering::SeqCst), 3);
}

#[test]
fn should_reject_unknown_philosophers() {
    let t = sut!(create(5));

    assert_eq!(t.dine(5, &mut || {}), Err(UnknownPhilosopher));
    assert_eq!(t.dine(99, &mut || {}), Err(UnknownPhilosopher));
}

#[test]
fn should_stay_correct_over_100_runs() {
    for run in 0..100 {
        let seats = 2 + run % 5;
        let t = sut!(create(seats));

        let (violations, eaten) = run_meals(&t, seats, 200, &thread::yield_now);

        assert_eq!(violations, 0, "run {run} ({seats} seats)");
        assert_eq!(eaten, vec![200; seats], "run {run}");
    }
}
