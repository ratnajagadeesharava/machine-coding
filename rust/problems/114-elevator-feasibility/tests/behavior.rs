#![allow(unused_imports, clippy::needless_range_loop)]
use p114_elevator_feasibility::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// 10 floors (0..=9), capacity 4.
fn lift(max_stops: u32) -> Option<Arc<dyn LiftController>> {
    create(10, 4, max_stops)
}

fn drain(l: &Arc<dyn LiftController>) -> Vec<u32> {
    let mut stops = vec![];
    while let Some(s) = l.step() {
        stops.push(s);
    }
    stops
}

#[test]
fn should_start_idle_at_the_ground_floor() {
    let l = sut!(lift(2));

    assert_eq!(l.current_floor(), 0);
    assert_eq!(l.onboard(), 0);
    assert!(l.plan().is_empty());
    assert_eq!(l.step(), None);
    assert_eq!(l.current_floor(), 0);
}

#[test]
fn should_accept_a_single_request_and_plan_its_pickup_and_drop_off() {
    let l = sut!(lift(0));

    assert_eq!(l.request("r1", 0, 5, 1), Ok(true));

    assert_eq!(l.plan(), vec![0, 5]);
}

#[test]
fn should_serve_a_downward_ride_on_the_downward_sweep() {
    let l = sut!(lift(0));
    assert_eq!(l.request("r1", 6, 2, 1), Ok(true));

    assert_eq!(l.plan(), vec![6, 2]);
    assert_eq!(l.step(), Some(6));
    assert_eq!(l.current_floor(), 6);
    assert_eq!(l.onboard(), 1);
    assert_eq!(l.step(), Some(2));
    assert_eq!(l.current_floor(), 2);
    assert_eq!(l.onboard(), 0);
    assert_eq!(l.step(), None);
}

#[test]
fn should_report_the_lifecycle_of_a_request() {
    let l = sut!(lift(0));
    assert_eq!(l.status("r1"), "UNKNOWN");

    l.request("r1", 0, 5, 2).unwrap();
    assert_eq!(l.status("r1"), "WAITING");
    l.step();
    assert_eq!(l.status("r1"), "ONBOARD");
    assert_eq!(l.onboard(), 2);
    l.step();
    assert_eq!(l.status("r1"), "DELIVERED");
    assert_eq!(l.onboard(), 0);
}

#[test]
fn should_return_exactly_the_planned_stops_when_stepping() {
    let l = sut!(lift(5));
    l.request("a", 0, 7, 1).unwrap();
    l.request("b", 3, 5, 1).unwrap();
    l.request("c", 8, 1, 2).unwrap();
    let planned = l.plan();

    assert!(!planned.is_empty());
    assert_eq!(drain(&l), planned);
    assert!(l.plan().is_empty());
}

#[test]
fn should_merge_riders_with_the_same_route_into_the_same_stops() {
    let l = sut!(lift(0));

    assert_eq!(l.request("r1", 0, 5, 1), Ok(true));
    assert_eq!(l.request("r2", 0, 5, 1), Ok(true));
    assert_eq!(l.request("r3", 0, 5, 1), Ok(true));

    assert_eq!(l.plan(), vec![0, 5]);
}

#[test]
fn should_reject_a_request_whose_stops_break_an_existing_riders_limit() {
    let l = sut!(lift(1));
    assert_eq!(l.request("r1", 0, 9, 1), Ok(true));

    // 3 -> 5 would add two stops (3 and 5) between r1's pickup and drop-off
    assert_eq!(l.request("r2", 3, 5, 1), Ok(false));
    // 0 -> 5 adds only one (5)
    assert_eq!(l.request("r3", 0, 5, 1), Ok(true));
    assert_eq!(l.plan(), vec![0, 5, 9]);
    // 0 -> 3 would now give r1 two intermediate stops (3 and 5)
    assert_eq!(l.request("r4", 0, 3, 1), Ok(false));
}

