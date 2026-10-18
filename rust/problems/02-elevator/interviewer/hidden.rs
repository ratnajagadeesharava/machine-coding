#![allow(unused_imports, clippy::needless_range_loop)]
use p02_elevator::{contract::*, create};
use playground_support::sut;
use std::collections::HashSet;

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

#[test]
fn should_reverse_after_finishing_upward_travel() {
    let mut e = sut!(create(20, 1));
    e.request_destination(0, 10);
    assert_eq!(served(e.as_mut(), 0, 10), vec![10]);

    e.request_destination(0, 2);
    assert_eq!(served(e.as_mut(), 0, 10), vec![2]);
    assert_eq!(e.floor_of(0), 2);
}

#[test]
fn should_handle_many_elevators_and_requests() {
    let mut e = sut!(create(40, 5));
    let mut hall = HashSet::new();
    let mut served_anywhere = HashSet::new();
    let mut seed = 7u64;
    for _ in 0..100 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let f = ((seed >> 33) % 39) as i32;
        e.request_pickup(f, "UP");
        hall.insert(f);
        e.step();
        for k in 0..5 {
            if e.doors_open(k) {
                served_anywhere.insert(e.floor_of(k));
            }
        }
    }
    for _ in 0..2000 {
        e.step();
        for k in 0..5 {
            if e.doors_open(k) {
                served_anywhere.insert(e.floor_of(k));
            }
        }
    }
    assert!(hall.is_subset(&served_anywhere));
}

#[test]
fn should_serve_downward_hall_call_from_above_then_continue() {
    let mut e = sut!(create(10, 1));
    e.request_pickup(6, "DOWN");
    e.request_destination(0, 8);
    assert_eq!(served(e.as_mut(), 0, 20), vec![8, 6]);
}
