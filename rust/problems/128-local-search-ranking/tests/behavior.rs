#![allow(unused_imports, clippy::needless_range_loop, clippy::too_many_arguments)]
use p128_local_search_ranking::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

type S = Arc<dyn PlaceSearch>;

/// Everything is allowed through: no category, no rating floor, price cap 4.
fn all(s: &S, query: Option<&str>, ux: i32, uy: i32, radius: i32, k: i32) -> Vec<String> {
    s.search(query, ux, uy, radius, None, 0, 4, k).unwrap()
}

fn coffee_district(s: &S) {
    s.add_place("c1", "Coffee", "cafe", 3, 0, 40, 2).unwrap();
    s.add_place("c2", "Coffee House", "cafe", 1, 0, 40, 2).unwrap();
    s.add_place("c3", "Iced Coffee Bar", "cafe", 0, 2, 45, 3).unwrap();
    s.add_place("c4", "Bean Cart", "coffee", 5, 0, 30, 1).unwrap();
    s.add_place("p1", "Pizza", "pizza", 1, 1, 50, 2).unwrap();
}

#[test]
fn should_reproduce_the_readme_coffee_examples() {
    let s = sut!(create());
    coffee_district(&s);

    assert_eq!(s.search(Some("coffee"), 0, 0, 100, None, 40, 2, 10).unwrap(), ["c1", "c2"]);
    assert_eq!(s.search(None, 0, 0, 100, Some("CAFE"), 0, 4, 2).unwrap(), ["c3", "c2"]);
    assert_eq!(s.search(Some("coffee"), 0, 0, 3, None, 0, 4, 10).unwrap(), ["c3", "c1", "c2"]);
    assert_eq!(s.nearest(0, 0, 3).unwrap(), ["c2", "c3", "p1"]);
    assert_eq!(s.categories(), ["cafe:3", "coffee:1", "pizza:1"]);
}

#[test]
fn should_rank_by_rating_times_ten_minus_three_times_distance() {
    let s = sut!(create());
    s.add_place("near", "Alpha", "cafe", 1, 0, 30, 2).unwrap(); // 297
    s.add_place("good", "Beta", "cafe", 5, 0, 40, 2).unwrap(); // 385
    s.add_place("far", "Gamma", "cafe", 40, 40, 50, 2).unwrap(); // 260

    assert_eq!(all(&s, None, 0, 0, 1000, 10), ["good", "near", "far"]);
}

#[test]
fn should_add_prefix_and_stacked_equality_bonuses_for_the_name() {
    let s = sut!(create());
    coffee_district(&s);

    // c3 444 (substring only), c1 441 (equals: +50), c2 422 (prefix: +25), c4 285 (category match, no bonus)
    assert_eq!(
        all(&s, Some("coffee"), 0, 0, 100, 10),
        ["c3", "c1", "c2", "c4"]
    );
}

#[test]
fn should_match_the_query_as_a_case_insensitive_substring_of_the_name() {
    let s = sut!(create());
    s.add_place("a", "The Golden DRAGON", "chinese", 1, 0, 40, 2).unwrap();
    s.add_place("b", "Dragonfly", "bar", 2, 0, 40, 2).unwrap();
    s.add_place("c", "Noodle Bar", "chinese", 3, 0, 40, 2).unwrap();

    assert_eq!(all(&s, Some("dragon"), 0, 0, 10, 10), ["b", "a"]);
    assert_eq!(all(&s, Some("GOLDEN dra"), 0, 0, 10, 10), ["a"]);
    assert!(all(&s, Some("xyz"), 0, 0, 10, 10).is_empty());
}

#[test]
fn should_match_the_query_when_it_equals_the_category_ignoring_case_but_not_when_only_part_of_it() {
    let s = sut!(create());
    s.add_place("a", "Spice Route", "Indian", 1, 0, 40, 2).unwrap();
    s.add_place("b", "Curry Leaf", "indian-fusion", 2, 0, 40, 2).unwrap();

    assert_eq!(all(&s, Some("INDIAN"), 0, 0, 10, 10), ["a"]);
    assert!(all(&s, Some("india"), 0, 0, 10, 10).is_empty());
    assert!(all(&s, Some("fusion"), 0, 0, 10, 10).is_empty());
}

