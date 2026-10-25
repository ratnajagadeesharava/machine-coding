#![allow(unused_imports, clippy::needless_range_loop)]
use p131_landmark_distances::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::Arc;

fn add_all(m: &Arc<dyn CityMap>, names: &[&str]) {
    for n in names {
        m.add_location(n).unwrap();
    }
}

/// A-B 4, A-C 1, C-B 2, B-D 5, C-D 8 ; E isolated.
fn town(m: &Arc<dyn CityMap>) {
    add_all(m, &["A", "B", "C", "D", "E"]);
    m.add_road("A", "B", 4).unwrap();
    m.add_road("A", "C", 1).unwrap();
    m.add_road("C", "B", 2).unwrap();
    m.add_road("B", "D", 5).unwrap();
    m.add_road("C", "D", 8).unwrap();
}

fn v(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

#[test]
fn should_reject_bad_location_names() {
    let m = sut!(create());
    m.add_location("A").unwrap();
    assert_eq!(m.add_location("A"), Err(MapError::DuplicateLocation));
    assert_eq!(m.add_location(" "), Err(MapError::InvalidArgument));
    assert_eq!(m.add_location(""), Err(MapError::InvalidArgument));
    m.add_location("a").unwrap(); // case-sensitive: a different location
    assert_eq!(m.within_radius("a", 10), Ok(v(&["a"])));
}

#[test]
fn should_reject_bad_roads() {
    let m = sut!(create());
    add_all(&m, &["A", "B"]);
    assert_eq!(m.add_road("A", "Z", 5), Err(MapError::UnknownLocation));
    assert_eq!(m.add_road("Z", "A", 5), Err(MapError::UnknownLocation));
    assert_eq!(m.add_road("A", "A", 5), Err(MapError::InvalidArgument));
    assert_eq!(m.add_road("A", "B", 0), Err(MapError::InvalidArgument));
    assert_eq!(m.add_road("A", "B", -3), Err(MapError::InvalidArgument));
    assert_eq!(m.shortest_distance("A", "B"), Ok(-1)); // nothing was added by the failed calls
}

#[test]
fn should_measure_distance_to_itself_as_zero() {
    let m = sut!(create());
    town(&m);
    assert_eq!(m.shortest_distance("E", "E"), Ok(0));
    assert_eq!(m.shortest_distance("A", "A"), Ok(0));
}

#[test]
fn should_find_the_shortest_distance_not_the_fewest_roads() {
    let m = sut!(create());
    town(&m);
    assert_eq!(m.shortest_distance("A", "D"), Ok(8)); // A-C-B-D, not A-B-D (9) or A-C-D (9)
    assert_eq!(m.shortest_distance("D", "A"), Ok(8)); // roads are undirected
    assert_eq!(m.shortest_distance("A", "B"), Ok(3));
}

#[test]
fn should_report_unreachable_as_minus_one_and_empty_path() {
    let m = sut!(create());
    town(&m);
    assert_eq!(m.shortest_distance("A", "E"), Ok(-1));
    assert_eq!(m.shortest_distance("E", "A"), Ok(-1));
    assert_eq!(m.shortest_path("A", "E"), Ok(vec![]));
}

#[test]
fn should_reject_unknown_locations_in_queries() {
    let m = sut!(create());
    town(&m);
    assert_eq!(m.shortest_distance("A", "Z"), Err(MapError::UnknownLocation));
    assert_eq!(m.shortest_distance("Z", "A"), Err(MapError::UnknownLocation));
    assert_eq!(m.shortest_path("A", "Z"), Err(MapError::UnknownLocation));
    assert_eq!(m.nearest_landmarks("Z", "cafe", 1), Err(MapError::UnknownLocation));
    assert_eq!(m.within_radius("Z", 5), Err(MapError::UnknownLocation));
    assert_eq!(m.remove_road("A", "Z"), Err(MapError::UnknownLocation));
    assert_eq!(m.tag_landmark("Z", "cafe"), Err(MapError::UnknownLocation));
    assert_eq!(m.untag_landmark("Z", "cafe"), Err(MapError::UnknownLocation));
}

#[test]
fn should_use_the_shortest_of_parallel_roads() {
    let m = sut!(create());
    add_all(&m, &["A", "B"]);
    m.add_road("A", "B", 10).unwrap();
    m.add_road("B", "A", 7).unwrap();
    m.add_road("A", "B", 9).unwrap();
    assert_eq!(m.shortest_distance("A", "B"), Ok(7));
}

#[test]
fn should_remove_every_road_between_a_pair_and_count_them() {
    let m = sut!(create());
    add_all(&m, &["A", "B", "C"]);
    m.add_road("A", "B", 10).unwrap();
    m.add_road("B", "A", 7).unwrap();
    m.add_road("B", "C", 1).unwrap();

    assert_eq!(m.remove_road("A", "B"), Ok(2));
    assert_eq!(m.shortest_distance("A", "B"), Ok(-1));
    assert_eq!(m.shortest_distance("B", "C"), Ok(1));
    assert_eq!(m.remove_road("B", "A"), Ok(0));
    assert_eq!(m.remove_road("A", "C"), Ok(0));
}

#[test]
fn should_see_road_changes_immediately() {
    let m = sut!(create());
    town(&m);
    assert_eq!(m.shortest_distance("A", "D"), Ok(8));

    m.add_road("A", "D", 3).unwrap();
    assert_eq!(m.shortest_distance("A", "D"), Ok(3));
    assert_eq!(m.shortest_path("A", "D"), Ok(v(&["A", "D"])));

    m.remove_road("A", "D").unwrap();
    assert_eq!(m.shortest_distance("A", "D"), Ok(8));

    m.remove_road("C", "B").unwrap();
    assert_eq!(m.shortest_distance("A", "D"), Ok(9));

    m.add_road("D", "E", 2).unwrap();
    assert_eq!(m.shortest_distance("A", "E"), Ok(11));
}

#[test]
fn should_handle_distances_beyond_int_range() {
    let m = sut!(create());
    add_all(&m, &["A", "B", "C", "D"]);
    m.add_road("A", "B", 2_000_000_000).unwrap();
    m.add_road("B", "C", 2_000_000_000).unwrap();
    m.add_road("C", "D", 2_000_000_000).unwrap();
    assert_eq!(m.shortest_distance("A", "D"), Ok(6_000_000_000));
    assert_eq!(m.shortest_path("A", "D"), Ok(v(&["A", "B", "C", "D"])));
    assert_eq!(m.within_radius("A", 5_999_999_999), Ok(v(&["A", "B", "C"])));
    assert_eq!(
        m.within_radius("A", 6_000_000_000),
        Ok(v(&["A", "B", "C", "D"]))
    );
}

#[test]
fn should_return_the_route_names_in_order() {
    let m = sut!(create());
    town(&m);
    assert_eq!(m.shortest_path("A", "D"), Ok(v(&["A", "C", "B", "D"])));
    assert_eq!(m.shortest_path("D", "A"), Ok(v(&["D", "B", "C", "A"])));
    assert_eq!(m.shortest_path("E", "E"), Ok(v(&["E"])));
}

#[test]
fn should_break_equal_length_routes_by_smallest_name_sequence() {
    let m = sut!(create());
    add_all(&m, &["S", "T", "X", "Y", "Z"]);
    for mid in ["Z", "Y", "X"] {
        m.add_road("S", mid, 1).unwrap();
    }
    for mid in ["Z", "Y", "X"] {
        m.add_road(mid, "T", 1).unwrap();
    }
    assert_eq!(m.shortest_path("S", "T"), Ok(v(&["S", "X", "T"])));
}

#[test]
fn should_compare_tie_routes_element_by_element_and_never_trade_length_for_names() {
    let m = sut!(create());
    add_all(&m, &["S", "T", "a", "b", "c", "d"]);
    // S-a-c-T and S-b-d-T both cost 3; S-a-T costs 5 and must lose despite having the smallest first name.
    m.add_road("S", "a", 1).unwrap();
    m.add_road("a", "c", 1).unwrap();
    m.add_road("c", "T", 1).unwrap();
    m.add_road("S", "b", 1).unwrap();
    m.add_road("b", "d", 1).unwrap();
    m.add_road("d", "T", 1).unwrap();
    m.add_road("a", "T", 4).unwrap();
    assert_eq!(m.shortest_path("S", "T"), Ok(v(&["S", "a", "c", "T"])));

    m.add_road("a", "d", 1).unwrap(); // S-a-d-T (3) ties too; "c" < "d" keeps S-a-c-T
    assert_eq!(m.shortest_path("S", "T"), Ok(v(&["S", "a", "c", "T"])));
    m.remove_road("a", "c").unwrap(); // S-a-d-T (3) and S-b-d-T (3) remain: "a" < "b"
    assert_eq!(m.shortest_path("S", "T"), Ok(v(&["S", "a", "d", "T"])));
}

#[test]
fn should_order_nearest_landmarks_by_distance_then_name() {
    let m = sut!(create());
    add_all(&m, &["S", "p", "q", "r", "s"]);
    m.add_road("S", "p", 5).unwrap();
    m.add_road("S", "q", 3).unwrap();
    m.add_road("S", "r", 5).unwrap();
    m.add_road("S", "s", 9).unwrap();
    for n in ["p", "q", "r", "s"] {
        m.tag_landmark(n, "cafe").unwrap();
    }
    assert_eq!(
        m.nearest_landmarks("S", "cafe", 10),
        Ok(v(&["q:3", "p:5", "r:5", "s:9"]))
    );
    assert_eq!(
        m.nearest_landmarks("S", "cafe", 3),
        Ok(v(&["q:3", "p:5", "r:5"]))
    );
    assert_eq!(m.nearest_landmarks("S", "cafe", 1), Ok(v(&["q:3"])));
}

#[test]
fn should_count_the_source_itself_at_distance_zero() {
    let m = sut!(create());
    town(&m);
    m.tag_landmark("A", "hospital").unwrap();
    m.tag_landmark("D", "hospital").unwrap();
    assert_eq!(
        m.nearest_landmarks("A", "hospital", 5),
        Ok(v(&["A:0", "D:8"]))
    );
    assert_eq!(m.nearest_landmarks("D", "hospital", 1), Ok(v(&["D:0"])));
}

#[test]
fn should_skip_unreachable_landmarks_and_unknown_types() {
    let m = sut!(create());
    town(&m);
    m.tag_landmark("E", "cafe").unwrap();
    assert_eq!(m.nearest_landmarks("A", "cafe", 3), Ok(vec![]));
    assert_eq!(m.nearest_landmarks("A", "museum", 3), Ok(vec![]));
    assert_eq!(m.nearest_landmarks("E", "cafe", 3), Ok(v(&["E:0"])));
}

#[test]
fn should_treat_types_as_case_insensitive_trimmed_and_allow_several_per_location() {
    let m = sut!(create());
    town(&m);
    m.tag_landmark("B", "  Cafe ").unwrap();
    m.tag_landmark("B", "HOSPITAL").unwrap();
    m.tag_landmark("B", "cafe").unwrap(); // harmless repeat
    assert_eq!(m.nearest_landmarks("A", "CAFE", 5), Ok(v(&["B:3"])));
    assert_eq!(m.nearest_landmarks("A", " hospital", 5), Ok(v(&["B:3"])));

    assert_eq!(m.untag_landmark("B", "Cafe"), Ok(true));
    assert_eq!(m.untag_landmark("B", "cafe"), Ok(false));
    assert_eq!(m.nearest_landmarks("A", "cafe", 5), Ok(vec![]));
    assert_eq!(m.nearest_landmarks("A", "hospital", 5), Ok(v(&["B:3"])));
}

#[test]
fn should_reject_bad_k_and_blank_types() {
    let m = sut!(create());
    town(&m);
    m.tag_landmark("B", "cafe").unwrap();
    assert_eq!(m.nearest_landmarks("A", "cafe", 0), Err(MapError::InvalidArgument));
    assert_eq!(m.nearest_landmarks("A", "cafe", -1), Err(MapError::InvalidArgument));
    assert_eq!(m.nearest_landmarks("A", " ", 1), Err(MapError::InvalidArgument));
    assert_eq!(m.tag_landmark("B", ""), Err(MapError::InvalidArgument));
    assert_eq!(m.untag_landmark("B", "  "), Err(MapError::InvalidArgument));
}

#[test]
fn should_see_tag_and_road_changes_in_nearest_landmarks_immediately() {
    let m = sut!(create());
    town(&m);
    m.tag_landmark("D", "cafe").unwrap();
    assert_eq!(m.nearest_landmarks("A", "cafe", 2), Ok(v(&["D:8"])));

    m.tag_landmark("C", "cafe").unwrap();
    assert_eq!(m.nearest_landmarks("A", "cafe", 2), Ok(v(&["C:1", "D:8"])));

    m.remove_road("A", "C").unwrap();
    assert_eq!(m.nearest_landmarks("A", "cafe", 2), Ok(v(&["C:6", "D:9"])));

    m.untag_landmark("C", "cafe").unwrap();
    m.add_road("A", "E", 1).unwrap();
    m.tag_landmark("E", "cafe").unwrap();
    assert_eq!(m.nearest_landmarks("A", "cafe", 2), Ok(v(&["E:1", "D:9"])));
}

#[test]
fn should_list_locations_within_radius_sorted_by_name_with_boundary_included() {
    let m = sut!(create());
    town(&m);
    assert_eq!(m.within_radius("A", 0), Ok(v(&["A"])));
    assert_eq!(m.within_radius("A", 2), Ok(v(&["A", "C"])));
    assert_eq!(m.within_radius("A", 3), Ok(v(&["A", "B", "C"])));
    assert_eq!(m.within_radius("A", 100), Ok(v(&["A", "B", "C", "D"])));
    assert_eq!(m.within_radius("E", 100), Ok(v(&["E"])));
    assert_eq!(m.within_radius("A", -1), Err(MapError::InvalidArgument));
}

#[test]
fn should_answer_three_thousand_queries_on_a_150_by_150_grid() {
    let n = 150usize;
    let m = sut!(create());
    for r in 0..n {
        for c in 0..n {
            m.add_location(&format!("{r},{c}")).unwrap();
        }
    }
    for r in 0..n {
        for c in 0..n {
            if c + 1 < n {
                m.add_road(&format!("{r},{c}"), &format!("{r},{}", c + 1), 7)
                    .unwrap();
            }
            if r + 1 < n {
                m.add_road(&format!("{r},{c}"), &format!("{},{c}", r + 1), 7)
                    .unwrap();
            }
        }
    }
    let start = std::time::Instant::now();
    let mut seed: u64 = 12345;
    for _ in 0..3_000 {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let r1 = ((seed >> 33) % n as u64) as i64;
        let c1 = ((seed >> 20) % n as u64) as i64;
        let r2 = ((seed >> 45) % n as u64) as i64;
        let c2 = ((seed >> 11) % n as u64) as i64;
        let expected = 7 * ((r1 - r2).abs() + (c1 - c2).abs());
        assert_eq!(
            m.shortest_distance(&format!("{r1},{c1}"), &format!("{r2},{c2}")),
            Ok(expected)
        );
    }
    assert!(
        start.elapsed().as_millis() < 90_000,
        "elapsed {:?}",
        start.elapsed()
    );
}

#[test]
fn should_give_consistent_answers_to_queries_running_during_updates() {
    let m = sut!(create());
    add_all(&m, &["A", "B", "M"]);
    m.add_road("A", "M", 2_500).unwrap();
    m.add_road("M", "B", 2_500).unwrap();
    m.tag_landmark("B", "cafe").unwrap();

    run_concurrently(5, |t| {
        if t == 0 {
            for d in (1..=1_000).rev() {
                m.add_road("A", "B", d).unwrap();
            }
            return;
        }
        let mut last = i64::MAX;
        for _ in 0..3_000 {
            let d = m.shortest_distance("A", "B").unwrap();
            assert!((1..=5_000).contains(&d), "distance {d}");
            assert!(d <= last, "only shorter roads are ever added");
            last = d;
            let path = m.shortest_path("A", "B").unwrap();
            assert_eq!(path.first().map(String::as_str), Some("A"));
            assert_eq!(path.last().map(String::as_str), Some("B"));
            assert_eq!(m.nearest_landmarks("A", "cafe", 1).unwrap().len(), 1);
        }
    });
    assert_eq!(m.shortest_distance("A", "B"), Ok(1));
    assert_eq!(m.shortest_path("A", "B"), Ok(v(&["A", "B"])));
}

#[test]
fn should_not_lose_concurrent_locations_roads_and_tags() {
    let m = sut!(create());
    m.add_location("hub").unwrap();
    let threads = 8usize;
    let per = 200usize;
    run_concurrently(threads, |t| {
        for i in 0..per {
            let name = format!("n{t}_{i}");
            m.add_location(&name).unwrap();
            m.add_road("hub", &name, (i + 1) as i64).unwrap();
            m.tag_landmark(&name, if i % 2 == 0 { "cafe" } else { "park" })
                .unwrap();
            m.shortest_distance("hub", &name).unwrap();
        }
    });
    assert_eq!(
        m.within_radius("hub", 1_000_000).unwrap().len(),
        threads * per + 1
    );
    assert_eq!(
        m.nearest_landmarks("hub", "cafe", 1_000_000).unwrap().len(),
        threads * per / 2
    );
    assert_eq!(m.shortest_distance("n3_7", "n5_9"), Ok(8 + 10));
}
