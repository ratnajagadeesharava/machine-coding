#![allow(unused_imports, clippy::needless_range_loop)]
use p01_parking_lot::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut, FakeClock};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

const MINUTE: u64 = 60_000;
const HOUR: u64 = 60 * MINUTE;

fn lot(
    clock: &FakeClock,
    floors: u32,
    small: u32,
    medium: u32,
    large: u32,
) -> Option<Arc<dyn ParkingLotApi>> {
    create(floors, small, medium, large, clock.as_clock())
}

#[test]
fn should_park_vehicle_successfully() {
    let clock = FakeClock::new(1_000_000);
    let lot = sut!(lot(&clock, 1, 0, 1, 0));

    let ticket = lot.enter("CAR", "KA-01");

    assert!(ticket.as_deref().is_some_and(|t| !t.is_empty()));
    assert_eq!(lot.available_spots("CAR"), 0);
}

#[test]
fn should_reject_vehicle_when_parking_is_full() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 1, 0, 1, 1));

    assert!(lot.enter("CAR", "A").is_some());
    assert!(lot.enter("CAR", "B").is_some());
    assert!(lot.enter("CAR", "C").is_none());
}

#[test]
fn should_reject_truck_when_no_large_spot_exists() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 2, 3, 3, 0));

    assert!(lot.enter("TRUCK", "T-1").is_none());
    assert_eq!(lot.available_spots("TRUCK"), 0);
}

#[test]
fn should_reject_unknown_vehicle_type() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 1, 1, 1, 1));

    assert!(lot.enter("HOVERCRAFT", "H-1").is_none());
    assert_eq!(lot.available_spots("CAR"), 2);
}

#[test]
fn should_use_smallest_fitting_spot_first() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 1, 1, 1, 0));

    assert!(lot.enter("MOTORCYCLE", "M-1").is_some()); // takes the small spot
    assert_eq!(lot.available_spots("CAR"), 1); // medium untouched
    assert!(lot.enter("CAR", "C-1").is_some());
    assert!(lot.enter("MOTORCYCLE", "M-2").is_none());
}

#[test]
fn should_report_availability_per_vehicle_type() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 1, 1, 1, 1));

    assert_eq!(lot.available_spots("MOTORCYCLE"), 3);
    assert_eq!(lot.available_spots("CAR"), 2);
    assert_eq!(lot.available_spots("TRUCK"), 1);
}

#[test]
fn should_assign_lower_floor_first() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 3, 0, 1, 0));

    let t0 = lot.enter("CAR", "A").unwrap();
    let t1 = lot.enter("CAR", "B").unwrap();
    let t2 = lot.enter("CAR", "C").unwrap();

    assert_eq!(lot.floor_of(&t0), Some(0));
    assert_eq!(lot.floor_of(&t1), Some(1));
    assert_eq!(lot.floor_of(&t2), Some(2));
}

#[test]
fn should_calculate_correct_fee_for_partial_hours() {
    let clock = FakeClock::new(1_000_000);
    let lot = sut!(lot(&clock, 1, 0, 1, 0));
    let ticket = lot.enter("CAR", "A").unwrap();

    clock.advance(90 * MINUTE);

    assert_eq!(lot.exit(&ticket), Some(4000));
}

#[test]
fn should_charge_minimum_one_hour() {
    let clock = FakeClock::new(1_000_000);
    let lot = sut!(lot(&clock, 1, 0, 1, 0));
    let ticket = lot.enter("CAR", "A").unwrap();

    assert_eq!(lot.exit(&ticket), Some(2000));
}

#[test]
fn should_treat_exact_hour_boundary_as_one_hour() {
    let clock = FakeClock::new(1_000_000);
    let lot = sut!(lot(&clock, 1, 0, 2, 0));
    let exact = lot.enter("CAR", "A").unwrap();
    let over = lot.enter("CAR", "B").unwrap();

    clock.advance(HOUR);
    assert_eq!(lot.exit(&exact), Some(2000));
    clock.advance(MINUTE);
    assert_eq!(lot.exit(&over), Some(4000));
}

