#![allow(unused_imports, clippy::needless_range_loop)]
use p14_ride_sharing::{contract::*, create};
use playground_support::sut;

#[test]
fn should_include_drivers_exactly_at_distance_ten_and_exclude_eleven_away() {
    let mut s = sut!(create());
    s.add_driver("edge", 10, 0);
    s.add_driver("beyond", 0, 11);

    let ride = s.request_ride("r1", 0, 0, 1, 1).unwrap();

    assert_eq!(s.driver_of(&ride).as_deref(), Some("edge"));
    assert!(s.request_ride("r2", 0, 0, 1, 1).is_none());
}

#[test]
fn should_handle_negative_coordinates_and_same_pickup_and_drop() {
    let mut s = sut!(create());
    s.add_driver("d", -3, -3);
    let ride = s.request_ride("r", -4, -3, -4, -3).unwrap();
    s.start_ride(&ride);

    assert_eq!(s.complete_ride(&ride), Some(500));
    assert_eq!(s.driver_location("d").as_deref(), Some("-4,-3"));
}

#[test]
fn riders_with_active_rides_should_not_block_other_riders() {
    let mut s = sut!(create());
    s.add_driver("a", 0, 0);
    s.add_driver("b", 0, 1);
    s.request_ride("r1", 0, 0, 3, 3);
    assert!(s.request_ride("r2", 0, 0, 3, 3).is_some());
}

#[test]
fn driver_should_be_matched_from_the_drop_location_after_a_completed_ride() {
    let mut s = sut!(create());
    s.add_driver("d1", 0, 0);
    s.add_driver("d2", 30, 30);
    let ride = s.request_ride("r1", 1, 1, 30, 31).unwrap();
    s.start_ride(&ride);
    s.complete_ride(&ride);

    let next = s.request_ride("r2", 30, 32, 0, 0).unwrap();
    assert_eq!(s.driver_of(&next).as_deref(), Some("d1")); // at (30,31): distance 1; d2 is 2 away
}
