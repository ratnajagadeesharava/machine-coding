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

/// Counts how often each floor opened its doors, summed over all elevators.
fn openings_per_floor(e: &mut dyn ElevatorApi, floors: usize, elevators: usize, steps: usize) -> Vec<usize> {
    let mut counts = vec![0; floors];
    for _ in 0..steps {
        e.step();
        for k in 0..elevators {
            if e.doors_open(k) {
                counts[e.floor_of(k) as usize] += 1;
            }
        }
    }
    counts
}

fn random_direction(rnd: &mut Lcg, f: i32, floors: i32) -> &'static str {
    if f == 0 {
        "UP"
    } else if f == floors - 1 {
        "DOWN"
    } else if rnd.next(2) == 0 {
        "UP"
    } else {
        "DOWN"
    }
}

#[test]
fn should_serve_an_opposite_direction_pickup_from_an_idle_elevator() {
    let mut e = sut!(create(10, 1));
    assert!(e.request_pickup(3, "DOWN"));

    assert_eq!(served(e.as_mut(), 0, 10), vec![3]);
}

#[test]
fn should_stay_on_the_last_served_floor_when_idle() {
    let mut e = sut!(create(10, 1));
    e.request_destination(0, 5);
    assert_eq!(served(e.as_mut(), 0, 8), vec![5]);

    for _ in 0..20 {
        e.step();
        assert_eq!(e.floor_of(0), 5);
        assert!(!e.doors_open(0));
    }
}

#[test]
fn should_serve_behind_stops_in_descending_order_after_reversing() {
    let mut e = sut!(create(10, 1));
    e.request_destination(0, 5);
    assert_eq!(served(e.as_mut(), 0, 8), vec![5]);

    e.request_destination(0, 1);
    e.request_destination(0, 4);
    e.request_destination(0, 3);

    assert_eq!(served(e.as_mut(), 0, 12), vec![4, 3, 1]);
}

#[test]
fn should_serve_cabin_requests_and_matching_hall_calls_on_the_way_down_and_turn_around_for_the_rest() {
    let mut e = sut!(create(10, 1));
    e.request_destination(0, 8);
    assert_eq!(served(e.as_mut(), 0, 10), vec![8]);

    e.request_pickup(5, "DOWN");
    e.request_pickup(3, "UP");
    e.request_destination(0, 1);

    assert_eq!(served(e.as_mut(), 0, 20), vec![5, 1, 3]);
}

#[test]
fn should_keep_earlier_promises_when_requests_keep_arriving_mid_journey() {
    let mut e = sut!(create(10, 1));
    e.request_destination(0, 9);
    for _ in 0..4 {
        e.step();
    }
    assert_eq!(e.floor_of(0), 4);

    assert!(e.request_destination(0, 2));
    assert!(e.request_pickup(6, "DOWN"));

    assert_eq!(served(e.as_mut(), 0, 30), vec![9, 6, 2]);
}

#[test]
fn should_serve_the_same_floor_again_when_it_is_requested_again_after_being_served() {
    let mut e = sut!(create(10, 1));
    e.request_destination(0, 3);
    assert_eq!(served(e.as_mut(), 0, 6), vec![3]);

    e.request_destination(0, 3);
    e.request_pickup(3, "UP");

    assert_eq!(served(e.as_mut(), 0, 4), vec![3]);
    assert_eq!(e.floor_of(0), 3);
}

#[test]
fn should_reopen_doors_without_moving_when_the_current_floor_is_requested_again_once_doors_are_closed() {
    let mut e = sut!(create(10, 1));
    e.request_destination(0, 2);
    served(e.as_mut(), 0, 5);
    assert!(!e.doors_open(0));

    assert!(e.request_pickup(2, "UP"));
    e.step();

    assert_eq!(e.floor_of(0), 2);
    assert!(e.doors_open(0));
}

#[test]
fn should_count_one_stop_per_requested_floor_for_consecutive_floors() {
    let mut e = sut!(create(10, 1));
    e.request_destination(0, 3);
    e.request_destination(0, 4);
    e.request_destination(0, 5);
    e.request_destination(0, 4);

    assert_eq!(served(e.as_mut(), 0, 15), vec![3, 4, 5]);
}

#[test]
fn should_serve_a_cabin_request_only_with_the_elevator_it_was_pressed_in() {
    let mut e = sut!(create(10, 3));
    assert!(e.request_destination(1, 4));
    assert!(e.request_destination(2, 6));

    let mut seen: Vec<Vec<i32>> = vec![Vec::new(); 3];
    for _ in 0..15 {
        e.step();
        for k in 0..3 {
            if e.doors_open(k) {
                seen[k].push(e.floor_of(k));
            }
        }
    }

    assert!(seen[0].is_empty());
    assert_eq!(e.floor_of(0), 0);
    assert_eq!(seen[1], vec![4]);
    assert_eq!(seen[2], vec![6]);
}

#[test]
fn should_answer_every_overlapping_hall_call_exactly_once_across_the_bank() {
    let mut e = sut!(create(12, 2));
    e.request_pickup(3, "UP");
    e.request_pickup(5, "UP");
    e.request_pickup(7, "UP");
    e.request_pickup(5, "UP");
    e.request_pickup(9, "DOWN");

    let counts = openings_per_floor(e.as_mut(), 12, 2, 40);

    for f in 0..12 {
        let requested = f == 3 || f == 5 || f == 7 || f == 9;
        assert_eq!(counts[f], if requested { 1 } else { 0 }, "openings at floor {f}");
    }
}