#[test]
fn should_reject_a_request_that_would_break_its_own_limit() {
    let l = sut!(lift(1));
    assert_eq!(l.request("r1", 0, 3, 1), Ok(true));
    assert_eq!(l.request("r2", 0, 5, 1), Ok(true));

    // r3 would pass stops 3 and 5 on the way to 9 (two intermediate stops); nobody else is hurt
    assert_eq!(l.request("r3", 0, 9, 1), Ok(false));
    assert_eq!(l.plan(), vec![0, 3, 5]);
}

#[test]
fn should_allow_only_non_stop_rides_when_the_limit_is_zero() {
    let l = sut!(lift(0));
    assert_eq!(l.request("r1", 0, 5, 1), Ok(true));

    assert_eq!(l.request("r2", 0, 7, 1), Ok(false)); // would stop at 5 on the way
    assert_eq!(l.request("r3", 5, 8, 1), Ok(true)); // starts exactly where r1 ends
    assert_eq!(l.plan(), vec![0, 5, 8]);
}

#[test]
fn should_reject_when_capacity_is_exceeded_at_pickup() {
    let l = sut!(lift(4));
    assert_eq!(l.request("r1", 0, 5, 3), Ok(true));

    assert_eq!(l.request("r2", 0, 5, 2), Ok(false));
    assert_eq!(l.request("r3", 0, 5, 1), Ok(true));
    assert_eq!(l.request("r4", 0, 5, 1), Ok(false));
}

#[test]
fn should_reject_when_routes_overlap_beyond_capacity_mid_route() {
    let l = sut!(create(10, 4, 5));
    assert_eq!(l.request("r1", 0, 8, 3), Ok(true));

    assert_eq!(l.request("r2", 3, 6, 2), Ok(false)); // 3 + 2 persons between floors 3 and 6
    assert_eq!(l.request("r3", 3, 6, 1), Ok(true));
    assert_eq!(l.request("r4", 7, 9, 3), Ok(false)); // r1 is still inside at 7 (3 + 3)
    assert_eq!(l.request("r5", 8, 9, 4), Ok(true)); // r1 leaves at 8 before r5 boards
}

#[test]
fn should_let_riders_leave_before_others_board_at_the_same_stop() {
    let l = sut!(lift(0));
    assert_eq!(l.request("r1", 0, 5, 4), Ok(true));

    assert_eq!(l.request("r2", 5, 8, 4), Ok(true));

    assert_eq!(l.plan(), vec![0, 5, 8]);
}

#[test]
fn should_reject_a_request_larger_than_the_capacity() {
    let l = sut!(create(10, 4, 5));

    assert_eq!(l.request("big", 0, 5, 5), Ok(false));
    assert!(l.plan().is_empty());
    assert_eq!(l.status("big"), "UNKNOWN");
}

#[test]
fn should_leave_everything_unchanged_when_a_request_is_rejected() {
    let l = sut!(lift(0));
    l.request("r1", 0, 5, 1).unwrap();
    l.step();
    let plan_before = l.plan();

    assert_eq!(l.request("r2", 2, 8, 1), Ok(false));

    assert_eq!(l.plan(), plan_before);
    assert_eq!(l.status("r2"), "UNKNOWN");
    assert_eq!(l.status("r1"), "ONBOARD");
    assert_eq!(l.onboard(), 1);
    assert_eq!(l.current_floor(), 0);
    // the id was never taken, so it can be used again for a request that fits
    assert_eq!(l.request("r2", 5, 8, 1), Ok(true));
}

#[test]
fn should_count_stops_already_made_against_riders_that_are_onboard() {
    let fresh = sut!(lift(1));
    fresh.request("r1", 0, 9, 1).unwrap();
    assert_eq!(fresh.request("r3", 6, 9, 1), Ok(true)); // only one stop (6) in between

    let l = sut!(lift(1));
    l.request("r1", 0, 9, 1).unwrap();
    l.request("r2", 0, 4, 1).unwrap();
    assert_eq!(l.step(), Some(0));
    assert_eq!(l.step(), Some(4)); // r1 has already seen one intermediate stop

    assert_eq!(l.request("r3", 6, 9, 1), Ok(false));
    assert_eq!(l.plan(), vec![9]);
}

