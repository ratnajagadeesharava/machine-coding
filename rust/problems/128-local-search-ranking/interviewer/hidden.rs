#![allow(unused_imports, clippy::needless_range_loop, clippy::too_many_arguments)]
use p128_local_search_ranking::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone)]
struct P {
    id: String,
    name: String,
    cat: String,
    x: i32,
    y: i32,
    rating: i32,
    price: i32,
}

struct Lcg(u64);
impl Lcg {
    fn next(&mut self, bound: u64) -> i32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 33) % bound) as i32
    }
}

fn oracle(ps: &[P], q: Option<&str>, ux: i32, uy: i32, radius: i32, cat: Option<&str>, min_r: i32, max_p: i32, k: usize) -> Vec<String> {
    let query = q.filter(|q| !q.trim().is_empty()).map(|q| q.to_lowercase());
    let mut rs: Vec<(i64, i64, String)> = Vec::new();
    for p in ps {
        let d = (p.x as i64 - ux as i64).abs() + (p.y as i64 - uy as i64).abs();
        let name = p.name.to_lowercase();
        let c = p.cat.to_lowercase();
        if d > radius as i64 || p.rating < min_r || p.price > max_p {
            continue;
        }
        if let Some(cat) = cat {
            if !cat.trim().is_empty() && c != cat.to_lowercase() {
                continue;
            }
        }
        let mut score = p.rating as i64 * 10 - 3 * d;
        if let Some(q) = &query {
            if !name.contains(q.as_str()) && &c != q {
                continue;
            }
            if name.starts_with(q.as_str()) {
                score += 25;
            }
            if &name == q {
                score += 25;
            }
        }
        rs.push((-score, d, p.id.clone()));
    }
    rs.sort();
    rs.into_iter().take(k).map(|r| r.2).collect()
}

#[test]
fn should_agree_with_a_brute_force_oracle_on_random_searches() {
    let s = sut!(create());
    let mut rnd = Lcg(42);
    let words = ["coffee", "Coffee", "tea", "bar", "Bar", "pizza", "grill", "cafe"];
    let cats = ["cafe", "Bar", "pizza", "tea", "coffee"];
    let mut ps = Vec::new();
    for i in 0..3_000 {
        let mut name = words[rnd.next(8) as usize].to_string();
        if rnd.next(2) == 0 {
            name = format!("{name} {}", words[rnd.next(8) as usize]);
        }
        let p = P {
            id: format!("id{}-{i}", rnd.next(1_000_000)),
            name,
            cat: cats[rnd.next(5) as usize].to_string(),
            x: rnd.next(121) - 60,
            y: rnd.next(121) - 60,
            rating: rnd.next(51),
            price: 1 + rnd.next(4),
        };
        s.add_place(&p.id, &p.name, &p.cat, p.x, p.y, p.rating, p.price).unwrap();
        ps.push(p);
    }
    let queries = [None, Some(""), Some("coffee"), Some("COFFEE"), Some("bar"), Some("Tea"), Some("pizza g"), Some("cafe"), Some("gril"), Some("zzz"), Some("coffee coffee")];
    for _ in 0..400 {
        let query = queries[rnd.next(queries.len() as u64) as usize];
        let cat = if rnd.next(3) == 0 { Some(cats[rnd.next(5) as usize].to_uppercase()) } else { None };
        let (ux, uy, radius) = (rnd.next(161) - 80, rnd.next(161) - 80, rnd.next(150));
        let (min_r, max_p, k) = (rnd.next(30), 1 + rnd.next(4), 1 + rnd.next(40));
        let got = s.search(query, ux, uy, radius, cat.as_deref(), min_r, max_p, k).unwrap();
        let want = oracle(&ps, query, ux, uy, radius, cat.as_deref(), min_r, max_p, k as usize);
        assert_eq!(got, want, "query={query:?} cat={cat:?} u=({ux},{uy}) r={radius} minR={min_r} maxP={max_p} k={k}");
    }
}

#[test]
fn should_not_overflow_with_extreme_coordinates_and_radii() {
    let s = sut!(create());
    s.add_place("west", "West", "cafe", i32::MIN, 0, 50, 1).unwrap();
    s.add_place("east", "East", "cafe", i32::MAX, 0, 50, 1).unwrap();
    s.add_place("origin", "Origin", "cafe", 0, 0, 0, 1).unwrap();

    // origin is 2^31 away from i32::MIN: just beyond the largest radius
    assert_eq!(s.search(None, i32::MIN, 0, i32::MAX, None, 0, 4, 10).unwrap(), ["west"]);
    assert_eq!(s.search(None, i32::MAX, 0, i32::MAX, None, 0, 4, 10).unwrap(), ["east", "origin"]);
    assert_eq!(s.nearest(i32::MIN, i32::MIN, 3).unwrap(), ["west", "origin", "east"]);
}