#[test]
fn should_apply_rate_per_vehicle_type() {
    let clock = FakeClock::new(1_000_000);
    let lot = sut!(lot(&clock, 1, 1, 0, 1));
    let bike = lot.enter("MOTORCYCLE", "M").unwrap();
    let truck = lot.enter("TRUCK", "T").unwrap();

    clock.advance(2 * HOUR);

    assert_eq!(lot.exit(&bike), Some(2000));
    assert_eq!(lot.exit(&truck), Some(10000));
}

#[test]
fn should_release_spot_after_vehicle_exits() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 1, 0, 1, 0));
    let ticket = lot.enter("CAR", "A").unwrap();
    assert!(lot.enter("CAR", "B").is_none());

    lot.exit(&ticket);

    assert_eq!(lot.available_spots("CAR"), 1);
    assert!(lot.enter("CAR", "B").is_some());
}

#[test]
fn should_reject_second_entry_for_same_plate() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 1, 0, 3, 0));

    assert!(lot.enter("CAR", "SAME").is_some());
    assert!(lot.enter("CAR", "SAME").is_none());
    assert_eq!(lot.available_spots("CAR"), 2);
}

#[test]
fn should_reject_exit_for_unknown_or_already_used_ticket() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 1, 0, 1, 0));
    let ticket = lot.enter("CAR", "A").unwrap();

    assert!(lot.exit("no-such-ticket").is_none());
    assert!(lot.exit(&ticket).is_some());
    assert!(lot.exit(&ticket).is_none());
    assert_eq!(lot.available_spots("CAR"), 1); // second exit did not free a phantom spot
}

#[test]
fn should_issue_distinct_tickets() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 1, 0, 5, 0));
    let tickets: HashSet<String> = (0..5)
        .map(|i| lot.enter("CAR", &format!("P{i}")).unwrap())
        .collect();
    assert_eq!(tickets.len(), 5);
}

#[test]
fn should_never_double_allocate_under_concurrent_entry() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 1, 0, 0, 10));

    let results = collect_concurrently(50, |i| lot.enter("TRUCK", &format!("T-{i}")));

    let tickets: Vec<String> = results.into_iter().flatten().collect();
    assert_eq!(tickets.len(), 10);
    assert_eq!(tickets.iter().collect::<HashSet<_>>().len(), 10);
    assert_eq!(lot.available_spots("TRUCK"), 0);
}

#[test]
fn should_keep_capacity_consistent_under_concurrent_entry_and_exit() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 2, 2, 2, 2));

    run_concurrently(12, |t| {
        for i in 0..300 {
            let plate = format!("T{t}-{i}");
            if let Some(ticket) = lot.enter("CAR", &plate) {
                lot.exit(&ticket);
            }
        }
    });

    assert_eq!(lot.available_spots("MOTORCYCLE"), 12);
}

#[test]
fn should_handle_many_operations_quickly() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 100, 30, 40, 30));
    let start = Instant::now();

    for i in 0..100_000 {
        let ticket = lot.enter("CAR", &format!("P{i}")).unwrap();
        lot.exit(&ticket);
    }

    assert!(start.elapsed() < Duration::from_secs(10));
}

#[test]
fn should_report_zero_availability_for_unknown_vehicle_type() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 1, 1, 1, 1));

    assert_eq!(lot.available_spots("HOVERCRAFT"), 0);
    assert_eq!(lot.available_spots(""), 0);
}

#[test]
fn should_treat_vehicle_types_as_exact_upper_case_strings() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 1, 1, 1, 1));

    assert!(lot.enter("car", "A").is_none());
    assert!(lot.enter("Truck", "B").is_none());
    assert_eq!(lot.available_spots("car"), 0);
    assert_eq!(lot.available_spots("CAR"), 2);
}

#[test]
fn should_count_spots_of_all_floors_in_availability() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 3, 2, 3, 4));

    assert_eq!(lot.available_spots("MOTORCYCLE"), 27);
    assert_eq!(lot.available_spots("CAR"), 21);
    assert_eq!(lot.available_spots("TRUCK"), 12);
}

#[test]
fn should_lower_availability_of_every_type_that_could_use_the_taken_spot() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 1, 1, 1, 1));

    lot.enter("CAR", "C-1"); // medium
    assert_eq!(lot.available_spots("MOTORCYCLE"), 2);
    assert_eq!(lot.available_spots("CAR"), 1);
    assert_eq!(lot.available_spots("TRUCK"), 1);

    lot.enter("TRUCK", "T-1"); // large
    assert_eq!(lot.available_spots("MOTORCYCLE"), 1);
    assert_eq!(lot.available_spots("CAR"), 0);
    assert_eq!(lot.available_spots("TRUCK"), 0);
}

