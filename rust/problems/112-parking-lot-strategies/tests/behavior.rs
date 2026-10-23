#![allow(unused_imports, clippy::needless_range_loop)]
use p112_parking_lot_strategies::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::collections::HashSet;
use std::sync::Arc;

const TWO: &str = "TWO_WHEELER";
const FOUR: &str = "FOUR_WHEELER";
const LOWEST: &str = "LOWEST_INDEX_SPOT";
const MOST_FREE: &str = "MOST_FREE_FLOOR";

fn make(two: &[usize], four: &[usize], strategy: &str) -> Option<Arc<dyn ParkingLot>> {
    create(two, four, strategy).map(|r| r.expect("valid configuration must be accepted"))
}

/// Standard lot: floor 0 has 2 two-wheeler + 1 four-wheeler spots (global 0,1 | 2),
/// floor 1 has 1 two-wheeler + 2 four-wheeler spots (global 3 | 4,5).
fn lot(strategy: &str) -> Option<Arc<dyn ParkingLot>> {
    make(&[2, 1], &[1, 2], strategy)
}

fn park(l: &Arc<dyn ParkingLot>, t: &str, reg: &str) -> String {
    l.park(t, reg)
        .unwrap()
        .unwrap_or_else(|| panic!("expected a spot for {reg}"))
}

fn at(l: &Arc<dyn ParkingLot>, ticket: &str) -> String {
    l.find_by_ticket(ticket).unwrap().expect("active ticket")
}

#[test]
fn should_park_two_wheelers_in_the_lowest_indexed_spots_first() {
    let l = sut!(lot(LOWEST));
    assert_eq!(at(&l, &park(&l, TWO, "A")), "0:0");
    assert_eq!(at(&l, &park(&l, TWO, "B")), "0:1");
    assert_eq!(at(&l, &park(&l, TWO, "C")), "1:3");
}

#[test]
fn should_park_four_wheelers_in_the_lowest_indexed_four_wheeler_spots_first() {
    let l = sut!(lot(LOWEST));
    assert_eq!(at(&l, &park(&l, FOUR, "A")), "0:2");
    assert_eq!(at(&l, &park(&l, FOUR, "B")), "1:4");
    assert_eq!(at(&l, &park(&l, FOUR, "C")), "1:5");
}

#[test]
fn should_never_put_a_vehicle_in_a_spot_of_the_other_type() {
    let l = sut!(lot(LOWEST));
    park(&l, TWO, "A");
    park(&l, TWO, "B");
    park(&l, TWO, "C");

    assert_eq!(l.park(TWO, "D"), Ok(None));
    assert_eq!(l.free_spots(FOUR), Ok(3));
    assert!(l.park(FOUR, "E").unwrap().is_some());
}

#[test]
fn should_return_none_when_the_lot_is_full_and_change_nothing() {
    let l = sut!(lot(LOWEST));
    for i in 0..3 {
        park(&l, FOUR, &format!("F{i}"));
    }

    assert_eq!(l.park(FOUR, "overflow"), Ok(None));
    assert_eq!(l.find_by_registration("overflow"), Ok(None));
    assert_eq!(l.free_spots(FOUR), Ok(0));
    assert_eq!(l.free_spots(TWO), Ok(3));
}

#[test]
fn should_allow_a_vehicle_that_was_turned_away_to_park_once_a_spot_is_free() {
    let l = sut!(lot(LOWEST));
    let t1 = park(&l, FOUR, "F1");
    park(&l, FOUR, "F2");
    park(&l, FOUR, "F3");
    assert_eq!(l.park(FOUR, "late"), Ok(None));

    l.unpark(&t1).unwrap();

    assert!(l.park(FOUR, "late").unwrap().is_some());
}

#[test]
fn should_reject_the_same_registration_while_it_is_parked() {
    let l = sut!(lot(LOWEST));
    park(&l, TWO, "KA-01");

    assert_eq!(l.park(TWO, "KA-01"), Err(ParkingError::AlreadyParked));
    assert_eq!(l.park(FOUR, "KA-01"), Err(ParkingError::AlreadyParked));
    assert_eq!(l.free_spots(TWO), Ok(2));
    assert_eq!(l.free_spots(FOUR), Ok(3));
}

#[test]
fn should_reject_a_duplicate_registration_even_when_the_lot_is_full() {
    let l = sut!(lot(LOWEST));
    park(&l, FOUR, "F1");
    park(&l, FOUR, "F2");
    park(&l, FOUR, "F3");

    assert_eq!(l.park(FOUR, "F1"), Err(ParkingError::AlreadyParked));
}

