#![allow(unused_imports, clippy::needless_range_loop)]
use p14_ride_sharing::{contract::*, create};
use playground_support::sut;

fn service() -> Option<Box<dyn RideSharingApi>> {
    let mut s = create()?;
    s.add_driver("d1", 0, 0);
    s.add_driver("d2", 3, 0);
    Some(s)
}

#[test]
fn should_match_the_nearest_available_driver() {
    let mut s = sut!(service());

    let ride = s.request_ride("r1", 1, 0, 5, 5).unwrap();

    assert_eq!(s.driver_of(&ride).as_deref(), Some("d1"));
    assert_eq!(s.status_of(&ride), "MATCHED");
}

#[test]
fn should_break_ties_by_smallest_driver_id() {
    let mut s = sut!(create());
    s.add_driver("d9", 2, 0);
    s.add_driver("d3", 0, 2);
    s.add_driver("d5", 1, 1);

    let ride = s.request_ride("r1", 0, 0, 1, 1).unwrap();

    assert_eq!(s.driver_of(&ride).as_deref(), Some("d3"));
}

#[test]
fn should_use_manhattan_distance() {
    let mut s = sut!(create());
    s.add_driver("far-diagonal", 4, 4); // Manhattan 8, Euclidean ~5.7
    s.add_driver("near-straight", 7, 0); // Manhattan 7, Euclidean 7

    let ride = s.request_ride("r1", 0, 0, 1, 1).unwrap();

    assert_eq!(s.driver_of(&ride).as_deref(), Some("near-straight"));
}

#[test]
fn should_refuse_when_no_driver_is_within_range() {
    let mut s = sut!(service());

    assert!(s.request_ride("r1", 50, 50, 0, 0).is_none());
}

#[test]
fn should_not_match_a_busy_driver() {
    let mut s = sut!(service());
    s.request_ride("r1", 1, 0, 5, 5);

    let second = s.request_ride("r2", 1, 0, 2, 2).unwrap();

    assert_eq!(s.driver_of(&second).as_deref(), Some("d2"));
    assert!(s.request_ride("r3", 1, 0, 2, 2).is_none());
}

#[test]
fn should_allow_only_one_active_ride_per_rider() {
    let mut s = sut!(service());
    s.request_ride("r1", 1, 0, 5, 5);

    assert!(s.request_ride("r1", 1, 0, 2, 2).is_none());
}

#[test]
fn should_progress_through_the_ride_lifecycle() {
    let mut s = sut!(service());
    let ride = s.request_ride("r1", 1, 0, 5, 5).unwrap();

    assert!(s.complete_ride(&ride).is_none()); // cannot complete before starting
    assert!(s.start_ride(&ride));
    assert_eq!(s.status_of(&ride), "IN_PROGRESS");
    assert!(!s.start_ride(&ride));
    assert!(s.complete_ride(&ride).is_some());
    assert_eq!(s.status_of(&ride), "COMPLETED");
}

#[test]
fn should_charge_base_fare_and_per_block_distance() {
    let mut s = sut!(service());
    let ride = s.request_ride("r1", 1, 0, 5, 5).unwrap();
    s.start_ride(&ride);

    assert_eq!(s.complete_ride(&ride), Some(500 + 100 * (4 + 5)));
}

#[test]
fn should_move_driver_to_drop_location_and_make_them_available_again() {
    let mut s = sut!(create());
    s.add_driver("d1", 0, 0);
    let ride = s.request_ride("r1", 1, 0, 8, 8).unwrap();
    s.start_ride(&ride);
    s.complete_ride(&ride);

    assert_eq!(s.driver_location("d1").as_deref(), Some("8,8"));
    let next = s.request_ride("r2", 9, 9, 0, 0).unwrap();
    assert_eq!(s.driver_of(&next).as_deref(), Some("d1"));
}

#[test]
fn should_release_driver_when_a_matched_ride_is_cancelled() {
    let mut s = sut!(create());
    s.add_driver("d1", 0, 0);
    let ride = s.request_ride("r1", 1, 0, 8, 8).unwrap();

    assert!(s.cancel_ride(&ride));

    assert_eq!(s.status_of(&ride), "CANCELLED");
    assert_eq!(s.driver_location("d1").as_deref(), Some("0,0"));
    assert!(s.request_ride("r2", 1, 0, 2, 2).is_some());
}

#[test]
fn should_let_a_rider_request_again_after_cancellation_or_completion() {
    let mut s = sut!(service());
    let first = s.request_ride("r1", 1, 0, 5, 5).unwrap();
    s.cancel_ride(&first);
    let second = s.request_ride("r1", 1, 0, 5, 5).unwrap();
    s.start_ride(&second);
    s.complete_ride(&second);

    assert!(s.request_ride("r1", 5, 5, 6, 6).is_some());
}

#[test]
fn should_refuse_illegal_transitions() {
    let mut s = sut!(service());
    let ride = s.request_ride("r1", 1, 0, 5, 5).unwrap();
    s.start_ride(&ride);

    assert!(!s.cancel_ride(&ride)); // in progress
    s.complete_ride(&ride);
    assert!(!s.cancel_ride(&ride)); // completed
    assert!(!s.start_ride(&ride));
    assert!(s.complete_ride(&ride).is_none());
}

#[test]
fn should_handle_unknown_ids_gracefully() {
    let mut s = sut!(service());

    assert_eq!(s.status_of("nope"), "UNKNOWN");
    assert!(s.driver_of("nope").is_none());
    assert!(!s.start_ride("nope"));
    assert!(!s.cancel_ride("nope"));
    assert!(s.complete_ride("nope").is_none());
    assert!(s.driver_location("ghost").is_none());
}

#[test]
fn should_find_the_nearest_among_many_drivers() {
    let mut s = sut!(create());
    for i in 0..2000 {
        s.add_driver(&format!("d{i:05}"), 1000 + i, 1000 + i);
    }
    s.add_driver("near", 4, 4);

    let ride = s.request_ride("r1", 0, 0, 1, 1).unwrap();

    assert_eq!(s.driver_of(&ride).as_deref(), Some("near"));
}