#[test]
fn should_treat_none_and_blank_queries_as_no_text_filter_and_add_no_bonus() {
    let s = sut!(create());
    s.add_place("a", "Coffee", "cafe", 0, 0, 40, 2).unwrap();
    s.add_place("b", "Tea", "cafe", 0, 1, 41, 2).unwrap();

    assert_eq!(all(&s, None, 0, 0, 10, 10), ["b", "a"]); // 407 vs 400
    assert_eq!(all(&s, Some(""), 0, 0, 10, 10), ["b", "a"]);
    assert_eq!(all(&s, Some("   "), 0, 0, 10, 10), ["b", "a"]);
}

#[test]
fn should_include_places_exactly_at_the_radius_and_exclude_those_beyond() {
    let s = sut!(create());
    s.add_place("in", "In", "cafe", 3, -2, 40, 2).unwrap(); // distance 5
    s.add_place("out", "Out", "cafe", 3, -3, 40, 2).unwrap(); // distance 6

    assert_eq!(all(&s, None, 0, 0, 5, 10), ["in"]);
    assert_eq!(all(&s, None, 0, 0, 6, 10), ["in", "out"]);
    assert!(all(&s, None, 0, 0, 0, 10).is_empty());
}

#[test]
fn should_include_a_place_at_distance_zero_when_the_radius_is_zero() {
    let s = sut!(create());
    s.add_place("here", "Here", "cafe", 7, 7, 20, 1).unwrap();

    assert_eq!(all(&s, None, 7, 7, 0, 5), ["here"]);
}

#[test]
fn should_filter_by_category_ignoring_case() {
    let s = sut!(create());
    coffee_district(&s);

    let mut cafes = s.search(None, 0, 0, 100, Some("CAFE"), 0, 4, 10).unwrap();
    cafes.sort();
    assert_eq!(cafes, ["c1", "c2", "c3"]);
    assert_eq!(s.search(None, 0, 0, 100, Some("Pizza"), 0, 4, 10).unwrap(), ["p1"]);
    assert!(s.search(None, 0, 0, 100, Some("sushi"), 0, 4, 10).unwrap().is_empty());
    assert_eq!(s.search(None, 0, 0, 100, Some(""), 0, 4, 10).unwrap().len(), 5);
}

#[test]
fn should_filter_by_minimum_rating_inclusive() {
    let s = sut!(create());
    coffee_district(&s);

    assert_eq!(s.search(Some("coffee"), 0, 0, 100, None, 45, 4, 10).unwrap(), ["c3"]);
    assert_eq!(s.search(Some("coffee"), 0, 0, 100, None, 40, 4, 10).unwrap(), ["c3", "c1", "c2"]);
    assert!(s.search(Some("coffee"), 0, 0, 100, None, 46, 4, 10).unwrap().is_empty());
}

#[test]
fn should_filter_by_maximum_price_level_inclusive() {
    let s = sut!(create());
    coffee_district(&s);

    assert_eq!(s.search(Some("coffee"), 0, 0, 100, None, 0, 3, 10).unwrap(), ["c3", "c1", "c2", "c4"]);
    assert_eq!(s.search(Some("coffee"), 0, 0, 100, None, 0, 2, 10).unwrap(), ["c1", "c2", "c4"]);
    assert_eq!(s.search(Some("coffee"), 0, 0, 100, None, 0, 1, 10).unwrap(), ["c4"]);
    assert!(s.search(Some("coffee"), 0, 0, 100, None, 0, 0, 10).unwrap().is_empty());
}

#[test]
fn should_break_equal_scores_by_distance_then_place_id() {
    let s = sut!(create());
    s.add_place("far", "Far", "cafe", 10, 0, 43, 2).unwrap(); // 400, distance 10
    s.add_place("close", "Close", "cafe", 0, 0, 40, 2).unwrap(); // 400, distance 0
    s.add_place("b", "Twin B", "cafe", 0, 5, 41, 2).unwrap(); // 395, distance 5
    s.add_place("a", "Twin A", "cafe", 5, 0, 41, 2).unwrap(); // 395, distance 5

    assert_eq!(all(&s, None, 0, 0, 100, 10), ["close", "far", "a", "b"]);
}

