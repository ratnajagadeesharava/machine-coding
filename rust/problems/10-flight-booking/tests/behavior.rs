#![allow(unused_imports, clippy::needless_range_loop)]
use p10_flight_booking::{contract::*, create};
use playground_support::{collect_concurrently, sut, FakeClock};
use std::collections::HashSet;
use std::sync::Arc;

const MINUTE: u64 = 60_000;
const HOUR: u64 = 60 * MINUTE;
const NOW: u64 = 1_000_000_000;
const DEPARTURE: u64 = NOW + 72 * HOUR;

fn system(clock: &FakeClock) -> Option<Arc<dyn FlightBookingApi>> {
    let api = create(clock.as_clock())?;
    api.add_flight("F1", "BLR", "DEL", DEPARTURE, 3, 1, 5000, 20000);
    Some(api)
}

#[test]
fn should_search_flights_by_route_ordered_by_departure() {
    let clock = FakeClock::new(NOW);
    let api = sut!(system(&clock));
    api.add_flight("F0", "BLR", "DEL", DEPARTURE - 5 * HOUR, 2, 0, 9000, 0);
    api.add_flight("F2", "BLR", "DEL", DEPARTURE, 2, 0, 4000, 0);
    api.add_flight("X1", "BLR", "BOM", DEPARTURE, 2, 0, 3000, 0);

    assert_eq!(api.search("BLR", "DEL", NOW), vec!["F0", "F2", "F1"]);
    assert_eq!(api.search("BLR", "DEL", DEPARTURE), vec!["F2", "F1"]);
    assert!(api.search("DEL", "BLR", NOW).is_empty());
}

#[test]
fn should_list_available_seats_business_first() {
    let clock = FakeClock::new(NOW);
    let api = sut!(system(&clock));

    assert_eq!(api.available_seats("F1"), vec!["B1", "E1", "E2", "E3"]);
    assert!(api.available_seats("nope").is_empty());
}

#[test]
fn should_reserve_seat_and_hide_it_from_availability() {
    let clock = FakeClock::new(NOW);
    let api = sut!(system(&clock));

    let r = api.reserve("F1", "E2", "Asha");

    assert!(r.is_some());
    assert_eq!(api.available_seats("F1"), vec!["B1", "E1", "E3"]);
}

#[test]
fn should_refuse_seat_that_is_taken_unknown_or_on_unknown_flight() {
    let clock = FakeClock::new(NOW);
    let api = sut!(system(&clock));
    api.reserve("F1", "E2", "Asha");

    assert!(api.reserve("F1", "E2", "Ravi").is_none());
    assert!(api.reserve("F1", "E9", "Ravi").is_none());
    assert!(api.reserve("F9", "E1", "Ravi").is_none());
    assert!(api.reserve("F1", "E1", " ").is_none());
}

#[test]
fn should_keep_confirmed_booking_beyond_the_hold_window() {
    let clock = FakeClock::new(NOW);
    let api = sut!(system(&clock));
    let id = api.reserve("F1", "E2", "Asha").unwrap();

    assert!(api.confirm(&id, true));
    clock.advance(2 * HOUR);

    assert!(!api.available_seats("F1").contains(&"E2".to_string()));
    assert!(api.reserve("F1", "E2", "Ravi").is_none());
}

#[test]
fn should_release_seat_when_hold_expires() {
    let clock = FakeClock::new(NOW);
    let api = sut!(system(&clock));
    let id = api.reserve("F1", "E2", "Asha").unwrap();

    clock.advance(10 * MINUTE);

    assert!(api.available_seats("F1").contains(&"E2".to_string()));
    assert!(!api.confirm(&id, true));
    assert!(api.reserve("F1", "E2", "Ravi").is_some());
}

#[test]
fn should_keep_hold_just_before_expiry() {
    let clock = FakeClock::new(NOW);
    let api = sut!(system(&clock));
    let id = api.reserve("F1", "E2", "Asha").unwrap();

    clock.advance(10 * MINUTE - 1);

    assert!(!api.available_seats("F1").contains(&"E2".to_string()));
    assert!(api.confirm(&id, true));
}

#[test]
fn should_keep_hold_after_failed_payment_and_allow_retry() {
    let clock = FakeClock::new(NOW);
    let api = sut!(system(&clock));
    let id = api.reserve("F1", "E2", "Asha").unwrap();

    assert!(!api.confirm(&id, false));
    assert!(!api.available_seats("F1").contains(&"E2".to_string()));
    clock.advance(5 * MINUTE);
    assert!(api.confirm(&id, true));
}

