#![allow(unused_imports, clippy::needless_range_loop)]
use p131_landmark_distances::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::Arc;

fn v(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn cell(r: usize, c: usize) -> String {
    format!("r{r:03}c{c:03}")
}

fn grid(m: &Arc<dyn CityMap>, n: usize, w: i64) {
    for r in 0..n {
        for c in 0..n {
            m.add_location(&cell(r, c)).unwrap();
        }
    }
    for r in 0..n {
        for c in 0..n {
            if c + 1 < n {
                m.add_road(&cell(r, c), &cell(r, c + 1), w).unwrap();
            }
            if r + 1 < n {
                m.add_road(&cell(r, c), &cell(r + 1, c), w).unwrap();
            }
        }
    }
}

#[test]
fn should_pick_the_lexicographically_smallest_of_a_huge_number_of_equal_routes() {
    let n = 150;
    let m = sut!(create());
    grid(&m, n, 1);
    let mut expected: Vec<String> = (0..n).map(|c| cell(0, c)).collect(); // "r000c001" < "r001c000"
    expected.extend((1..n).map(|r| cell(r, n - 1)));
    assert_eq!(m.shortest_path(&cell(0, 0), &cell(n - 1, n - 1)), Ok(expected));
}

#[test]
fn should_mix_all_query_kinds_on_a_large_grid() {
    let n = 150usize;
    let m = sut!(create());
    grid(&m, n, 3);
    for i in (0..n).step_by(15) {
        m.tag_landmark(&cell(i, (i * 7) % n), "Cafe").unwrap();
    }
    let start = std::time::Instant::now();
    let mut seed: u64 = 99;
    for q in 0..3_000 {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let r1 = ((seed >> 33) % n as u64) as usize;
        let c1 = ((seed >> 20) % n as u64) as usize;
        let r2 = ((seed >> 45) % n as u64) as usize;
        let c2 = ((seed >> 11) % n as u64) as usize;
        let manhattan = r1.abs_diff(r2) + c1.abs_diff(c2);
        match q % 4 {
            0 => assert_eq!(
                m.shortest_distance(&cell(r1, c1), &cell(r2, c2)),
                Ok(3 * manhattan as i64)
            ),
            1 => assert_eq!(
                m.shortest_path(&cell(r1, c1), &cell(r2, c2)).unwrap().len(),
                manhattan + 1
            ),
            2 => assert_eq!(m.nearest_landmarks(&cell(r1, c1), "cafe", 3).unwrap().len(), 3),
            _ => assert!(!m.within_radius(&cell(r1, c1), 6).unwrap().is_empty()),
        }
    }
    assert!(start.elapsed().as_millis() < 120_000, "{:?}", start.elapsed());
}

#[test]
fn should_reflect_every_update_on_a_grid_with_no_stale_answers() {
    let n = 40usize;
    let m = sut!(create());
    grid(&m, n, 1);
    let from = cell(0, 0);
    let to = cell(0, n - 1);
    let straight = (n - 1) as i64;
    assert_eq!(m.shortest_distance(&from, &to), Ok(straight));

    m.remove_road(&cell(0, 20), &cell(0, 21)).unwrap();
    assert_eq!(m.shortest_distance(&from, &to), Ok(straight + 2));

    for r in 1..n - 1 {
        m.remove_road(&cell(r, 20), &cell(r, 21)).unwrap();
    }
    assert_eq!(m.shortest_distance(&from, &to), Ok(straight + 2 * straight));

    m.add_road(&cell(0, 20), &cell(0, 21), 1).unwrap();
    assert_eq!(m.shortest_distance(&from, &to), Ok(straight));
    assert_eq!(m.remove_road(&cell(0, 21), &cell(0, 20)), Ok(1));
    assert_eq!(m.shortest_distance(&from, &to), Ok(straight + 2 * straight));
}

#[test]
fn should_reconnect_separate_components_and_route_across_the_new_road() {
    let m = sut!(create());
    for s in ["a1", "a2", "b1", "b2"] {
        m.add_location(s).unwrap();
    }
    m.add_road("a1", "a2", 5).unwrap();
    m.add_road("b1", "b2", 5).unwrap();
    m.tag_landmark("b2", "park").unwrap();
    assert_eq!(m.nearest_landmarks("a1", "park", 1), Ok(vec![]));
    assert_eq!(m.shortest_path("a1", "b2"), Ok(vec![]));

    m.add_road("a2", "b1", 2).unwrap();
    assert_eq!(m.nearest_landmarks("a1", "park", 1), Ok(v(&["b2:12"])));
    assert_eq!(m.shortest_path("a1", "b2"), Ok(v(&["a1", "a2", "b1", "b2"])));
    assert_eq!(m.within_radius("a1", 11), Ok(v(&["a1", "a2", "b1"])));
}

#[test]
fn should_report_large_totals_in_landmark_results() {
    let m = sut!(create());
    for s in ["home", "mid", "far"] {
        m.add_location(s).unwrap();
    }
    m.add_road("home", "mid", 3_000_000_000).unwrap();
    m.add_road("mid", "far", 3_000_000_000).unwrap();
    m.tag_landmark("far", "Airport").unwrap();
    m.tag_landmark("mid", "airport").unwrap();
    assert_eq!(
        m.nearest_landmarks("home", "AIRPORT", 5),
        Ok(v(&["mid:3000000000", "far:6000000000"]))
    );
    assert_eq!(m.shortest_distance("far", "home"), Ok(6_000_000_000));
}

#[test]
fn should_break_a_thousand_way_distance_tie_by_name_and_compare_names_as_plain_strings() {
    let m = sut!(create());
    m.add_location("centre").unwrap();
    let mut names: Vec<String> = (0..1_000).map(|i| format!("shop{i}")).collect();
    names.push("Shop".to_string());
    for s in &names {
        m.add_location(s).unwrap();
        m.add_road("centre", s, 10).unwrap();
        m.tag_landmark(s, "shop").unwrap();
    }
    // plain byte order: "Shop" < "shop0" < "shop1" < "shop10" < "shop100" < "shop101" ...
    assert_eq!(
        m.nearest_landmarks("centre", "shop", 5),
        Ok(v(&["Shop:10", "shop0:10", "shop1:10", "shop10:10", "shop100:10"]))
    );
    let all = m.within_radius("centre", 10).unwrap();
    assert_eq!(all.len(), 1_002);
    assert_eq!(all[0], "Shop");
}

#[test]
fn should_keep_queries_consistent_while_tags_and_roads_flip_concurrently() {
    let m = sut!(create());
    for s in ["s", "x", "y"] {
        m.add_location(s).unwrap();
    }
    m.add_road("s", "x", 4).unwrap();
    m.add_road("s", "y", 9).unwrap();
    m.tag_landmark("s", "base").unwrap();

    run_concurrently(6, |t| {
        if t == 0 {
            for _ in 0..4_000 {
                m.tag_landmark("x", "cafe").unwrap();
                m.untag_landmark("x", "cafe").unwrap();
            }
            return;
        }
        if t == 1 {
            for _ in 0..4_000 {
                m.add_road("x", "y", 1).unwrap();
                m.remove_road("x", "y").unwrap();
            }
            return;
        }
        for _ in 0..2_000 {
            let cafes = m.nearest_landmarks("s", "cafe", 5).unwrap();
            assert!(cafes.is_empty() || cafes == v(&["x:4"]), "{cafes:?}");
            let d = m.shortest_distance("s", "y").unwrap();
            assert!(d == 9 || d == 5, "{d}");
            assert_eq!(m.nearest_landmarks("s", "base", 5), Ok(v(&["s:0"])));
            let near = m.within_radius("s", 4).unwrap();
            assert!(near.contains(&"s".to_string()) && near.contains(&"x".to_string()));
        }
    });
    assert_eq!(m.shortest_distance("s", "y"), Ok(9));
    assert_eq!(m.nearest_landmarks("s", "cafe", 1), Ok(vec![]));
}

#[test]
fn should_stay_correct_when_many_threads_grow_and_query_the_same_map() {
    let m = sut!(create());
    m.add_location("root").unwrap();
    run_concurrently(8, |t| {
        let mut prev = "root".to_string();
        for i in 0..300 {
            let cur = format!("t{t}-{i}");
            m.add_location(&cur).unwrap();
            m.add_road(&prev, &cur, 2).unwrap();
            prev = cur.clone();
            assert_eq!(m.shortest_distance("root", &cur), Ok(2 * (i as i64 + 1)));
        }
    });
    assert_eq!(m.shortest_distance("t0-299", "t7-299"), Ok(2 * 300 * 2));
    assert_eq!(
        m.shortest_path("t1-1", "t2-0"),
        Ok(v(&["t1-1", "t1-0", "root", "t2-0"]))
    );
}
