#![allow(unused_imports, clippy::needless_range_loop)]
use p10_flight_booking::{contract::*, create};
use playground_support::{sut, FakeClock};
use std::sync::Arc;

const MINUTE: u64 = 60_000;
const HOUR: u64 = 60 * MINUTE;
const NOW: u64 = 5_000_000;
const DEPARTURE: u64 = NOW + 100 * HOUR;

fn system(clock: &FakeClock) -> Option<Arc<dyn FlightBookingApi>> {
    let api = create(clock.as_clock())?;
    api.add_flight("F1", "BLR", "DEL", DEPARTURE, 3, 1, 5001, 20001);
    Some(api)
}

#[test]
fn should_give_full_refund_exactly_24_hours_before_and_half_one_millisecond_later() {
    let clock = FakeClock::new(NOW);
    let api = sut!(system(&clock));
    let a = api.reserve("F1", "E1", "A").unwrap();
    api.confirm(&a, true);
    let b = api.reserve("F1", "E2", "B").unwrap();
    api.confirm(&b, true);

    clock.set(DEPARTURE - 24 * HOUR);
    assert_eq!(api.cancel(&a), Some(5001));
    clock.advance(1);
    assert_eq!(api.cancel(&b), Some(5001 / 2));
}

#[test]
fn should_round_half_refund_down() {
    let clock = FakeClock::new(NOW);
    let api = sut!(system(&clock));
    let id = api.reserve("F1", "B1", "A").unwrap();
    api.confirm(&id, true);
    clock.set(DEPARTURE - HOUR);
    assert_eq!(api.cancel(&id), Some(10000));
}

#[test]
fn failed_payment_should_not_extend_the_hold() {
    let clock = FakeClock::new(NOW);
    let api = sut!(system(&clock));
    let id = api.reserve("F1", "E1", "A").unwrap();
    clock.advance(9 * MINUTE);
    assert!(!api.confirm(&id, false));
    clock.advance(MINUTE);
    assert!(!api.confirm(&id, true));
}

#[test]
fn same_passenger_may_hold_several_seats() {
    let clock = FakeClock::new(NOW);
    let api = sut!(system(&clock));
    assert!(api.reserve("F1", "E1", "A").is_some());
    assert!(api.reserve("F1", "E2", "A").is_some());
    assert_eq!(api.available_seats("F1"), vec!["B1", "E3"]);
}

#[test]
fn cancelled_seat_can_be_reserved_by_another_passenger() {
    let clock = FakeClock::new(NOW);
    let api = sut!(system(&clock));
    let id = api.reserve("F1", "E1", "A").unwrap();
    api.cancel(&id);
    assert!(api.reserve("F1", "E1", "B").is_some());
    assert!(api.cancel(&id).is_none());
}