#[test]
fn should_pick_up_an_upward_rider_on_the_next_upward_sweep() {
    let l = sut!(lift(5));
    l.request("down", 6, 2, 1).unwrap();
    assert_eq!(l.step(), Some(6)); // lift is now at 6 and heading down

    assert_eq!(l.request("up", 3, 5, 1), Ok(true));

    // continues down to 2, turns at the bottom and picks "up" up at 3 on the way back
    assert_eq!(l.plan(), vec![2, 3, 5]);
}

#[test]
fn should_carry_a_downward_rider_that_starts_above_the_lift_after_the_upward_sweep() {
    let l = sut!(lift(5));
    l.request("up", 0, 3, 1).unwrap();

    assert_eq!(l.request("down", 8, 4, 1), Ok(true));

    assert_eq!(l.plan(), vec![0, 3, 8, 4]);
}

#[test]
fn should_serve_riders_at_the_top_and_bottom_floors() {
    let l = sut!(lift(0));

    assert_eq!(l.request("r1", 0, 9, 1), Ok(true));
    assert_eq!(l.request("r2", 9, 0, 1), Ok(true));

    assert_eq!(l.plan(), vec![0, 9, 0]);
}

#[test]
fn should_board_at_the_current_floor_when_the_direction_matches() {
    let l = sut!(lift(2));
    l.request("r1", 0, 5, 1).unwrap();
    l.step();
    l.step();
    assert_eq!(l.current_floor(), 5);

    assert_eq!(l.request("r2", 5, 8, 1), Ok(true));
    assert_eq!(l.request("r3", 5, 2, 1), Ok(true));

    assert_eq!(l.plan(), vec![5, 8, 5, 2]);
}

#[test]
fn should_reject_invalid_input() {
    let l = sut!(lift(2));

    assert_eq!(l.request("r", 0, 10, 1), Err(LiftError::InvalidArgument));
    assert_eq!(l.request("r", 10, 3, 1), Err(LiftError::InvalidArgument));
    assert_eq!(l.request("r", 4, 4, 1), Err(LiftError::InvalidArgument));
    assert_eq!(l.request("r", 0, 3, 0), Err(LiftError::InvalidArgument));
    assert_eq!(l.request(" ", 0, 3, 1), Err(LiftError::InvalidArgument));
    assert_eq!(l.request("", 0, 3, 1), Err(LiftError::InvalidArgument));
    assert!(l.plan().is_empty());
}

#[test]
fn should_reject_a_duplicate_id_of_an_active_request() {
    let l = sut!(lift(2));
    l.request("r1", 0, 3, 1).unwrap();

    assert_eq!(l.request("r1", 1, 4, 1), Err(LiftError::InvalidArgument));
    assert_eq!(l.plan(), vec![0, 3]);
}

#[test]
fn should_accept_exactly_capacity_worth_of_identical_requests_from_many_threads() {
    let l = sut!(create(10, 4, 0));
    let accepted = AtomicUsize::new(0);

    run_concurrently(8, |t| {
        for i in 0..5 {
            if l.request(&format!("r-{t}-{i}"), 0, 5, 1).unwrap() {
                accepted.fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    assert_eq!(accepted.load(Ordering::SeqCst), 4);
    assert_eq!(l.plan(), vec![0, 5]);
    assert_eq!(drain(&l), vec![0, 5]);
    assert_eq!(l.onboard(), 0);
}

#[test]
fn should_lose_no_request_when_many_threads_request_at_once() {
    let l = sut!(create(50, 1000, 1000));

    run_concurrently(8, |t| {
        for i in 0..20u32 {
            assert_eq!(
                l.request(&format!("r-{t}-{i}"), t as u32, t as u32 + 20, 1),
                Ok(true)
            );
        }
    });

    drain(&l);
    for t in 0..8 {
        for i in 0..20 {
            assert_eq!(l.status(&format!("r-{t}-{i}")), "DELIVERED");
        }
    }
    assert_eq!(l.onboard(), 0);
}