#[test]
fn should_raise_availability_of_every_fitting_type_when_a_spot_is_freed() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 1, 1, 1, 1));
    let bike = lot.enter("MOTORCYCLE", "M-1").unwrap(); // small
    let car = lot.enter("CAR", "C-1").unwrap(); // medium
    let truck = lot.enter("TRUCK", "T-1").unwrap(); // large

    lot.exit(&truck);
    assert_eq!(lot.available_spots("TRUCK"), 1);
    assert_eq!(lot.available_spots("CAR"), 1);
    assert_eq!(lot.available_spots("MOTORCYCLE"), 1);

    lot.exit(&car);
    assert_eq!(lot.available_spots("CAR"), 2);
    assert_eq!(lot.available_spots("MOTORCYCLE"), 2);

    lot.exit(&bike);
    assert_eq!(lot.available_spots("MOTORCYCLE"), 3);
}

#[test]
fn should_fill_the_lowest_floor_before_moving_up() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 2, 0, 2, 0));

    let floors: Vec<Option<u32>> = ["A", "B", "C", "D"]
        .iter()
        .map(|p| lot.floor_of(&lot.enter("CAR", p).unwrap()))
        .collect();

    assert_eq!(floors, vec![Some(0), Some(0), Some(1), Some(1)]);
    assert!(lot.enter("CAR", "E").is_none());
}

#[test]
fn should_prefer_smallest_size_even_on_a_higher_floor() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 2, 1, 0, 1));

    let m1 = lot.enter("MOTORCYCLE", "M-1").unwrap(); // small, floor 0
    let m2 = lot.enter("MOTORCYCLE", "M-2").unwrap(); // small, floor 1 (not large, floor 0)
    let m3 = lot.enter("MOTORCYCLE", "M-3").unwrap(); // large, floor 0
    let m4 = lot.enter("MOTORCYCLE", "M-4").unwrap(); // large, floor 1

    assert_eq!(lot.floor_of(&m1), Some(0));
    assert_eq!(lot.floor_of(&m2), Some(1));
    assert_eq!(lot.floor_of(&m3), Some(0));
    assert_eq!(lot.floor_of(&m4), Some(1));
    assert!(lot.enter("MOTORCYCLE", "M-5").is_none());
}

#[test]
fn should_place_cars_in_medium_spots_of_all_floors_before_large_ones() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 2, 0, 1, 1));

    let c1 = lot.enter("CAR", "C-1").unwrap(); // medium
    let c2 = lot.enter("CAR", "C-2").unwrap(); // medium
    let c3 = lot.enter("CAR", "C-3").unwrap(); // large
    let c4 = lot.enter("CAR", "C-4").unwrap(); // large

    assert_eq!(lot.floor_of(&c1), Some(0));
    assert_eq!(lot.floor_of(&c2), Some(1));
    assert_eq!(lot.floor_of(&c3), Some(0));
    assert_eq!(lot.floor_of(&c4), Some(1));
    assert!(lot.enter("CAR", "C-5").is_none());
}

#[test]
fn should_refuse_truck_when_cars_have_taken_every_large_spot() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 1, 0, 1, 1));

    assert!(lot.enter("CAR", "C-1").is_some());
    assert!(lot.enter("CAR", "C-2").is_some()); // spills into the large spot
    assert!(lot.enter("TRUCK", "T-1").is_none());
    assert_eq!(lot.available_spots("TRUCK"), 0);
}

#[test]
fn should_hand_out_the_lowest_free_floor_again_after_spots_are_freed() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 3, 0, 1, 0));
    let a = lot.enter("CAR", "A").unwrap(); // floor 0
    let b = lot.enter("CAR", "B").unwrap(); // floor 1
    let c = lot.enter("CAR", "C").unwrap(); // floor 2

    lot.exit(&b);
    let d = lot.enter("CAR", "D").unwrap();
    assert_eq!(lot.floor_of(&d), Some(1));

    lot.exit(&c);
    lot.exit(&a);
    let e = lot.enter("CAR", "E").unwrap();
    let f = lot.enter("CAR", "F").unwrap();
    assert_eq!(lot.floor_of(&e), Some(0));
    assert_eq!(lot.floor_of(&f), Some(2));
}