#[test]
fn should_accept_the_same_registration_again_after_it_left() {
    let l = sut!(lot(LOWEST));
    let first = park(&l, TWO, "KA-01");
    assert_eq!(l.unpark(&first), Ok(true));

    let second = park(&l, TWO, "KA-01");

    assert_ne!(first, second);
    assert_eq!(l.find_by_registration("KA-01"), Ok(Some("0:0".to_string())));
}

#[test]
fn should_treat_registration_numbers_as_exact_case_sensitive_text() {
    let l = sut!(lot(LOWEST));
    park(&l, TWO, "ka-01");

    assert!(l.park(TWO, "KA-01").unwrap().is_some());
    assert_eq!(l.find_by_registration("Ka-01"), Ok(None));
}

#[test]
fn should_unpark_a_ticket_only_once() {
    let l = sut!(lot(LOWEST));
    let t = park(&l, TWO, "A");

    assert_eq!(l.unpark(&t), Ok(true));
    assert_eq!(l.unpark(&t), Ok(false));
    assert_eq!(l.free_spots(TWO), Ok(3));
}

#[test]
fn should_return_false_for_an_unknown_ticket() {
    let l = sut!(lot(LOWEST));
    park(&l, TWO, "A");

    assert_eq!(l.unpark("no-such-ticket"), Ok(false));
    assert_eq!(l.free_spots(TWO), Ok(2));
}

#[test]
fn should_hand_out_the_freed_spot_to_the_next_vehicle() {
    let l = sut!(lot(LOWEST));
    let a = park(&l, TWO, "A");
    park(&l, TWO, "B");
    park(&l, TWO, "C");
    l.unpark(&a).unwrap();

    assert_eq!(at(&l, &park(&l, TWO, "D")), "0:0");
}

#[test]
fn should_find_vehicles_by_registration_and_by_ticket() {
    let l = sut!(lot(LOWEST));
    let t = park(&l, FOUR, "KA-09");

    assert_eq!(l.find_by_registration("KA-09"), Ok(Some("0:2".to_string())));
    assert_eq!(l.find_by_ticket(&t), Ok(Some("0:2".to_string())));
    assert_eq!(l.find_by_registration("other"), Ok(None));
    assert_eq!(l.find_by_ticket("other"), Ok(None));
}

#[test]
fn should_not_find_a_vehicle_after_it_was_unparked() {
    let l = sut!(lot(LOWEST));
    let t = park(&l, FOUR, "KA-09");
    l.unpark(&t).unwrap();

    assert_eq!(l.find_by_registration("KA-09"), Ok(None));
    assert_eq!(l.find_by_ticket(&t), Ok(None));
}

#[test]
fn should_report_free_spots_in_total_and_per_floor() {
    let l = sut!(lot(LOWEST));
    assert_eq!(l.free_spots(TWO), Ok(3));
    assert_eq!(l.free_spots(FOUR), Ok(3));
    assert_eq!(l.free_spots_on_floor(0, TWO), Ok(2));
    assert_eq!(l.free_spots_on_floor(0, FOUR), Ok(1));
    assert_eq!(l.free_spots_on_floor(1, TWO), Ok(1));
    assert_eq!(l.free_spots_on_floor(1, FOUR), Ok(2));

    let a = park(&l, TWO, "A"); // 0:0
    park(&l, FOUR, "B"); // 0:2
    park(&l, FOUR, "C"); // 1:4

    assert_eq!(l.free_spots(TWO), Ok(2));
    assert_eq!(l.free_spots(FOUR), Ok(1));
    assert_eq!(l.free_spots_on_floor(0, TWO), Ok(1));
    assert_eq!(l.free_spots_on_floor(0, FOUR), Ok(0));
    assert_eq!(l.free_spots_on_floor(1, FOUR), Ok(1));

    l.unpark(&a).unwrap();
    assert_eq!(l.free_spots_on_floor(0, TWO), Ok(2));
    assert_eq!(l.free_spots(TWO), Ok(3));
}