#[test]
fn should_return_at_most_k_results_keeping_the_best_ones() {
    let s = sut!(create());
    for i in 0..20 {
        s.add_place(&format!("p{i:02}"), &format!("Place {i}"), "cafe", i, 0, 20, 2).unwrap();
    }

    assert_eq!(all(&s, None, 0, 0, 100, 3), ["p00", "p01", "p02"]);
    assert_eq!(all(&s, None, 0, 0, 100, 1000).len(), 20);
}

#[test]
fn should_return_an_empty_list_when_nothing_matches_or_nothing_exists() {
    let s = sut!(create());
    assert!(all(&s, Some("coffee"), 0, 0, 10, 5).is_empty());
    assert!(s.nearest(0, 0, 5).unwrap().is_empty());
    assert!(s.categories().is_empty());
}

#[test]
fn should_rank_negative_scores_correctly() {
    let s = sut!(create());
    s.add_place("a", "A", "cafe", 1000, 0, 0, 1).unwrap(); // -3000
    s.add_place("b", "B", "cafe", 500, 0, 0, 1).unwrap(); // -1500
    s.add_place("c", "C", "cafe", 2000, 0, 50, 1).unwrap(); // -5500

    assert_eq!(all(&s, None, 0, 0, 5000, 10), ["b", "a", "c"]);
}

#[test]
fn should_handle_negative_coordinates() {
    let s = sut!(create());
    s.add_place("a", "A", "cafe", -3, -4, 40, 2).unwrap();
    s.add_place("b", "B", "cafe", 3, 4, 40, 2).unwrap();

    assert_eq!(all(&s, None, 0, 0, 7, 10), ["a", "b"]);
    assert_eq!(all(&s, None, -3, -4, 0, 10), ["a"]);
}

#[test]
fn should_reject_invalid_search_arguments() {
    let s = sut!(create());
    coffee_district(&s);

    let bad = Err(SearchError::InvalidArgument);
    assert_eq!(s.search(None, 0, 0, 10, None, 0, 4, 0), bad);
    assert_eq!(s.search(None, 0, 0, 10, None, 0, 4, -3), bad);
    assert_eq!(s.search(None, 0, 0, -1, None, 0, 4, 5), bad);
    assert_eq!(s.nearest(0, 0, 0), bad);
    assert_eq!(s.nearest(0, 0, -1), bad);
}

#[test]
fn should_reject_duplicate_and_invalid_places() {
    let s = sut!(create());
    s.add_place("a", "A", "cafe", 0, 0, 40, 2).unwrap();

    assert_eq!(s.add_place("a", "Other", "bar", 1, 1, 10, 1), Err(SearchError::DuplicatePlace));
    let bad = Err(SearchError::InvalidArgument);
    assert_eq!(s.add_place(" ", "N", "cafe", 0, 0, 10, 1), bad);
    assert_eq!(s.add_place("x", "", "cafe", 0, 0, 10, 1), bad);
    assert_eq!(s.add_place("x", "N", " ", 0, 0, 10, 1), bad);
    assert_eq!(s.add_place("x", "N", "cafe", 0, 0, 51, 1), bad);
    assert_eq!(s.add_place("x", "N", "cafe", 0, 0, -1, 1), bad);
    assert_eq!(s.add_place("x", "N", "cafe", 0, 0, 10, 0), bad);
    assert_eq!(s.add_place("x", "N", "cafe", 0, 0, 10, 5), bad);

    assert_eq!(all(&s, None, 0, 0, 100, 10), ["a"]);
    assert_eq!(s.categories(), ["cafe:1"]);
}

#[test]
fn should_accept_the_boundary_ratings_and_price_levels() {
    let s = sut!(create());
    s.add_place("lo", "Lo", "cafe", 0, 0, 0, 1).unwrap();
    s.add_place("hi", "Hi", "cafe", 1, 0, 50, 4).unwrap();

    assert_eq!(all(&s, None, 0, 0, 5, 10), ["hi", "lo"]);
}

