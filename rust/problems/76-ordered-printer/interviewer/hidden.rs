#![allow(unused_imports, clippy::needless_range_loop)]
use p76_ordered_printer::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::Mutex;

#[test]
fn should_hand_over_the_turn_twenty_thousand_times_quickly() {
    let workers = 4;
    let p = sut!(create(workers));
    let log = Mutex::new(Vec::new());

    run_concurrently(workers, |id| {
        for _ in 0..5_000 {
            p.take_turn(id, &mut || log.lock().unwrap().push(id))
                .unwrap();
        }
    });

    let log = log.into_inner().unwrap();
    assert_eq!(log.len(), 20_000);
    for (i, w) in log.iter().enumerate() {
        assert_eq!(*w, i % workers);
    }
}

#[test]
fn should_support_a_ring_of_one() {
    let p = sut!(create(1));
    let mut n = 0;

    for _ in 0..1_000 {
        p.take_turn(0, &mut || n += 1).unwrap();
    }

    assert_eq!(n, 1_000);
}