#[test]
fn should_accept_an_enormous_k_without_preallocating_for_it() {
    let s = sut!(create());
    s.add_place("a", "A", "cafe", 1, 0, 10, 1).unwrap();
    s.add_place("b", "B", "cafe", 2, 0, 10, 1).unwrap();

    assert_eq!(s.search(None, 0, 0, 10, None, 0, 4, i32::MAX).unwrap(), ["a", "b"]);
    assert_eq!(s.nearest(0, 0, i32::MAX).unwrap(), ["a", "b"]);
}

#[test]
fn should_apply_bonuses_on_the_name_only_and_ignore_case() {
    let s = sut!(create());
    s.add_place("cart", "Coffee Cart", "coffee", 0, 0, 40, 1).unwrap(); // 425
    s.add_place("exact", "COFFEE", "bakery", 5, 0, 40, 1).unwrap(); // 435
    s.add_place("sub", "Best coffee", "bakery", 0, 0, 43, 1).unwrap(); // 430
    s.add_place("cat", "Beans", "COFFEE", 0, 0, 42, 1).unwrap(); // 420

    assert_eq!(
        s.search(Some("Coffee"), 0, 0, 50, None, 0, 4, 10).unwrap(),
        ["exact", "sub", "cart", "cat"]
    );
}

#[test]
fn should_keep_results_sorted_when_places_are_updated_and_removed_between_searches() {
    let s = sut!(create());
    for i in 0..200 {
        let cat = if i % 2 == 0 { "cafe" } else { "bar" };
        s.add_place(&format!("p{i:03}"), &format!("Spot {i}"), cat, i, 0, 25, 2).unwrap();
    }
    for i in (0..200).step_by(3) {
        s.remove_place(&format!("p{i:03}")).unwrap();
    }
    for i in (1..200).step_by(3) {
        s.update_place(&format!("p{i:03}"), &format!("Spot {i}"), "cafe", 0, i, 25, 2).unwrap();
    }

    assert_eq!(
        s.search(None, 0, 0, 1_000, None, 0, 4, 5).unwrap(),
        ["p001", "p002", "p004", "p005", "p007"]
    );
    assert_eq!(s.categories(), ["bar:33", "cafe:100"]);
    assert_eq!(s.nearest(0, 0, 3).unwrap(), ["p001", "p002", "p004"]);
}

#[test]
fn should_stay_consistent_while_searches_run_during_adds_updates_and_removes() {
    let s = sut!(create());
    for i in 0..500 {
        s.add_place(&format!("base{i}"), &format!("Base {i}"), "cafe", i, i, 25, 2).unwrap();
    }

    run_concurrently(8, |t| {
        if t < 3 {
            for i in 0..1_500 {
                let id = format!("w{t}-{i}");
                let name = format!("Writer {t}");
                s.add_place(&id, &name, "bar", i % 100, t as i32, 30, 1).unwrap();
                s.update_place(&id, &name, "bar", i % 100, t as i32 + 1, 31, 1).unwrap();
                if i % 2 == 0 {
                    s.remove_place(&id).unwrap();
                }
            }
            return;
        }
        for _ in 0..3_000 {
            let r = s.search(Some("base"), 0, 0, 10_000, Some("cafe"), 0, 4, 10).unwrap();
            assert_eq!(&r[..3], ["base0", "base1", "base2"]);
            assert_eq!(r.len(), 10);
            assert_eq!(s.nearest(0, 0, 3).unwrap().len(), 3);
        }
    });

    assert_eq!(s.categories(), ["bar:2250", "cafe:500"]);
    assert_eq!(s.search(Some("writer 1"), 1, 2, 0, Some("bar"), 0, 4, 1_000).unwrap().len(), 15);
}

#[test]
fn should_handle_ten_thousand_searches_over_one_hundred_thousand_places() {
    let s = sut!(create());
    let mut rnd = Lcg(7);
    for i in 0..100_000usize {
        s.add_place(&format!("p{i}"), &format!("Place {}", i % 977), &format!("cat{}", i % 13), rnd.next(4_000), rnd.next(4_000), rnd.next(51), 1 + (i % 4) as i32)
            .unwrap();
    }
    let start = Instant::now();
    for q in 0..10_000i32 {
        let query = if q % 2 == 0 { Some("place 9") } else { None };
        s.search(query, rnd.next(4_000), rnd.next(4_000), 200 + q % 300, None, 20, 3, 20).unwrap();
    }
    assert!(start.elapsed().as_secs() < 120, "took {:?}", start.elapsed());
}