#[test]
fn should_number_spots_globally_floor_by_floor_with_two_wheelers_first_on_each_floor() {
    let l = sut!(make(&[0, 3, 1], &[2, 0, 2], LOWEST));

    assert_eq!(at(&l, &park(&l, FOUR, "a")), "0:0");
    assert_eq!(at(&l, &park(&l, FOUR, "b")), "0:1");
    assert_eq!(at(&l, &park(&l, TWO, "c")), "1:2");
    assert_eq!(at(&l, &park(&l, FOUR, "d")), "2:6");
    assert_eq!(at(&l, &park(&l, TWO, "e")), "1:3");
    assert_eq!(at(&l, &park(&l, TWO, "f")), "1:4");
    assert_eq!(at(&l, &park(&l, TWO, "g")), "2:5");
    assert_eq!(at(&l, &park(&l, FOUR, "h")), "2:7");
}

#[test]
fn should_prefer_the_floor_with_the_most_free_spots_for_the_vehicle_type() {
    let l = sut!(lot(MOST_FREE));

    assert_eq!(at(&l, &park(&l, FOUR, "A")), "1:4"); // floor 1 has 2 free, floor 0 has 1
    assert_eq!(at(&l, &park(&l, FOUR, "B")), "0:2"); // tie 1-1: lowest floor
    assert_eq!(at(&l, &park(&l, FOUR, "C")), "1:5");
    assert_eq!(l.park(FOUR, "D"), Ok(None));
}

#[test]
fn should_break_floor_ties_by_lowest_floor_then_lowest_spot() {
    let l = sut!(make(&[2, 2, 2], &[0, 0, 0], MOST_FREE));

    assert_eq!(at(&l, &park(&l, TWO, "a")), "0:0");
    assert_eq!(at(&l, &park(&l, TWO, "b")), "1:2");
    assert_eq!(at(&l, &park(&l, TWO, "c")), "2:4");
    assert_eq!(at(&l, &park(&l, TWO, "d")), "0:1");
    assert_eq!(at(&l, &park(&l, TWO, "e")), "1:3");
    assert_eq!(at(&l, &park(&l, TWO, "f")), "2:5");
}

#[test]
fn should_count_only_free_spots_of_the_vehicle_type_when_choosing_the_floor() {
    // floor 0: 1 two + 5 four, floor 1: 3 two + 1 four
    let l = sut!(make(&[1, 3], &[5, 1], MOST_FREE));

    assert_eq!(at(&l, &park(&l, TWO, "t")), "1:6"); // floor 1 has most free two-wheeler spots
    assert_eq!(at(&l, &park(&l, FOUR, "f")), "0:1"); // floor 0 has most free four-wheeler spots
}

#[test]
fn should_reuse_a_freed_spot_under_most_free_floor_when_its_floor_is_the_emptiest() {
    let l = sut!(lot(MOST_FREE));
    let a = park(&l, TWO, "A"); // floor 0 (2 free vs 1) -> 0:0
    park(&l, TWO, "B"); // tie 1-1 -> 0:1
    park(&l, TWO, "C"); // floor 1 -> 1:3
    l.unpark(&a).unwrap(); // floor 0 has 1 free, floor 1 has 0

    assert_eq!(at(&l, &park(&l, TWO, "D")), "0:0");
}

#[test]
fn should_apply_a_new_strategy_only_to_later_parks_and_leave_parked_vehicles_alone() {
    let l = sut!(lot(LOWEST));
    let a = park(&l, FOUR, "A"); // 0:2
    assert_eq!(at(&l, &a), "0:2");

    l.set_strategy(MOST_FREE).unwrap();

    assert_eq!(at(&l, &a), "0:2");
    assert_eq!(at(&l, &park(&l, FOUR, "B")), "1:4"); // floor 0 has 0 free, floor 1 has 2
    l.set_strategy(LOWEST).unwrap();
    assert_eq!(at(&l, &park(&l, FOUR, "C")), "1:5");
}

#[test]
fn should_give_every_ticket_a_unique_id_that_is_never_reused() {
    let l = sut!(lot(LOWEST));
    let mut seen = HashSet::new();
    for i in 0..200 {
        let t = park(&l, TWO, &format!("R{i}"));
        assert!(!t.trim().is_empty());
        assert!(seen.insert(t.clone()), "ticket {t} reused");
        assert_eq!(l.unpark(&t), Ok(true));
    }
}

#[test]
fn should_not_let_an_old_ticket_release_the_spot_of_a_later_vehicle() {
    let l = sut!(lot(LOWEST));
    let old = park(&l, TWO, "A");
    l.unpark(&old).unwrap();
    let fresh = park(&l, TWO, "B"); // takes the same spot

    assert_eq!(l.unpark(&old), Ok(false));
    assert_eq!(at(&l, &fresh), "0:0");
    assert_eq!(l.find_by_registration("B"), Ok(Some("0:0".to_string())));
}

