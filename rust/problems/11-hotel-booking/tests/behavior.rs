#![allow(unused_imports, clippy::needless_range_loop)]
use p11_hotel_booking::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut, FakeClock};
use std::collections::HashSet;
use std::sync::Arc;

const MINUTE: u64 = 60_000;

fn hotel(clock: &FakeClock) -> Option<Arc<dyn HotelBookingApi>> {
    let h = create(clock.as_clock())?;
    h.add_room("R1", "DOUBLE", 100);
    h.add_room("R2", "DOUBLE", 100);
    h.add_room("S1", "SUITE", 300);
    Some(h)
}

#[test]
fn should_book_room_and_reduce_availability_only_for_the_stay_nights() {
    let clock = FakeClock::new(1_000_000);
    let h = sut!(hotel(&clock));

    let id = h.book("alice", "DOUBLE", 1, 4);

    assert!(id.is_some());
    assert_eq!(h.available_rooms("DOUBLE", 1), 1);
    assert_eq!(h.available_rooms("DOUBLE", 3), 1);
    assert_eq!(h.available_rooms("DOUBLE", 4), 2); // checkout night is free
    assert_eq!(h.available_rooms("DOUBLE", 0), 2);
}

#[test]
fn should_assign_smallest_free_room_id_first() {
    let clock = FakeClock::new(1_000_000);
    let h = sut!(hotel(&clock));

    let first = h.book("alice", "DOUBLE", 1, 3).unwrap();
    let second = h.book("bob", "DOUBLE", 1, 3).unwrap();

    assert_eq!(h.room_of(&first).as_deref(), Some("R1"));
    assert_eq!(h.room_of(&second).as_deref(), Some("R2"));
}

#[test]
fn should_allow_back_to_back_stays_in_the_same_room() {
    let clock = FakeClock::new(1_000_000);
    let h = sut!(hotel(&clock));

    let a = h.book("alice", "DOUBLE", 1, 4).unwrap();
    let b = h.book("bob", "DOUBLE", 4, 6).unwrap();

    assert_eq!(h.room_of(&a).as_deref(), Some("R1"));
    assert_eq!(h.room_of(&b).as_deref(), Some("R1"));
}

#[test]
fn should_refuse_when_no_room_is_free_for_the_whole_range() {
    let clock = FakeClock::new(1_000_000);
    let h = sut!(hotel(&clock));
    h.book("alice", "DOUBLE", 1, 4);
    h.book("bob", "DOUBLE", 4, 6);
    h.book("carol", "DOUBLE", 2, 5);

    assert!(h.book("dave", "DOUBLE", 3, 4).is_none());
    assert!(h.book("dave", "DOUBLE", 1, 7).is_none());
}

#[test]
fn should_fit_stays_around_existing_bookings_without_double_booking() {
    let clock = FakeClock::new(1_000_000);
    let h = sut!(hotel(&clock));
    let a = h.book("a", "DOUBLE", 1, 3).unwrap(); // R1
    let b = h.book("b", "DOUBLE", 3, 5).unwrap(); // R1
    let c = h.book("c", "DOUBLE", 2, 4).unwrap(); // R2
    let d = h.book("d", "DOUBLE", 1, 2).unwrap(); // R2 (R1 busy on night 1)

    assert_eq!(h.room_of(&a).as_deref(), Some("R1"));
    assert_eq!(h.room_of(&b).as_deref(), Some("R1"));
    assert_eq!(h.room_of(&c).as_deref(), Some("R2"));
    assert_eq!(h.room_of(&d).as_deref(), Some("R2"));
    assert!(h.book("e", "DOUBLE", 1, 5).is_none());
}

#[test]
fn should_keep_room_types_independent() {
    let clock = FakeClock::new(1_000_000);
    let h = sut!(hotel(&clock));

    assert!(h.book("alice", "SUITE", 1, 3).is_some());
    assert!(h.book("bob", "SUITE", 2, 4).is_none());
    assert_eq!(h.available_rooms("DOUBLE", 2), 2);
}

#[test]
fn should_refuse_invalid_requests() {
    let clock = FakeClock::new(1_000_000);
    let h = sut!(hotel(&clock));

    assert!(h.book("alice", "DOUBLE", 3, 3).is_none());
    assert!(h.book("alice", "DOUBLE", 5, 2).is_none());
    assert!(h.book("alice", "PENTHOUSE", 1, 2).is_none());
    assert_eq!(h.available_rooms("PENTHOUSE", 1), 0);
}

