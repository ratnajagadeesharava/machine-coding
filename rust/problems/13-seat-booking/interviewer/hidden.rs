#![allow(unused_imports, clippy::needless_range_loop)]
use p13_seat_booking::{contract::*, create};
use playground_support::sut;
use std::sync::Arc;

fn system(seats: u32) -> Option<Arc<dyn SeatBookingApi>> {
    let api = create()?;
    api.add_trip("T", &["A", "B", "C", "D", "E"], seats);
    Some(api)
}

#[test]
fn should_handle_overlapping_but_not_nested_journeys() {
    let api = sut!(system(2));
    let a = api.book("T", "A", "C", "p1").unwrap(); // seat 1
    let b = api.book("T", "B", "D", "p2").unwrap(); // seat 2
    let c = api.book("T", "C", "E", "p3").unwrap(); // seat 1 (free from C)
    assert_eq!(api.seat_of(&a), Some(1));
    assert_eq!(api.seat_of(&b), Some(2));
    assert_eq!(api.seat_of(&c), Some(1));
    assert!(api.book("T", "A", "E", "p4").is_none());
}

#[test]
fn book_seat_should_succeed_when_seat_is_busy_only_outside_the_journey() {
    let api = sut!(system(1));
    api.book_seat("T", 1, "A", "B", "p1");
    assert!(api.book_seat("T", 1, "D", "E", "p2").is_some());
    assert!(api.book_seat("T", 1, "B", "D", "p3").is_some());
    assert_eq!(api.available_seats("T", "A", "E"), 0);
}

#[test]
fn cancelling_one_booking_must_not_free_neighbouring_segments() {
    let api = sut!(system(1));
    let a = api.book("T", "A", "B", "p1").unwrap();
    api.book("T", "B", "C", "p2");
    api.cancel(&a);
    assert_eq!(api.available_seats("T", "A", "B"), 1);
    assert_eq!(api.available_seats("T", "B", "C"), 0);
    assert!(api.book("T", "A", "C", "p3").is_none());
}

#[test]
fn whole_route_journey_should_need_a_fully_free_seat() {
    let api = sut!(system(2));
    api.book_seat("T", 1, "B", "C", "p1");
    api.book_seat("T", 2, "D", "E", "p2");
    assert!(api.book("T", "A", "E", "p3").is_none());
    assert_eq!(api.available_seats("T", "A", "B"), 2);
}