#[test]
fn should_update_a_place_by_replacing_all_its_attributes() {
    let s = sut!(create());
    s.add_place("a", "Old Name", "cafe", 50, 50, 10, 1).unwrap();
    s.add_place("b", "Rival", "cafe", 1, 0, 30, 2).unwrap();
    assert_eq!(all(&s, None, 0, 0, 200, 10), ["b", "a"]);

    s.update_place("a", "New Diner", "diner", 0, 1, 50, 3).unwrap();

    assert_eq!(all(&s, None, 0, 0, 200, 10), ["a", "b"]);
    assert!(all(&s, Some("old"), 0, 0, 200, 10).is_empty());
    assert_eq!(all(&s, Some("new"), 0, 0, 200, 10), ["a"]);
    assert_eq!(s.search(None, 0, 0, 200, Some("cafe"), 0, 4, 10).unwrap(), ["b"]);
    assert_eq!(s.categories(), ["cafe:1", "diner:1"]);
}

#[test]
fn should_reject_updates_and_removals_of_unknown_places_and_keep_state_on_failed_update() {
    let s = sut!(create());
    s.add_place("a", "Alpha", "cafe", 2, 0, 40, 2).unwrap();

    assert_eq!(s.update_place("zzz", "Z", "cafe", 0, 0, 10, 1), Err(SearchError::UnknownPlace));
    assert_eq!(s.remove_place("zzz"), Err(SearchError::UnknownPlace));
    assert_eq!(s.update_place("a", "Beta", "bar", 9, 9, 60, 2), Err(SearchError::InvalidArgument));
    assert_eq!(s.update_place("a", "Beta", "bar", 9, 9, 10, 9), Err(SearchError::InvalidArgument));

    assert_eq!(all(&s, Some("alpha"), 2, 0, 0, 5), ["a"]);
    assert_eq!(s.categories(), ["cafe:1"]);
}

#[test]
fn should_remove_places_and_allow_the_id_to_be_used_again() {
    let s = sut!(create());
    coffee_district(&s);

    s.remove_place("c3").unwrap();

    assert_eq!(all(&s, Some("coffee"), 0, 0, 100, 10), ["c1", "c2", "c4"]);
    assert_eq!(s.remove_place("c3"), Err(SearchError::UnknownPlace));
    s.add_place("c3", "Fresh Coffee", "cafe", 0, 1, 50, 1).unwrap();
    assert_eq!(all(&s, Some("coffee"), 0, 0, 100, 10)[0], "c3");
}

#[test]
fn should_return_the_nearest_places_ignoring_every_other_filter_with_id_tie_break() {
    let s = sut!(create());
    s.add_place("z", "Z", "cafe", 0, 2, 0, 4).unwrap();
    s.add_place("m", "M", "bar", 2, 0, 50, 1).unwrap();
    s.add_place("a", "A", "bar", 1, 1, 10, 3).unwrap();
    s.add_place("far", "Far", "cafe", 9, 9, 50, 1).unwrap();
    s.add_place("here", "Here", "cafe", 0, 0, 0, 4).unwrap();

    assert_eq!(s.nearest(0, 0, 4).unwrap(), ["here", "a", "m", "z"]);
    assert_eq!(s.nearest(0, 0, 100).unwrap(), ["here", "a", "m", "z", "far"]);
    assert_eq!(s.nearest(9, 9, 1).unwrap(), ["far"]);
}

#[test]
fn should_list_categories_with_counts_sorted_and_merged_ignoring_case() {
    let s = sut!(create());
    s.add_place("1", "A", "Cafe", 0, 0, 10, 1).unwrap();
    s.add_place("2", "B", "cafe", 0, 1, 10, 1).unwrap();
    s.add_place("3", "C", "BAR", 0, 2, 10, 1).unwrap();
    s.add_place("4", "D", "pizza", 0, 3, 10, 1).unwrap();
    assert_eq!(s.categories(), ["bar:1", "cafe:2", "pizza:1"]);

    s.update_place("2", "B", "pizza", 0, 1, 10, 1).unwrap();
    s.remove_place("3").unwrap();
    assert_eq!(s.categories(), ["cafe:1", "pizza:2"]);
}