#[test]
fn should_not_send_a_second_elevator_for_a_duplicate_hall_call_made_while_one_is_on_the_way() {
    let mut e = sut!(create(12, 3));
    e.request_pickup(8, "UP");
    e.step();
    e.step();
    assert!(e.request_pickup(8, "UP"));

    let counts = openings_per_floor(e.as_mut(), 12, 3, 30);

    assert_eq!(counts[8], 1);
    let at_zero = (0..3).filter(|&k| e.floor_of(k) == 0).count();
    assert_eq!(at_zero, 2);
}

#[test]
fn should_accept_the_boundary_floors_and_reject_just_outside_them() {
    let mut e = sut!(create(10, 2));

    assert!(e.request_pickup(0, "UP"));
    assert!(e.request_pickup(9, "DOWN"));
    assert!(e.request_destination(1, 9));
    assert!(e.request_destination(1, 0));
    assert!(!e.request_destination(1, 10));
    assert!(!e.request_destination(1, -1));
    assert!(!e.request_destination(0, 10));
    assert!(e.request_pickup(0, "UP"));
}

#[test]
fn should_compare_directions_case_sensitively() {
    let mut e = sut!(create(10, 1));

    assert!(!e.request_pickup(4, "up"));
    assert!(!e.request_pickup(4, "Down"));
    assert!(!e.request_pickup(4, "UP "));
    assert!(!e.request_pickup(4, ""));

    assert!(served(e.as_mut(), 0, 10).is_empty());
    assert_eq!(e.floor_of(0), 0);
}

#[test]
fn should_not_be_disturbed_by_rejected_requests_in_the_middle_of_a_journey() {
    let mut clean = sut!(create(10, 2));
    let mut noisy = sut!(create(10, 2));
    for api in [&mut clean, &mut noisy] {
        api.request_destination(0, 6);
        api.request_pickup(4, "UP");
        api.request_destination(1, 3);
    }

    for i in 0..25 {
        noisy.request_pickup(10, "UP");
        noisy.request_pickup(0, "DOWN");
        noisy.request_pickup(2, "up");
        noisy.request_destination(5, 2);
        noisy.request_destination(0, -3);
        clean.step();
        noisy.step();
        for k in 0..2 {
            assert_eq!(noisy.floor_of(k), clean.floor_of(k), "floor of {k} at step {i}");
            assert_eq!(noisy.doors_open(k), clean.doors_open(k), "doors of {k} at step {i}");
        }
    }
}

#[test]
fn should_not_advance_time_when_reading_state() {
    let mut e = sut!(create(10, 1));
    e.request_destination(0, 4);
    e.step();

    for _ in 0..10 {
        assert_eq!(e.floor_of(0), 1);
        assert!(!e.doors_open(0));
    }
    assert_eq!(served(e.as_mut(), 0, 6), vec![4]);
}

#[test]
fn should_serve_a_far_hall_call_while_new_nearby_requests_keep_arriving() {
    let mut e = sut!(create(15, 1));
    e.request_pickup(14, "DOWN");
    let mut far_served = false;

    let mut i = 0;
    while i < 120 && !far_served {
        e.request_destination(0, i % 3);
        e.step();
        far_served = e.doors_open(0) && e.floor_of(0) == 14;
        i += 1;
    }

    assert!(far_served);
}

#[test]
fn should_keep_every_elevator_in_range_and_move_at_most_one_floor_per_step() {
    let floors = 20;
    let elevators = 4usize;
    let mut e = sut!(create(floors as u32, elevators as u32));
    let mut rnd = Lcg(7);
    let mut previous = vec![0i32; elevators];

    for _ in 0..2000 {
        if rnd.next(3) == 0 {
            let f = rnd.next(floors as u64) as i32;
            let dir = random_direction(&mut rnd, f, floors);
            if rnd.next(2) == 0 {
                e.request_pickup(f, dir);
            } else {
                e.request_destination(rnd.next(elevators as u64) as usize, f);
            }
        }
        e.step();
        for k in 0..elevators {
            let now = e.floor_of(k);
            assert!((0..floors).contains(&now));
            assert!((now - previous[k]).abs() <= 1);
            previous[k] = now;
        }
    }
}

#[test]
fn should_behave_identically_for_identical_request_sequences() {
    let floors = 12;
    let elevators = 3usize;
    let mut a = sut!(create(floors as u32, elevators as u32));
    let mut b = sut!(create(floors as u32, elevators as u32));
    let mut rnd = Lcg(99);

    for _ in 0..600 {
        if rnd.next(4) == 0 {
            let f = rnd.next(floors as u64) as i32;
            let dir = random_direction(&mut rnd, f, floors);
            let hall = rnd.next(2) == 0;
            let k = rnd.next(elevators as u64) as usize;
            if hall {
                assert!(a.request_pickup(f, dir));
                assert!(b.request_pickup(f, dir));
            } else {
                assert!(a.request_destination(k, f));
                assert!(b.request_destination(k, f));
            }
        }
        a.step();
        b.step();
        for k in 0..elevators {
            assert_eq!(b.floor_of(k), a.floor_of(k));
            assert_eq!(b.doors_open(k), a.doors_open(k));
        }
    }
}

#[test]
fn should_simulate_a_hundred_thousand_steps_on_a_large_building_quickly() {
    let floors = 100;
    let elevators = 16usize;
    let mut e = sut!(create(floors as u32, elevators as u32));
    let mut rnd = Lcg(5);
    let start = std::time::Instant::now();

    for i in 0..100_000 {
        if i % 5 == 0 {
            let f = 1 + rnd.next(floors as u64 - 2) as i32;
            if rnd.next(2) == 0 {
                e.request_pickup(f, if rnd.next(2) == 0 { "UP" } else { "DOWN" });
            } else {
                e.request_destination(rnd.next(elevators as u64) as usize, f);
            }
        }
        e.step();
    }

    assert!(start.elapsed() < std::time::Duration::from_secs(20));
}
