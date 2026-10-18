#![allow(unused_imports, clippy::needless_range_loop)]
use p13_seat_booking::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::collections::HashSet;
use std::sync::Arc;

fn system() -> Option<Arc<dyn SeatBookingApi>> {
    let api = create()?;
    api.add_trip("T", &["A", "B", "C", "D"], 2);
    Some(api)
}

#[test]
fn should_assign_lowest_seat_and_reuse_it_for_non_overlapping_journeys() {
    let api = sut!(system());

    let p1 = api.book("T", "A", "C", "p1").unwrap();
    let p2 = api.book("T", "C", "D", "p2").unwrap();

    assert_eq!(api.seat_of(&p1), Some(1));
    assert_eq!(api.seat_of(&p2), Some(1));
}

#[test]
fn should_use_another_seat_when_journeys_overlap() {
    let api = sut!(system());
    api.book("T", "A", "C", "p1");

    let p3 = api.book("T", "B", "D", "p3").unwrap();

    assert_eq!(api.seat_of(&p3), Some(2));
}

#[test]
fn should_refuse_when_no_seat_is_free_for_the_whole_journey() {
    let api = sut!(system());
    api.book("T", "A", "C", "p1");
    api.book("T", "B", "D", "p3");

    assert!(api.book("T", "B", "C", "p4").is_none());
    assert!(api.book("T", "A", "D", "p5").is_none());
}

#[test]
fn should_report_availability_per_journey() {
    let api = sut!(system());
    api.book("T", "A", "B", "p1");

    assert_eq!(api.available_seats("T", "A", "B"), 1);
    assert_eq!(api.available_seats("T", "B", "D"), 2);
    assert_eq!(api.available_seats("T", "A", "D"), 1);
    assert_eq!(api.available_seats("T", "C", "D"), 2);
}

#[test]
fn should_report_zero_availability_when_every_seat_is_busy_on_some_segment() {
    let api = sut!(system());
    api.book("T", "A", "B", "p1"); // seat 1, segment A-B
    api.book("T", "C", "D", "p2"); // seat 1, segment C-D
    api.book("T", "B", "C", "p3"); // seat 1, segment B-C
    api.book("T", "A", "D", "p4"); // seat 2 whole trip

    assert_eq!(api.available_seats("T", "A", "D"), 0);
    assert_eq!(api.available_seats("T", "B", "C"), 0);
}

#[test]
fn should_book_specific_seat() {
    let api = sut!(system());

    let id = api.book_seat("T", 2, "A", "B", "p1").unwrap();

    assert_eq!(api.seat_of(&id), Some(2));
    assert!(api.book_seat("T", 2, "A", "B", "p2").is_none());
    assert!(api.book_seat("T", 2, "B", "C", "p3").is_some());
    let auto = api.book("T", "A", "B", "p4").unwrap();
    assert_eq!(api.seat_of(&auto), Some(1));
}

#[test]
fn should_refuse_seats_out_of_range() {
    let api = sut!(system());

    assert!(api.book_seat("T", 0, "A", "B", "p").is_none());
    assert!(api.book_seat("T", 3, "A", "B", "p").is_none());
    assert!(api.book_seat("T", -1, "A", "B", "p").is_none());
}

#[test]
fn should_refuse_invalid_journeys() {
    let api = sut!(system());

    assert!(api.book("T", "B", "B", "p").is_none());
    assert!(api.book("T", "C", "A", "p").is_none());
    assert!(api.book("T", "A", "Z", "p").is_none());
    assert!(api.book("T", "Z", "A", "p").is_none());
    assert!(api.book("NOPE", "A", "B", "p").is_none());
    assert_eq!(api.available_seats("T", "D", "A"), 0);
    assert_eq!(api.available_seats("NOPE", "A", "B"), 0);
    assert_eq!(api.available_seats("T", "A", "D"), 2);
}

#[test]
fn should_free_only_the_cancelled_segments() {
    let api = sut!(system());
    let a = api.book("T", "A", "C", "p1").unwrap();
    api.book("T", "C", "D", "p2");

    assert!(api.cancel(&a));

    assert_eq!(api.available_seats("T", "A", "C"), 2);
    assert_eq!(api.available_seats("T", "C", "D"), 1);
    assert_eq!(api.seat_of(&a), None);
}

#[test]
fn should_refuse_cancelling_twice_or_unknown_bookings() {
    let api = sut!(system());
    let a = api.book("T", "A", "B", "p1").unwrap();

    assert!(api.cancel(&a));
    assert!(!api.cancel(&a));
    assert!(!api.cancel("nope"));
    assert_eq!(api.available_seats("T", "A", "B"), 2);
}

#[test]
fn should_assign_lowest_free_seat_again_after_cancellation() {
    let api = sut!(system());
    let a = api.book("T", "A", "D", "p1").unwrap(); // seat 1
    api.book("T", "A", "D", "p2"); // seat 2

    api.cancel(&a);

    let again = api.book("T", "B", "C", "p3").unwrap();
    assert_eq!(api.seat_of(&again), Some(1));
}

#[test]
fn should_keep_trips_independent() {
    let api = sut!(system());
    api.add_trip("U", &["A", "B"], 1);

    api.book("T", "A", "B", "p1");
    api.book("T", "A", "B", "p2");

    assert_eq!(api.available_seats("U", "A", "B"), 1);
    assert!(api.book("U", "A", "B", "p3").is_some());
}

#[test]
fn should_never_sell_more_seats_than_exist_when_many_passengers_race() {
    let api = sut!(create());
    api.add_trip("BUS", &["A", "B"], 40);

    let results = collect_concurrently(100, |i| api.book("BUS", "A", "B", &format!("p{i}")));

    let ids: Vec<String> = results.into_iter().flatten().collect();
    assert_eq!(ids.len(), 40);
    let seats: HashSet<u32> = ids.iter().map(|id| api.seat_of(id).unwrap()).collect();
    assert_eq!(seats.len(), 40);
    assert_eq!(api.available_seats("BUS", "A", "B"), 0);
}

#[test]
fn should_never_double_sell_a_seat_on_overlapping_journeys_under_concurrency() {
    let api = sut!(create());
    let stops = ["S0", "S1", "S2", "S3", "S4", "S5"];
    api.add_trip("TRAIN", &stops, 5);
    let journey = |i: usize| {
        let from = i % 5;
        let to = (from + 1 + (i % 3)).min(5);
        (from, to)
    };

    let results = collect_concurrently(60, |i| {
        let (from, to) = journey(i);
        api.book("TRAIN", stops[from], stops[to], &format!("p{i}"))
    });

    let mut occupancy = [[0u32; 6]; 5];
    for (i, r) in results.iter().enumerate() {
        let Some(id) = r else { continue };
        let (from, to) = journey(i);
        let seat = api.seat_of(id).unwrap() as usize;
        for seg in from..to {
            occupancy[seat - 1][seg] += 1;
            assert_eq!(occupancy[seat - 1][seg], 1, "seat {seat} segment {seg}");
        }
    }
}

#[test]
fn should_restore_full_availability_after_concurrent_book_and_cancel() {
    let api = sut!(create());
    let stops = ["A", "B", "C", "D"];
    api.add_trip("T", &stops, 3);

    run_concurrently(16, |t| {
        for i in 0..200usize {
            let from = (t + i) % 3;
            let to = if i % 2 == 0 { 3 } else { from + 1 };
            if let Some(id) = api.book("T", stops[from], stops[to], &format!("p{t}")) {
                api.cancel(&id);
            }
        }
    });

    assert_eq!(api.available_seats("T", "A", "D"), 3);
}
