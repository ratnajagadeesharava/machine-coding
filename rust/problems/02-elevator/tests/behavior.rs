#![allow(unused_imports, clippy::needless_range_loop)]
use p02_elevator::{contract::*, create};
use playground_support::sut;
use std::collections::HashSet;

/// Steps up to `max_steps` and returns the floors at which the elevator opened its doors, in order.
fn served(e: &mut dyn ElevatorApi, elevator: usize, max_steps: usize) -> Vec<i32> {
    let mut floors = Vec::new();
    for _ in 0..max_steps {
        e.step();
        if e.doors_open(elevator) {
            floors.push(e.floor_of(elevator));
        }
    }
    floors
}

/// Tiny deterministic PRNG so the tests need no dependencies.
struct Lcg(u64);
impl Lcg {
    fn next(&mut self, bound: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) % bound
    }
}

#[test]
fn should_move_one_floor_per_step_and_open_doors_on_arrival() {
    let mut e = sut!(create(10, 1));
    assert!(e.request_pickup(3, "UP"));

    e.step();
    assert_eq!(e.floor_of(0), 1);
    assert!(!e.doors_open(0));
    e.step();
    assert_eq!(e.floor_of(0), 2);
    e.step();
    assert_eq!(e.floor_of(0), 3);
    assert!(e.doors_open(0));
    e.step();
    assert_eq!(e.floor_of(0), 3);
    assert!(!e.doors_open(0));
}

#[test]
fn should_open_doors_without_moving_for_request_on_current_floor() {
    let mut e = sut!(create(10, 1));
    assert!(e.request_destination(0, 0));

    e.step();

    assert_eq!(e.floor_of(0), 0);
    assert!(e.doors_open(0));
}

#[test]
fn should_stop_at_intermediate_floor_in_travel_direction() {
    let mut e = sut!(create(10, 1));
    e.request_destination(0, 5);
    e.request_pickup(3, "UP");

    assert_eq!(served(e.as_mut(), 0, 12), vec![3, 5]);
}

#[test]
fn should_not_stop_for_opposite_direction_pickup_until_it_returns() {
    let mut e = sut!(create(10, 1));
    e.request_destination(0, 5);
    e.request_pickup(3, "DOWN");

    assert_eq!(served(e.as_mut(), 0, 12), vec![5, 3]);
}

#[test]
fn should_pick_up_new_requests_ahead_of_moving_elevator() {
    let mut e = sut!(create(10, 1));
    e.request_destination(0, 5);
    e.step();
    e.step();
    assert_eq!(e.floor_of(0), 2);

    e.request_destination(0, 3);

    assert_eq!(served(e.as_mut(), 0, 10), vec![3, 5]);
}

#[test]
fn should_serve_requests_behind_after_finishing_current_direction() {
    let mut e = sut!(create(10, 1));
    e.request_destination(0, 5);
    e.step();
    e.step();

    e.request_destination(0, 1);

    assert_eq!(served(e.as_mut(), 0, 12), vec![5, 1]);
}

#[test]
fn should_reject_invalid_requests_without_side_effects() {
    let mut e = sut!(create(10, 2));

    assert!(!e.request_pickup(10, "UP"));
    assert!(!e.request_pickup(-1, "UP"));
    assert!(!e.request_pickup(3, "SIDEWAYS"));
    assert!(!e.request_pickup(0, "DOWN"));
    assert!(!e.request_pickup(9, "UP"));
    assert!(!e.request_destination(2, 4));
    assert!(!e.request_destination(0, 99));

    for _ in 0..20 {
        e.step();
        assert_eq!(e.floor_of(0), 0);
        assert_eq!(e.floor_of(1), 0);
        assert!(!e.doors_open(0));
    }
}

#[test]
fn should_not_serve_duplicate_request_twice() {
    let mut e = sut!(create(10, 1));
    e.request_pickup(3, "UP");
    e.request_pickup(3, "UP");
    e.request_destination(0, 3);

    assert_eq!(served(e.as_mut(), 0, 10), vec![3]);
}

#[test]
fn should_stay_idle_when_nothing_is_requested() {
    let mut e = sut!(create(10, 3));
    for _ in 0..50 {
        e.step();
    }
    for k in 0..3 {
        assert_eq!(e.floor_of(k), 0);
        assert!(!e.doors_open(k));
    }
}

#[test]
fn should_send_only_one_elevator_to_a_hall_call() {
    let mut e = sut!(create(10, 2));
    e.request_pickup(5, "UP");

    for _ in 0..5 {
        e.step();
    }

    let at_five = (0..2)
        .filter(|&k| e.floor_of(k) == 5 && e.doors_open(k))
        .count();
    let at_zero = (0..2).filter(|&k| e.floor_of(k) == 0).count();
    assert_eq!(at_five, 1);
    assert_eq!(at_zero, 1);
}

#[test]
fn should_prefer_elevator_already_heading_toward_the_call() {
    let mut e = sut!(create(12, 2));
    e.request_destination(0, 8);
    for _ in 0..3 {
        e.step();
    }
    assert_eq!(e.floor_of(0), 3);

    e.request_pickup(5, "UP");
    e.step();
    e.step();

    assert_eq!(e.floor_of(0), 5);
    assert!(e.doors_open(0));
    assert_eq!(e.floor_of(1), 0);
}

#[test]
fn should_prefer_nearby_idle_elevator_over_one_that_already_passed_the_floor() {
    let mut e = sut!(create(12, 2));
    e.request_destination(0, 8);
    for _ in 0..3 {
        e.step();
    }

    e.request_pickup(2, "UP");
    e.step();
    e.step();

    assert_eq!(e.floor_of(1), 2);
    assert!(e.doors_open(1));
    assert_eq!(e.floor_of(0), 5);
}

#[test]
fn should_eventually_serve_every_request_under_random_load() {
    let floors = 15u64;
    let elevators = 3usize;
    let mut e = sut!(create(floors as u32, elevators as u32));
    let mut rnd = Lcg(42);
    let mut hall_floors = HashSet::new();
    let mut cabin: Vec<HashSet<i32>> = vec![HashSet::new(); elevators];
    let mut served_at: Vec<HashSet<i32>> = vec![HashSet::new(); elevators];
    let mut served_anywhere = HashSet::new();

    for i in 0..540 {
        if i < 40 {
            if rnd.next(2) == 0 {
                let f = rnd.next(floors) as i32;
                let dir = if f == 0 {
                    "UP"
                } else if f == floors as i32 - 1 {
                    "DOWN"
                } else if rnd.next(2) == 0 {
                    "UP"
                } else {
                    "DOWN"
                };
                assert!(e.request_pickup(f, dir));
                hall_floors.insert(f);
            } else {
                let k = rnd.next(elevators as u64) as usize;
                let f = rnd.next(floors) as i32;
                assert!(e.request_destination(k, f));
                cabin[k].insert(f);
            }
        }
        e.step();
        for k in 0..elevators {
            if e.doors_open(k) {
                served_at[k].insert(e.floor_of(k));
                served_anywhere.insert(e.floor_of(k));
            }
        }
    }

    assert!(hall_floors.is_subset(&served_anywhere));
    for k in 0..elevators {
        assert!(cabin[k].is_subset(&served_at[k]));
    }
}
