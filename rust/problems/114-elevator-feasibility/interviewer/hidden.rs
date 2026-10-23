#![allow(unused_imports, clippy::needless_range_loop)]
use p114_elevator_feasibility::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// Small deterministic generator, so the scenarios never depend on a random seed from the environment.
struct Lcg(u64);
impl Lcg {
    fn next(&mut self, bound: u32) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) % bound as u64) as u32
    }
}

#[test]
fn should_count_a_stop_at_the_current_floor_as_an_extra_stop_for_onboard_riders() {
    let l = sut!(create(10, 4, 0));
    l.request("r1", 0, 9, 1).unwrap();
    l.step(); // at 0, r1 onboard

    // doors would reopen at 0 once more: r1 would see an intermediate stop
    assert_eq!(l.request("r2", 0, 9, 1), Ok(false));
    assert_eq!(l.plan(), vec![9]);
}

#[test]
fn should_keep_promises_across_many_accepted_requests() {
    let l = sut!(create(12, 6, 2));
    let mut rnd = Lcg(7);
    let mut accepted = vec![];
    for i in 0..60 {
        let from = rnd.next(12);
        let to = rnd.next(12);
        if from == to {
            continue;
        }
        let id = format!("r{i}");
        if l.request(&id, from, to, 1 + rnd.next(3)).unwrap() {
            accepted.push(id);
        }
        if i % 7 == 0 {
            l.step();
        }
    }
    assert!(!accepted.is_empty());

    let planned = l.plan();
    let mut executed = vec![];
    while let Some(s) = l.step() {
        executed.push(s);
        assert!(l.onboard() <= 6);
    }
    assert_eq!(executed, planned);
    for id in &accepted {
        assert_eq!(l.status(id), "DELIVERED");
    }
}

#[test]
fn should_leave_the_plan_identical_after_many_rejections() {
    let l = sut!(create(10, 4, 0));
    l.request("a", 0, 5, 4).unwrap();
    l.request("b", 5, 9, 4).unwrap();
    l.step();
    let before = l.plan();
    let floor = l.current_floor();
    let load = l.onboard();

    for i in 0..100u32 {
        assert_eq!(
            l.request(&format!("x{i}"), 1 + i % 4, 2 + i % 4, 1 + i % 4),
            Ok(false)
        );
    }

    assert_eq!(l.plan(), before);
    assert_eq!(l.current_floor(), floor);
    assert_eq!(l.onboard(), load);
    for i in 0..100 {
        assert_eq!(l.status(&format!("x{i}")), "UNKNOWN");
    }
}

#[test]
fn should_free_capacity_after_riders_are_delivered() {
    let l = sut!(create(10, 4, 0));
    for i in 0..4 {
        assert_eq!(l.request(&format!("a{i}"), 0, 5, 1), Ok(true));
    }
    assert_eq!(l.request("late", 0, 5, 1), Ok(false));

    assert_eq!(l.step(), Some(0));
    assert_eq!(l.step(), Some(5));

    assert_eq!(l.request("late", 0, 5, 4), Ok(true));
    assert_eq!(l.plan(), vec![0, 5]);
}

#[test]
fn should_wait_a_full_lap_for_a_rider_whose_direction_is_already_past() {
    let l = sut!(create(10, 4, 3));
    l.request("down", 7, 1, 1).unwrap();
    l.step(); // 7, heading down
    l.step(); // 1, delivered; the lift is on its way down at floor 1

    l.request("up", 4, 6, 1).unwrap();
    l.request("down2", 8, 3, 1).unwrap();

    // from floor 1 heading down: bottom turn, up through 4 -> 6, top turn, down through 8 -> 3
    assert_eq!(l.plan(), vec![4, 6, 8, 3]);
}

#[test]
fn should_reject_when_a_later_boarding_overlaps_a_waiting_rider_on_the_return_sweep() {
    let l = sut!(create(10, 4, 5));
    assert_eq!(l.request("d1", 9, 0, 3), Ok(true));

    assert_eq!(l.request("d2", 6, 2, 2), Ok(false)); // d1 still inside (3 + 2)
    assert_eq!(l.request("d3", 6, 2, 1), Ok(true));
    assert_eq!(l.plan(), vec![9, 6, 2, 0]);
}

#[test]
fn should_handle_a_tall_building_with_many_requests() {
    let l = sut!(create(500, 20, 400));
    let mut rnd = Lcg(11);
    let mut accepted = 0;
    for i in 0..300 {
        let from = rnd.next(500);
        let to = rnd.next(500);
        if from != to && l.request(&format!("t{i}"), from, to, 1).unwrap() {
            accepted += 1;
        }
    }
    assert!(accepted > 50);

    let mut stops = 0;
    while l.step().is_some() {
        stops += 1;
    }
    assert!(stops > 0 && stops <= 2 * accepted);
    assert_eq!(l.onboard(), 0);
}

#[test]
fn should_keep_the_capacity_promise_while_requests_and_steps_race() {
    let l = sut!(create(20, 5, 6));
    let overloaded = AtomicBool::new(false);
    let accepted_ids = Mutex::new(vec![]);

    run_concurrently(6, |t| {
        if t == 5 {
            for _ in 0..3_000 {
                l.step();
                if l.onboard() > 5 {
                    overloaded.store(true, Ordering::SeqCst);
                }
            }
            return;
        }
        let mut rnd = Lcg(100 + t as u64);
        for i in 0..100 {
            let from = rnd.next(20);
            let to = rnd.next(20);
            if from == to {
                continue;
            }
            let id = format!("r-{t}-{i}");
            if l.request(&id, from, to, 1 + rnd.next(2)).unwrap() {
                accepted_ids.lock().unwrap().push(id);
            }
        }
    });

    assert!(!overloaded.load(Ordering::SeqCst));
    let ids = accepted_ids.lock().unwrap();
    assert!(!ids.is_empty());
    while l.step().is_some() {
        assert!(l.onboard() <= 5);
    }
    for id in ids.iter() {
        assert_eq!(l.status(id), "DELIVERED");
    }
}