#[test]
fn should_never_show_a_half_updated_place_to_concurrent_searches() {
    let s = sut!(create());
    s.add_place("p", "Alpha", "cafe", 0, 0, 50, 1).unwrap();
    let stop = AtomicBool::new(false);
    let torn = AtomicUsize::new(0);

    run_concurrently(5, |t| {
        if t == 0 {
            for i in 0..20_000 {
                if i % 2 == 0 {
                    s.update_place("p", "Beta", "bar", 100, 100, 0, 4).unwrap();
                } else {
                    s.update_place("p", "Alpha", "cafe", 0, 0, 50, 1).unwrap();
                }
            }
            stop.store(true, Ordering::SeqCst);
            return;
        }
        while !stop.load(Ordering::SeqCst) {
            // each of these asks for a mix of old and new values: it must never match
            let mixed = [
                s.search(Some("alpha"), 0, 0, 1000, Some("bar"), 0, 4, 5),
                s.search(Some("beta"), 0, 0, 1000, Some("cafe"), 0, 4, 5),
                s.search(Some("alpha"), 100, 100, 0, None, 0, 4, 5),
                s.search(Some("beta"), 0, 0, 0, None, 0, 4, 5),
                s.search(Some("beta"), 0, 0, 1000, None, 1, 4, 5),
                s.search(Some("beta"), 0, 0, 1000, None, 0, 3, 5),
            ];
            if mixed.iter().any(|r| !r.as_ref().unwrap().is_empty()) {
                torn.fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    assert_eq!(torn.load(Ordering::SeqCst), 0);
    assert_eq!(s.nearest(0, 0, 5).unwrap(), ["p"]);
}

#[test]
fn should_lose_no_place_when_many_threads_add_at_once() {
    let s = sut!(create());

    run_concurrently(16, |t| {
        for i in 0..250 {
            s.add_place(&format!("t{t}-{i}"), &format!("Place {i}"), "cafe", t as i32, i, 20, 2).unwrap();
            s.search(Some("place"), 0, 0, 10_000, None, 0, 4, 5).unwrap();
        }
    });

    assert_eq!(s.nearest(0, 0, 10_000).unwrap().len(), 4_000);
    assert_eq!(s.categories(), ["cafe:4000"]);
}

#[test]
fn should_answer_two_thousand_searches_over_fifty_thousand_places_within_thirty_seconds() {
    let s = sut!(create());
    let cats = ["cafe", "pizza", "bar", "sushi", "bakery", "gym", "books", "pharmacy"];
    let words = ["Golden", "Blue", "Happy", "Urban", "Old", "Royal", "Green", "Spicy"];
    let mut seed: u64 = 12345;
    let mut next = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((seed >> 33) % 2_000) as i32
    };
    for i in 0..50_000usize {
        let (a, b) = (next(), next());
        let cat = cats[(i / 8) % 8];
        s.add_place(&format!("p{i}"), &format!("{} {} {}", words[i % 8], cat, i), cat, a, b, ((i * 7) % 51) as i32, 1 + (i % 4) as i32)
            .unwrap();
    }

    let start = Instant::now();
    let mut non_empty = 0;
    for q in 0..2_000i32 {
        let query = match q % 3 {
            0 => Some("golden"),
            1 => Some("pizza"),
            _ => None,
        };
        let cat = if q % 5 == 0 { Some("cafe") } else { None };
        let r = s.search(query, (q * 37) % 2_000, (q * 91) % 2_000, 300 + q % 400, cat, 10, 4, 10).unwrap();
        assert!(r.len() <= 10);
        if !r.is_empty() {
            non_empty += 1;
        }
    }

    assert!(non_empty > 1_500, "only {non_empty} searches found anything");
    assert!(start.elapsed().as_secs() < 30, "2,000 searches over 50,000 places took {:?}", start.elapsed());
    assert_eq!(s.categories().len(), 8);
}
