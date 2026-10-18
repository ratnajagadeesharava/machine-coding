#![allow(unused_imports, clippy::needless_range_loop)]
use p01_parking_lot::{contract::*, create};
use playground_support::{sut, FakeClock};
use std::sync::Arc;

const HOUR: u64 = 3_600_000;

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
fn should_allow_same_plate_again_after_exit() {
    let clock = FakeClock::new(5_000);
    let lot = sut!(lot(&clock, 1, 0, 2, 0));
    let t = lot.enter("CAR", "X").unwrap();
    lot.exit(&t);
    assert!(lot.enter("CAR", "X").is_some());
}

#[test]
fn should_forget_ticket_floor_after_exit() {
    let clock = FakeClock::new(5_000);
    let lot = sut!(lot(&clock, 1, 0, 1, 0));
    let t = lot.enter("CAR", "X").unwrap();
    lot.exit(&t);
    assert_eq!(lot.floor_of(&t), None);
}

#[test]
fn should_spill_motorcycle_then_car_into_larger_spots() {
    let clock = FakeClock::new(5_000);
    let lot = sut!(lot(&clock, 1, 0, 0, 2));
    assert!(lot.enter("MOTORCYCLE", "M").is_some());
    assert!(lot.enter("CAR", "C").is_some());
    assert!(lot.enter("MOTORCYCLE", "M2").is_none());
}

#[test]
fn should_charge_long_stays_by_started_hours() {
    let clock = FakeClock::new(5_000);
    let lot = sut!(lot(&clock, 1, 0, 1, 0));
    let t = lot.enter("CAR", "X").unwrap();
    clock.advance(24 * HOUR + 1);
    assert_eq!(lot.exit(&t), Some(25 * 2000));
}

#[test]
fn refused_entry_must_not_consume_spot_or_plate() {
    let clock = FakeClock::new(5_000);
    let lot = sut!(lot(&clock, 1, 0, 0, 1));
    assert!(lot.enter("CAR", "A").is_some());
    assert!(lot.enter("CAR", "B").is_none()); // full
    assert!(lot.enter("CAR", "A").is_none()); // duplicate plate
    assert_eq!(lot.available_spots("CAR"), 0);
}
