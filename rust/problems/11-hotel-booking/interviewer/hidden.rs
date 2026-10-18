#![allow(unused_imports, clippy::needless_range_loop)]
use p11_hotel_booking::{contract::*, create};
use playground_support::{sut, FakeClock};
use std::sync::Arc;

fn hotel(clock: &FakeClock) -> Option<Arc<dyn HotelBookingApi>> {
    let h = create(clock.as_clock())?;
    h.add_room("A", "STD", 50);
    h.add_room("B", "STD", 50);
    Some(h)
}

#[test]
fn should_accept_one_night_stays() {
    let clock = FakeClock::new(0);
    let h = sut!(hotel(&clock));
    let id = h.book("x", "STD", 7, 8).unwrap();
    assert_eq!(h.total_price(&id), Some(50));
    assert_eq!(h.available_rooms("STD", 7), 1);
    assert_eq!(h.available_rooms("STD", 8), 2);
}

#[test]
fn should_not_let_two_partial_overlaps_block_every_room_incorrectly() {
    let clock = FakeClock::new(0);
    let h = sut!(hotel(&clock));
    h.book("a", "STD", 1, 3); // A
    h.book("b", "STD", 4, 6); // A
    h.book("c", "STD", 2, 5); // B
    let id = h.book("d", "STD", 3, 4).unwrap();
    assert_eq!(h.room_of(&id).as_deref(), Some("A"));
}

#[test]
fn cancelling_an_expired_hold_should_be_refused() {
    let clock = FakeClock::new(0);
    let h = sut!(hotel(&clock));
    let hold = h.hold("x", "STD", 1, 2).unwrap();
    clock.advance(20 * 60_000);
    assert!(!h.cancel(&hold));
}

#[test]
fn types_should_be_case_sensitive() {
    let clock = FakeClock::new(0);
    let h = sut!(hotel(&clock));
    assert!(h.book("x", "std", 1, 2).is_none());
}

#[test]
fn expired_hold_should_not_block_later_bookings_of_the_same_room() {
    let clock = FakeClock::new(0);
    let h = sut!(hotel(&clock));
    h.hold("x", "STD", 1, 5);
    h.hold("y", "STD", 1, 5);
    assert!(h.book("z", "STD", 2, 3).is_none());
    clock.advance(16 * 60_000);
    assert!(h.book("z", "STD", 2, 3).is_some());
}