#[test]
fn should_charge_nights_times_rate() {
    let clock = FakeClock::new(1_000_000);
    let h = sut!(hotel(&clock));

    let id = h.book("alice", "SUITE", 10, 13).unwrap();

    assert_eq!(h.total_price(&id), Some(900));
    assert_eq!(h.total_price("nope"), None);
}

#[test]
fn should_release_room_on_cancel() {
    let clock = FakeClock::new(1_000_000);
    let h = sut!(hotel(&clock));
    h.book("a", "SUITE", 1, 5);
    let id = h.book("b", "DOUBLE", 1, 5).unwrap();

    assert!(h.cancel(&id));

    assert_eq!(h.available_rooms("DOUBLE", 2), 2);
    assert_eq!(h.room_of(&id), None);
    assert!(!h.cancel(&id));
    assert!(!h.cancel("nope"));
}

#[test]
fn should_block_room_while_held_and_confirm_into_booking() {
    let clock = FakeClock::new(1_000_000);
    let h = sut!(hotel(&clock));
    let hold = h.hold("alice", "SUITE", 1, 3).unwrap();

    assert!(h.book("bob", "SUITE", 2, 4).is_none());
    assert!(h.confirm(&hold));
    clock.advance(60 * MINUTE);

    assert!(h.book("bob", "SUITE", 2, 4).is_none());
    assert_eq!(h.room_of(&hold).as_deref(), Some("S1"));
}

#[test]
fn should_release_room_when_hold_expires() {
    let clock = FakeClock::new(1_000_000);
    let h = sut!(hotel(&clock));
    let hold = h.hold("alice", "SUITE", 1, 3).unwrap();

    clock.advance(15 * MINUTE);

    assert_eq!(h.available_rooms("SUITE", 1), 1);
    assert!(!h.confirm(&hold));
    assert!(h.book("bob", "SUITE", 1, 3).is_some());
}

#[test]
fn should_keep_hold_just_before_expiry() {
    let clock = FakeClock::new(1_000_000);
    let h = sut!(hotel(&clock));
    let hold = h.hold("alice", "SUITE", 1, 3).unwrap();

    clock.advance(15 * MINUTE - 1);

    assert_eq!(h.available_rooms("SUITE", 1), 0);
    assert!(h.confirm(&hold));
}

#[test]
fn should_release_hold_on_cancel_and_refuse_confirming_twice() {
    let clock = FakeClock::new(1_000_000);
    let h = sut!(hotel(&clock));
    let hold = h.hold("alice", "SUITE", 1, 3).unwrap();
    assert!(h.cancel(&hold));
    assert!(!h.confirm(&hold));

    let hold2 = h.hold("bob", "SUITE", 1, 3).unwrap();
    assert!(h.confirm(&hold2));
    assert!(!h.confirm(&hold2));
}

#[test]
fn should_never_book_the_same_room_twice_under_concurrency() {
    let clock = FakeClock::new(1_000_000);
    let h = sut!(hotel(&clock));

    let results = collect_concurrently(30, |i| h.book(&format!("g{i}"), "SUITE", 5, 9));

    assert_eq!(results.iter().flatten().count(), 1);
    assert_eq!(h.available_rooms("SUITE", 6), 0);
}

#[test]
fn should_assign_distinct_rooms_to_concurrent_bookings_of_the_same_dates() {
    let clock = FakeClock::new(1_000_000);
    let h = sut!(create(clock.as_clock()));
    for i in 1..=5 {
        h.add_room(&format!("R{i}"), "DOUBLE", 100);
    }

    let results = collect_concurrently(40, |i| h.book(&format!("g{i}"), "DOUBLE", 1, 4));

    let ids: Vec<String> = results.into_iter().flatten().collect();
    assert_eq!(ids.len(), 5);
    let rooms: HashSet<String> = ids.iter().map(|id| h.room_of(id).unwrap()).collect();
    assert_eq!(rooms.len(), 5);
}

#[test]
fn should_stay_consistent_under_concurrent_book_and_cancel_of_overlapping_ranges() {
    let clock = FakeClock::new(1_000_000);
    let h = sut!(create(clock.as_clock()));
    h.add_room("R1", "DOUBLE", 100);
    h.add_room("R2", "DOUBLE", 100);

    run_concurrently(16, |t| {
        for i in 0..200usize {
            let start = ((t + i) % 7) as i32;
            if let Some(id) = h.book(
                &format!("g{t}"),
                "DOUBLE",
                start,
                start + 1 + (i % 3) as i32,
            ) {
                h.cancel(&id);
            }
        }
    });

    for day in 0..12 {
        assert_eq!(h.available_rooms("DOUBLE", day), 2);
    }
}