#[test]
fn should_keep_other_tickets_valid_when_one_vehicle_leaves() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 2, 0, 1, 0));
    let first = lot.enter("CAR", "A").unwrap();
    let second = lot.enter("CAR", "B").unwrap();

    lot.exit(&first);

    assert_eq!(lot.floor_of(&first), None);
    assert_eq!(lot.floor_of(&second), Some(1));
    assert!(lot.exit(&second).is_some());
}

#[test]
fn should_report_no_floor_for_unknown_ticket() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 1, 0, 1, 0));

    assert_eq!(lot.floor_of("no-such-ticket"), None);
}

#[test]
fn should_charge_minimum_hour_for_every_vehicle_type() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 1, 1, 1, 1));
    let bike = lot.enter("MOTORCYCLE", "M").unwrap();
    let car = lot.enter("CAR", "C").unwrap();
    let truck = lot.enter("TRUCK", "T").unwrap();

    assert_eq!(lot.exit(&bike), Some(1000));
    assert_eq!(lot.exit(&car), Some(2000));
    assert_eq!(lot.exit(&truck), Some(5000));
}

#[test]
fn should_charge_each_ticket_from_its_own_entry_time() {
    let clock = FakeClock::new(1_000_000);
    let lot = sut!(lot(&clock, 1, 0, 2, 0));
    let early = lot.enter("CAR", "A").unwrap();
    clock.advance(30 * MINUTE);
    let late = lot.enter("CAR", "B").unwrap();
    clock.advance(60 * MINUTE);

    assert_eq!(lot.exit(&early), Some(4000)); // 90 minutes
    assert_eq!(lot.exit(&late), Some(2000)); // 60 minutes
}

#[test]
fn should_charge_returning_plate_from_its_second_entry() {
    let clock = FakeClock::new(1_000_000);
    let lot = sut!(lot(&clock, 1, 0, 1, 0));
    let first = lot.enter("CAR", "A").unwrap();
    clock.advance(3 * HOUR);
    assert_eq!(lot.exit(&first), Some(6000));

    let second = lot.enter("CAR", "A").unwrap();
    clock.advance(20 * MINUTE);

    assert_eq!(lot.exit(&second), Some(2000));
}

#[test]
fn should_refuse_parked_plate_on_another_floor_or_with_another_type() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 2, 1, 1, 1));
    assert!(lot.enter("CAR", "X").is_some());

    assert!(lot.enter("CAR", "X").is_none());
    assert!(lot.enter("MOTORCYCLE", "X").is_none());
    assert!(lot.enter("TRUCK", "X").is_none());
    assert_eq!(lot.available_spots("MOTORCYCLE"), 5);
}

#[test]
fn should_keep_ticket_usable_when_same_plate_is_refused() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 1, 0, 2, 0));
    let ticket = lot.enter("CAR", "X").unwrap();
    assert!(lot.enter("CAR", "X").is_none());

    assert_eq!(lot.floor_of(&ticket), Some(0));
    assert_eq!(lot.exit(&ticket), Some(2000));
    assert!(lot.enter("CAR", "X").is_some());
}

#[test]
fn should_let_only_one_concurrent_exit_succeed_per_ticket() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 1, 0, 1, 0));
    let ticket = lot.enter("CAR", "A").unwrap();

    let results = collect_concurrently(20, |_| lot.exit(&ticket));

    assert_eq!(results.iter().filter(|r| r.is_some()).count(), 1);
    assert_eq!(lot.available_spots("CAR"), 1);
}

#[test]
fn should_let_only_one_concurrent_entry_succeed_per_plate() {
    let clock = FakeClock::new(0);
    let lot = sut!(lot(&clock, 1, 0, 50, 0));

    let results = collect_concurrently(30, |_| lot.enter("CAR", "SAME"));

    assert_eq!(results.iter().filter(|r| r.is_some()).count(), 1);
    assert_eq!(lot.available_spots("CAR"), 49);
}
