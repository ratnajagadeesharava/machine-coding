#![allow(unused_imports, clippy::needless_range_loop)]
use p12_travel_itinerary::{contract::*, create};
use playground_support::sut;

#[test]
fn removal_that_would_break_travel_time_between_new_neighbours_should_be_refused() {
    let mut p = sut!(create());
    p.create_trip("T", 1, 1, 1000);
    p.set_travel_minutes("X", "Y", 30);
    p.set_travel_minutes("Y", "Z", 30);
    p.set_travel_minutes("X", "Z", 180);
    assert!(p.add_activity("T", "A", "X", 1, 540, 600, 0));
    assert!(p.add_activity("T", "B", "Y", 1, 630, 690, 0));
    assert!(p.add_activity("T", "C", "Z", 1, 720, 780, 0));

    assert!(!p.remove_activity("T", "B"));

    assert_eq!(p.agenda("T", 1), vec!["A", "B", "C"]);
}

#[test]
fn removal_that_keeps_consecutive_activities_feasible_should_succeed() {
    let mut p = sut!(create());
    p.create_trip("T", 1, 1, 1000);
    p.set_travel_minutes("X", "Z", 100);
    p.add_activity("T", "A", "X", 1, 540, 600, 0);
    p.add_activity("T", "B", "X", 1, 600, 650, 0);
    p.add_activity("T", "C", "Z", 1, 760, 800, 0);

    assert!(p.remove_activity("T", "B"));
    assert_eq!(p.agenda("T", 1), vec!["A", "C"]);
}

#[test]
fn touching_activities_at_different_locations_need_zero_travel_time_to_be_allowed() {
    let mut p = sut!(create());
    p.create_trip("T", 1, 1, 1000);
    p.set_travel_minutes("X", "Y", 1);
    p.add_activity("T", "A", "X", 1, 540, 600, 0);

    assert!(!p.add_activity("T", "B", "Y", 1, 600, 660, 0));
    assert!(p.add_activity("T", "B", "Y", 1, 601, 660, 0));
}

#[test]
fn travel_time_set_after_planning_should_not_invalidate_existing_plans() {
    let mut p = sut!(create());
    p.create_trip("T", 1, 1, 1000);
    p.add_activity("T", "A", "X", 1, 540, 600, 0);
    p.add_activity("T", "B", "Y", 1, 600, 660, 0);
    p.set_travel_minutes("X", "Y", 120);

    assert_eq!(p.agenda("T", 1), vec!["A", "B"]);
    assert!(p.add_activity("T", "C", "Y", 1, 700, 720, 0));
}

#[test]
fn should_check_budget_before_mutating() {
    let mut p = sut!(create());
    p.create_trip("T", 1, 1, 10);
    assert!(!p.add_activity("T", "A", "X", 1, 540, 600, 11));
    assert!(p.agenda("T", 1).is_empty());
    assert!(p.add_activity("T", "A", "X", 1, 540, 600, 10));
}
