#![allow(unused_imports, clippy::needless_range_loop)]
use p138_page_visit_counter::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::collections::{BTreeMap, HashMap};

fn num(line: &str, from_end: usize) -> u64 {
    line.rsplit(':').nth(from_end - 1).unwrap().parse().unwrap()
}

#[test]
fn should_not_grow_with_the_number_of_visits_when_visitors_are_few() {
    // 5,000,000 visits from 100 visitors; a record per visit would be visible in the memory of the process.
    let c = sut!(create());
    let ids: Vec<String> = (0..100).map(|i| format!("visitor-{i}")).collect();
    for i in 0..5_000_000usize {
        c.visit("https://s.com/hot", &ids[i % 100]).unwrap();
    }
    assert_eq!(c.total_visits("https://s.com/hot"), Ok(5_000_000));
    assert_eq!(c.unique_visitors("https://s.com/hot"), Ok(100));
    assert_eq!(c.visits_by("https://s.com/hot", "visitor-99"), Ok(50_000));
}

#[test]
fn should_fold_many_spellings_of_one_page_into_one_entry_from_all_threads() {
    let c = sut!(create());
    let spellings = [
        "https://Shop.com/cart",
        "HTTPS://shop.COM/cart/",
        "https://shop.com/cart?x=1",
        "https://shop.com/cart#top",
        "  https://shop.com/cart/?a#b  ",
        "https://SHOP.com/cart//",
    ];
    run_concurrently(16, |t| {
        for i in 0..20_000 {
            c.visit(spellings[(t + i) % spellings.len()], &format!("u{}", i % 64)).unwrap();
        }
    });

    assert_eq!(c.page_count(), 1);
    assert_eq!(c.total_visits("https://shop.com/cart"), Ok(320_000));
    assert_eq!(c.unique_visitors("https://shop.com/cart"), Ok(64));
    assert_eq!(c.snapshot(), vec!["https://shop.com/cart:320000:64"]);
}

#[test]
fn should_count_distinct_visitors_exactly_when_there_are_hundreds_of_thousands() {
    let c = sut!(create());
    run_concurrently(16, |t| {
        for i in 0..20_000 {
            c.visit("https://s.com/big", &format!("id-{}", t + 16 * i)).unwrap();
            c.visit("https://s.com/big", &format!("shared-{}", i % 10_000)).unwrap();
        }
    });

    assert_eq!(c.unique_visitors("https://s.com/big"), Ok(330_000));
    assert_eq!(c.total_visits("https://s.com/big"), Ok(640_000));
    assert_eq!(c.visits_by("https://s.com/big", "shared-5"), Ok(32));
    assert_eq!(c.visits_by("https://s.com/big", "id-17"), Ok(1));
}

#[test]
fn should_keep_snapshots_monotonic_and_serial_while_writers_run_triple_ordered_chains() {
    let c = sut!(create());
    // Each writer visits a, b, c in turn: any serial prefix has a >= b >= c >= a - 1.
    run_concurrently(14, |t| {
        if t < 8 {
            for i in 0..25_000 {
                for p in ["a", "b", "c"] {
                    c.visit(&format!("https://s.com/{p}{t}"), &format!("v{}", i % 7)).unwrap();
                }
            }
            return;
        }
        let mut last: HashMap<String, u64> = HashMap::new();
        for _ in 0..200 {
            let mut now: BTreeMap<String, u64> = BTreeMap::new();
            for line in c.snapshot() {
                let url = line[..line.rfind(':').unwrap()].rsplit_once(':').unwrap().0.to_string();
                now.insert(url, num(&line, 2));
            }
            for (u, v) in &now {
                assert!(*v >= last.get(u).copied().unwrap_or(0), "{u} went backwards");
            }
            for w in 0..8 {
                let g = |p: &str| now.get(&format!("https://s.com/{p}{w}")).copied().unwrap_or(0);
                let (a, b, cc) = (g("a"), g("b"), g("c"));
                assert!(a >= b && a - b <= 1, "a-b {w}: {a} {b}");
                assert!(b >= cc && b - cc <= 1, "b-c {w}: {b} {cc}");
            }
            last.extend(now);
        }
    });

    assert_eq!(c.grand_total(), 8 * 75_000);
}

#[test]
fn should_return_well_formed_rankings_while_counts_change() {
    let c = sut!(create());
    run_concurrently(16, |t| {
        if t < 12 {
            for i in 0..20_000 {
                c.visit(&format!("https://s.com/p{}", i % 30), &format!("v{}", (i + t) % 25)).unwrap();
            }
            return;
        }
        for _ in 0..300 {
            let top = c.top_pages(5).unwrap();
            assert!(top.len() <= 5);
            let mut prev = (u64::MAX, String::new());
            for line in &top {
                let total = num(line, 1);
                let url = line[..line.rfind(':').unwrap()].to_string();
                assert!(total <= prev.0);
                if total == prev.0 {
                    assert!(url > prev.1);
                }
                prev = (total, url);
            }
            let who = c.top_visitors("https://s.com/p3", 4).unwrap();
            assert!(who.len() <= 4);
            let mut p = u64::MAX;
            for line in &who {
                assert!(num(line, 1) <= p);
                p = num(line, 1);
            }
        }
    });

    assert_eq!(c.grand_total(), 240_000);
    assert_eq!(c.top_pages(100).unwrap().len(), 30);
    assert_eq!(c.top_pages(1).unwrap().len(), 1);
}

#[test]
fn should_keep_totals_equal_to_visitor_sums_after_concurrent_resets() {
    let c = sut!(create());
    run_concurrently(16, |t| {
        for i in 0..10_000 {
            if t % 4 == 3 {
                if i % 25 == 0 {
                    c.reset(&format!("https://s.com/p{}", i % 5)).unwrap();
                }
                if i % 1000 == 0 {
                    c.reset_all();
                }
            } else {
                c.visit(&format!("https://s.com/p{}", i % 5), &format!("v{}", i % 13)).unwrap();
            }
        }
    });

    let mut sums: BTreeMap<String, u64> = BTreeMap::new();
    for line in c.snapshot_detail() {
        let no_count = &line[..line.rfind(':').unwrap()];
        *sums.entry(no_count[..no_count.rfind(':').unwrap()].to_string()).or_default() += num(&line, 1);
    }
    let expected: Vec<String> = sums.iter().map(|(u, s)| format!("{u}:{s}")).collect();
    let totals: Vec<String> = c
        .snapshot()
        .iter()
        .map(|l| l[..l.rfind(':').unwrap()].to_string())
        .collect();
    assert_eq!(totals, expected);
    assert_eq!(c.grand_total(), sums.values().sum::<u64>());
}

#[test]
fn should_reset_and_revisit_concurrently_without_torn_counts() {
    let c = sut!(create());
    run_concurrently(8, |t| {
        for _ in 0..20_000 {
            if t == 0 {
                c.reset("https://s.com/hot").unwrap();
            } else {
                c.visit("https://s.com/hot", &format!("v{t}")).unwrap();
                c.total_visits("https://s.com/hot").unwrap();
                assert!(c.unique_visitors("https://s.com/hot").unwrap() <= 7);
            }
        }
    });

    c.reset("https://s.com/hot").unwrap();
    c.visit("https://s.com/hot", "last").unwrap();
    assert_eq!(c.snapshot(), vec!["https://s.com/hot:1:1"]);
}
