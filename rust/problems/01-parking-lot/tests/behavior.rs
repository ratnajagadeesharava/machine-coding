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
