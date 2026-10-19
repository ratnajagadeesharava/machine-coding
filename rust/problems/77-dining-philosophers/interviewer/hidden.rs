#![allow(unused_imports, clippy::needless_range_loop)]
use p77_dining_philosophers::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

#[test]
fn should_not_starve_anyone_when_neighbours_are_greedy() {
    let seats = 5;
    let t = sut!(create(seats));
    let meals: Vec<AtomicUsize> = (0..seats).map(|_| AtomicUsize::new(0)).collect();

    run_concurrently(seats, |p| {
        // even philosophers eat back to back, odd ones are slower; everyone must still finish
        let count = if p % 2 == 0 { 20_000 } else { 2_000 };
        for _ in 0..count {
            t.dine(p, &mut || {
                meals[p].fetch_add(1, Ordering::SeqCst);
                if p % 2 == 1 {
                    thread::yield_now();
                }
            })
            .unwrap();
        }
    });

    for p in 0..seats {
        assert_eq!(meals[p].load(Ordering::SeqCst), if p % 2 == 0 { 20_000 } else { 2_000 });
    }
}

#[test]
fn should_survive_one_hundred_philosophers() {
    let seats = 100;
    let t = sut!(create(seats));
    let meals: Vec<AtomicUsize> = (0..seats).map(|_| AtomicUsize::new(0)).collect();

    run_concurrently(seats, |p| {
        for _ in 0..300 {
            t.dine(p, &mut || {
                meals[p].fetch_add(1, Ordering::SeqCst);
            })
            .unwrap();
        }
    });

    for p in 0..seats {
        assert_eq!(meals[p].load(Ordering::SeqCst), 300);
    }
}