#[test]
fn should_not_confirm_twice() {
    let clock = FakeClock::new(NOW);
    let api = sut!(system(&clock));
    let id = api.reserve("F1", "E2", "Asha").unwrap();

    assert!(api.confirm(&id, true));
    assert!(!api.confirm(&id, true));
    assert!(!api.confirm("unknown", true));
}

#[test]
fn should_refund_full_fare_when_cancelling_well_before_departure() {
    let clock = FakeClock::new(NOW);
    let api = sut!(system(&clock));
    let id = api.reserve("F1", "E2", "Asha").unwrap();
    api.confirm(&id, true);

    assert_eq!(api.cancel(&id), Some(5000));
    assert!(api.available_seats("F1").contains(&"E2".to_string()));
}

#[test]
fn should_refund_half_when_cancelling_within_twenty_four_hours() {
    let clock = FakeClock::new(NOW);
    let api = sut!(system(&clock));
    let id = api.reserve("F1", "B1", "Asha").unwrap();
    api.confirm(&id, true);

    clock.advance(72 * HOUR - 23 * HOUR);

    assert_eq!(api.cancel(&id), Some(10000));
}

#[test]
fn should_release_seat_with_zero_refund_when_cancelling_an_unconfirmed_reservation() {
    let clock = FakeClock::new(NOW);
    let api = sut!(system(&clock));
    let id = api.reserve("F1", "E2", "Asha").unwrap();

    assert_eq!(api.cancel(&id), Some(0));
    assert!(api.available_seats("F1").contains(&"E2".to_string()));
}

#[test]
fn should_refuse_to_cancel_twice_or_cancel_unknown_ids() {
    let clock = FakeClock::new(NOW);
    let api = sut!(system(&clock));
    let id = api.reserve("F1", "E2", "Asha").unwrap();
    api.confirm(&id, true);

    assert!(api.cancel(&id).is_some());
    assert!(api.cancel(&id).is_none());
    assert!(api.cancel("nope").is_none());
}

#[test]
fn should_refuse_to_cancel_expired_reservation() {
    let clock = FakeClock::new(NOW);
    let api = sut!(system(&clock));
    let id = api.reserve("F1", "E2", "Asha").unwrap();

    clock.advance(11 * MINUTE);

    assert!(api.cancel(&id).is_none());
}

#[test]
fn should_apply_fare_by_cabin() {
    let clock = FakeClock::new(NOW);
    let api = sut!(system(&clock));

    assert_eq!(api.fare_of("F1", "B1"), Some(20000));
    assert_eq!(api.fare_of("F1", "E3"), Some(5000));
    assert_eq!(api.fare_of("F1", "E4"), None);
    assert_eq!(api.fare_of("F9", "E1"), None);
}

#[test]
fn should_not_reserve_on_flights_that_already_departed() {
    let clock = FakeClock::new(NOW);
    let api = sut!(system(&clock));

    clock.set(DEPARTURE + 1);

    assert!(api.reserve("F1", "E1", "Asha").is_none());
}

#[test]
fn should_hide_full_flights_from_search_until_holds_expire() {
    let clock = FakeClock::new(NOW);
    let api = sut!(create(clock.as_clock()));
    api.add_flight("S1", "BLR", "DEL", DEPARTURE, 1, 0, 5000, 0);
    api.reserve("S1", "E1", "Asha");

    assert!(api.search("BLR", "DEL", NOW).is_empty());
    clock.advance(10 * MINUTE);
    assert_eq!(api.search("BLR", "DEL", NOW), vec!["S1"]);
}

#[test]
fn should_allow_only_one_reservation_when_many_customers_race_for_the_same_seat() {
    let clock = FakeClock::new(NOW);
    let api = sut!(system(&clock));

    let results = collect_concurrently(32, |i| api.reserve("F1", "E1", &format!("p{i}")));

    assert_eq!(results.iter().filter(|r| r.is_some()).count(), 1);
    assert!(!api.available_seats("F1").contains(&"E1".to_string()));
}

#[test]
fn should_never_double_book_when_many_customers_race_for_many_seats() {
    let clock = FakeClock::new(NOW);
    let api = sut!(create(clock.as_clock()));
    api.add_flight("BIG", "BLR", "DEL", DEPARTURE, 40, 10, 5000, 20000);
    let seats = api.available_seats("BIG");
    assert_eq!(seats.len(), 50);

    let results = collect_concurrently(64, |i| {
        (0..seats.len())
            .find_map(|k| api.reserve("BIG", &seats[(i + k) % seats.len()], &format!("p{i}")))
    });

    let ids: Vec<String> = results.into_iter().flatten().collect();
    assert_eq!(ids.len(), 50);
    assert_eq!(ids.iter().collect::<HashSet<_>>().len(), 50);
    assert!(api.available_seats("BIG").is_empty());
    assert!(ids.iter().all(|id| api.confirm(id, true)));
}
