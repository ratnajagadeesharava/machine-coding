#![allow(unused_imports, clippy::needless_range_loop)]
use p12_travel_itinerary::{contract::*, create};
use playground_support::sut;

fn planner() -> Option<Box<dyn ItineraryApi>> {
    let mut p = create()?;
    p.create_trip("T", 10, 12, 500);
    p.set_travel_minutes("Hotel", "Museum", 30);
    Some(p)
}

#[test]
fn should_add_activities_and_list_them_in_time_order() {
    let mut p = sut!(planner());

    assert!(p.add_activity("T", "lunch", "Hotel", 10, 720, 780, 20));
    assert!(p.add_activity("T", "breakfast", "Hotel", 10, 480, 540, 10));

    assert_eq!(p.agenda("T", 10), vec!["breakfast", "lunch"]);
    assert!(p.agenda("T", 11).is_empty());
    assert_eq!(p.total_cost("T"), 30);
}

#[test]
fn should_refuse_activities_outside_the_trip_or_with_invalid_data() {
    let mut p = sut!(planner());

    assert!(!p.add_activity("T", "a", "Hotel", 9, 480, 540, 0));
    assert!(!p.add_activity("T", "a", "Hotel", 13, 480, 540, 0));
    assert!(!p.add_activity("T", "a", "Hotel", 10, 540, 540, 0));
    assert!(!p.add_activity("T", "a", "Hotel", 10, 600, 540, 0));
    assert!(!p.add_activity("T", "a", "Hotel", 10, -5, 60, 0));
    assert!(!p.add_activity("T", "a", "Hotel", 10, 1400, 1500, 0));
    assert!(!p.add_activity("T", "a", "Hotel", 10, 480, 540, -1));
    assert!(!p.add_activity("NOPE", "a", "Hotel", 10, 480, 540, 0));
    assert!(p.agenda("T", 10).is_empty());
}

#[test]
fn should_refuse_duplicate_activity_ids() {
    let mut p = sut!(planner());
    p.add_activity("T", "a", "Hotel", 10, 480, 540, 5);

    assert!(!p.add_activity("T", "a", "Hotel", 11, 480, 540, 5));
    assert_eq!(p.total_cost("T"), 5);
}

#[test]
fn should_refuse_overlapping_activities_on_the_same_day() {
    let mut p = sut!(planner());
    p.add_activity("T", "a", "Hotel", 10, 480, 600, 0);

    assert!(!p.add_activity("T", "b", "Hotel", 10, 540, 660, 0));
    assert!(!p.add_activity("T", "c", "Hotel", 10, 420, 481, 0));
    assert!(!p.add_activity("T", "d", "Hotel", 10, 500, 520, 0));
    assert!(!p.add_activity("T", "e", "Hotel", 10, 400, 700, 0));
}

#[test]
fn should_allow_touching_activities_at_the_same_location_and_overlap_on_other_days() {
    let mut p = sut!(planner());
    p.add_activity("T", "a", "Hotel", 10, 480, 600, 0);

    assert!(p.add_activity("T", "b", "Hotel", 10, 600, 660, 0));
    assert!(p.add_activity("T", "c", "Hotel", 10, 420, 480, 0));
    assert!(p.add_activity("T", "d", "Hotel", 11, 480, 600, 0));
    assert_eq!(p.agenda("T", 10), vec!["c", "a", "b"]);
}

#[test]
fn should_enforce_travel_time_between_different_locations() {
    let mut p = sut!(planner());
    p.add_activity("T", "a1", "Hotel", 10, 540, 600, 0);

    assert!(!p.add_activity("T", "a2", "Museum", 10, 620, 700, 50));
    assert!(p.add_activity("T", "a2", "Museum", 10, 630, 700, 50));
    assert_eq!(p.agenda("T", 10), vec!["a1", "a2"]);
}

#[test]
fn should_apply_travel_time_symmetrically() {
    let mut p = sut!(planner());
    p.add_activity("T", "m", "Museum", 10, 600, 660, 0);

    assert!(!p.add_activity("T", "h1", "Hotel", 10, 680, 750, 0)); // gap 20 < 30
    assert!(p.add_activity("T", "h2", "Hotel", 10, 690, 750, 0)); // gap 30
}

#[test]
fn should_check_both_neighbours_when_inserting_between_two_activities() {
    let mut p = sut!(planner());
    p.add_activity("T", "early", "Hotel", 10, 480, 540, 0);
    p.add_activity("T", "late", "Hotel", 10, 700, 760, 0);

    // Museum visit needs 30 min after the hotel and 30 min before the hotel again.
    assert!(!p.add_activity("T", "m1", "Museum", 10, 560, 650, 0)); // 20 after early
    assert!(!p.add_activity("T", "m2", "Museum", 10, 570, 680, 0)); // 20 before late
    assert!(p.add_activity("T", "m3", "Museum", 10, 570, 670, 0));
}

#[test]
fn should_treat_unspecified_travel_as_zero() {
    let mut p = sut!(planner());
    p.add_activity("T", "a", "Airport", 10, 480, 540, 0);

    assert!(p.add_activity("T", "b", "Beach", 10, 540, 600, 0));
}

#[test]
fn should_enforce_budget_and_allow_spending_it_exactly() {
    let mut p = sut!(planner());

    assert!(p.add_activity("T", "a", "Hotel", 10, 480, 540, 300));
    assert!(!p.add_activity("T", "b", "Hotel", 10, 600, 660, 201));
    assert!(p.add_activity("T", "c", "Hotel", 10, 600, 660, 200));
    assert!(!p.add_activity("T", "d", "Hotel", 10, 700, 760, 1));
    assert_eq!(p.total_cost("T"), 500);
}

#[test]
fn should_free_time_and_budget_when_removing_an_activity() {
    let mut p = sut!(planner());
    p.add_activity("T", "a", "Hotel", 10, 480, 540, 300);

    assert!(p.remove_activity("T", "a"));

    assert_eq!(p.total_cost("T"), 0);
    assert!(p.agenda("T", 10).is_empty());
    assert!(p.add_activity("T", "b", "Hotel", 10, 480, 540, 500));
}

#[test]
fn should_refuse_removing_unknown_activities_or_from_unknown_trips() {
    let mut p = sut!(planner());

    assert!(!p.remove_activity("T", "ghost"));
    assert!(!p.remove_activity("NOPE", "a"));
    assert_eq!(p.total_cost("NOPE"), 0);
    assert!(p.agenda("NOPE", 10).is_empty());
}

#[test]
fn should_keep_trips_independent() {
    let mut p = sut!(planner());
    p.create_trip("U", 10, 12, 100);

    p.add_activity("T", "a", "Hotel", 10, 480, 540, 50);

    assert!(p.add_activity("U", "a", "Hotel", 10, 480, 540, 60));
    assert_eq!(p.total_cost("T"), 50);
    assert_eq!(p.total_cost("U"), 60);
}

#[test]
fn should_handle_many_activities_per_day() {
    let mut p = sut!(create());
    p.create_trip("BIG", 1, 60, 1_000_000);
    for day in 1..=60 {
        for i in 0..30 {
            assert!(p.add_activity(
                "BIG",
                &format!("d{day}-{i}"),
                "X",
                day,
                i * 40,
                i * 40 + 30,
                1
            ));
        }
    }
    assert_eq!(p.agenda("BIG", 30).len(), 30);
    assert_eq!(p.agenda("BIG", 30)[0], "d30-0");
    assert_eq!(p.total_cost("BIG"), 1800);
}