#[test]
fn should_reject_invalid_input() {
    let l = sut!(lot(LOWEST));
    park(&l, TWO, "A");
    let bad = ParkingError::InvalidArgument;

    assert_eq!(l.park("TRUCK", "X").unwrap_err(), bad);
    assert_eq!(l.park(TWO, "").unwrap_err(), bad);
    assert_eq!(l.park(TWO, "  ").unwrap_err(), bad);
    assert_eq!(l.unpark(" ").unwrap_err(), bad);
    assert_eq!(l.find_by_registration("").unwrap_err(), bad);
    assert_eq!(l.find_by_ticket("").unwrap_err(), bad);
    assert_eq!(l.free_spots("BUS").unwrap_err(), bad);
    assert_eq!(l.free_spots_on_floor(2, TWO).unwrap_err(), bad);
    assert_eq!(l.free_spots_on_floor(0, "BUS").unwrap_err(), bad);
    assert_eq!(l.set_strategy("RANDOM").unwrap_err(), bad);
    assert_eq!(l.free_spots(TWO), Ok(2)); // nothing above changed the state
}

#[test]
fn should_keep_the_current_strategy_when_an_unknown_one_is_rejected() {
    let l = sut!(lot(MOST_FREE));

    assert_eq!(l.set_strategy("nope"), Err(ParkingError::InvalidArgument));

    assert_eq!(at(&l, &park(&l, FOUR, "A")), "1:4"); // still most-free-floor
}

#[test]
fn should_reject_invalid_lot_configuration() {
    sut!(lot(LOWEST)); // skips while unimplemented
    let bad = |two: &[usize], four: &[usize], s: &str| match create(two, four, s) {
        Some(Err(e)) => e == ParkingError::InvalidArgument,
        _ => false,
    };
    assert!(bad(&[1], &[1, 1], LOWEST));
    assert!(bad(&[], &[], LOWEST));
    assert!(bad(&[1], &[1], "RANDOM"));
}

#[test]
fn should_support_a_lot_with_no_spots_of_one_type() {
    let l = sut!(make(&[0, 0], &[1, 1], LOWEST));

    assert_eq!(l.free_spots(TWO), Ok(0));
    assert_eq!(l.park(TWO, "bike"), Ok(None));
    assert_eq!(l.free_spots(FOUR), Ok(2));
}

#[test]
fn should_never_double_book_a_spot_when_many_threads_park_at_once() {
    for strategy in [LOWEST, MOST_FREE] {
        let l = sut!(make(&[10, 10, 10], &[0, 0, 0], strategy));

        let results = collect_concurrently(64, |t| l.park(TWO, &format!("V{t}")).unwrap());

        let issued: Vec<String> = results.into_iter().flatten().collect();
        let tickets: HashSet<&String> = issued.iter().collect();
        let spots: HashSet<String> = issued.iter().map(|id| at(&l, id)).collect();
        assert_eq!(issued.len(), 30);
        assert_eq!(tickets.len(), 30);
        assert_eq!(spots.len(), 30);
        assert_eq!(l.free_spots(TWO), Ok(0));
    }
}

#[test]
fn should_admit_exactly_one_of_many_threads_parking_the_same_registration() {
    let l = sut!(lot(LOWEST));

    let parked = collect_concurrently(16, |_| matches!(l.park(TWO, "SAME"), Ok(Some(_))));

    assert_eq!(parked.iter().filter(|b| **b).count(), 1);
    assert_eq!(l.free_spots(TWO), Ok(2));
}

#[test]
fn should_keep_counts_consistent_while_threads_park_and_leave() {
    let l = sut!(lot(MOST_FREE));

    run_concurrently(8, |t| {
        for i in 0..500 {
            let ty = if i % 2 == 0 { TWO } else { FOUR };
            if let Some(id) = l.park(ty, &format!("t{t}-{i}")).unwrap() {
                assert_eq!(l.unpark(&id), Ok(true));
            }
        }
    });

    assert_eq!(l.free_spots(TWO), Ok(3));
    assert_eq!(l.free_spots(FOUR), Ok(3));
    assert_eq!(
        l.free_spots_on_floor(0, TWO).unwrap() + l.free_spots_on_floor(1, TWO).unwrap(),
        3
    );
}
