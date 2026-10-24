#![allow(unused_imports, clippy::needless_range_loop)]
use p138_page_visit_counter::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

const A: &str = "https://site.com/a";
const B: &str = "https://site.com/b";

fn num(line: &str, from_end: usize) -> u64 {
    line.rsplit(':').nth(from_end - 1).unwrap().parse().unwrap()
}

fn url_of(t: usize, i: usize) -> String {
    format!("https://site.com/p{}", (t * 31 + i * 7) % 50)
}

fn visitor_of(t: usize, i: usize) -> String {
    format!("v{}", (t * 131 + i * 17) % 1000)
}

/// Same page, varying spelling: exercises normalisation from many threads.
fn spelling(url: &str, i: usize) -> String {
    match i % 3 {
        0 => url.to_string(),
        1 => format!("{}/?n={}", url.replace("https://site.com", "HTTPS://SITE.com"), i),
        _ => format!("  {url}#f{i} "),
    }
}

fn check_pairs(snap: &HashMap<String, u64>, writers: usize) {
    for w in 0..writers {
        let x = snap.get(&format!("https://s.com/x{w}")).copied().unwrap_or(0);
        let y = snap.get(&format!("https://s.com/y{w}")).copied().unwrap_or(0);
        assert!(x >= y && x - y <= 1, "pair {w} x={x} y={y}");
    }
}

#[test]
fn should_count_total_visits_per_page_and_return_zero_for_unknown_pages() {
    let c = sut!(create());
    c.visit(A, "u1").unwrap();
    c.visit(A, "u2").unwrap();
    c.visit(B, "u1").unwrap();

    assert_eq!(c.total_visits(A), Ok(2));
    assert_eq!(c.total_visits(B), Ok(1));
    assert_eq!(c.total_visits("https://site.com/never"), Ok(0));
    assert_eq!(c.unique_visitors("https://site.com/never"), Ok(0));
    assert_eq!(c.visits_by("https://site.com/never", "u1"), Ok(0));
}

#[test]
fn should_count_unique_visitors_once_no_matter_how_often_they_return() {
    let c = sut!(create());
    for _ in 0..5 {
        c.visit(A, "u1").unwrap();
    }
    c.visit(A, "u2").unwrap();

    assert_eq!(c.total_visits(A), Ok(6));
    assert_eq!(c.unique_visitors(A), Ok(2));
    assert_eq!(c.visits_by(A, "u1"), Ok(5));
    assert_eq!(c.visits_by(A, "u2"), Ok(1));
    assert_eq!(c.visits_by(A, "u3"), Ok(0));
}

#[test]
fn should_keep_unique_visitors_per_page_independent() {
    let c = sut!(create());
    c.visit(A, "u1").unwrap();
    c.visit(B, "u1").unwrap();
    c.visit(B, "u2").unwrap();

    assert_eq!(c.unique_visitors(A), Ok(1));
    assert_eq!(c.unique_visitors(B), Ok(2));
    assert_eq!(c.visits_by(A, "u2"), Ok(0));
}

#[test]
fn should_treat_visitor_ids_as_case_sensitive_and_untrimmed() {
    let c = sut!(create());
    c.visit(A, "Bob").unwrap();
    c.visit(A, "bob").unwrap();
    c.visit(A, " bob").unwrap();

    assert_eq!(c.unique_visitors(A), Ok(3));
    assert_eq!(c.visits_by(A, "bob"), Ok(1));
    assert_eq!(c.visits_by(A, " bob"), Ok(1));
}

#[test]
fn should_normalise_scheme_host_query_fragment_and_whitespace_to_one_page() {
    let c = sut!(create());
    c.visit("https://example.com/about", "u1").unwrap();
    c.visit("  HTTPS://Example.COM/about  ", "u2").unwrap();
    c.visit("https://example.com/about?utm=1&x=2", "u3").unwrap();
    c.visit("https://example.com/about#team", "u4").unwrap();
    c.visit("https://example.com/about/?a=b#c", "u5").unwrap();

    assert_eq!(c.page_count(), 1);
    assert_eq!(c.total_visits("https://EXAMPLE.com/about"), Ok(5));
    assert_eq!(c.unique_visitors("https://example.com/about/"), Ok(5));
    assert_eq!(c.top_pages(5), Ok(vec!["https://example.com/about:5".to_string()]));
}

#[test]
fn should_keep_path_case_and_treat_scheme_port_and_path_as_different_pages() {
    let c = sut!(create());
    c.visit("https://example.com/About", "u").unwrap();
    c.visit("https://example.com/about", "u").unwrap();
    c.visit("http://example.com/about", "u").unwrap();
    c.visit("https://example.com:8443/about", "u").unwrap();
    c.visit("https://example.com/about/team", "u").unwrap();

    assert_eq!(c.page_count(), 5);
    assert_eq!(c.total_visits("https://example.com/About"), Ok(1));
}

#[test]
fn should_normalise_the_root_path_to_a_single_slash() {
    let c = sut!(create());
    c.visit("https://Example.com", "u1").unwrap();
    c.visit("https://example.com/", "u2").unwrap();
    c.visit("https://example.com//", "u3").unwrap();
    c.visit("https://example.com/?q=1", "u4").unwrap();

    assert_eq!(c.page_count(), 1);
    assert_eq!(c.top_pages(1), Ok(vec!["https://example.com/:4".to_string()]));
}

#[test]
fn should_strip_every_trailing_slash_of_a_non_root_path() {
    let c = sut!(create());
    c.visit("https://example.com/docs///", "u1").unwrap();
    c.visit("https://example.com/docs", "u2").unwrap();

    assert_eq!(c.top_pages(3), Ok(vec!["https://example.com/docs:2".to_string()]));
}

#[test]
fn should_accept_urls_without_a_scheme() {
    let c = sut!(create());
    c.visit("Example.com/Pricing/", "u1").unwrap();
    c.visit("example.com/Pricing?x=1", "u2").unwrap();

    assert_eq!(c.top_pages(3), Ok(vec!["example.com/Pricing:2".to_string()]));
}

#[test]
fn should_reject_blank_and_hostless_urls_and_blank_visitors() {
    let c = sut!(create());
    c.visit(A, "u1").unwrap();
    let bad = CounterError::InvalidArgument;

    assert_eq!(c.visit("   ", "u"), Err(bad));
    assert_eq!(c.visit("", "u").unwrap_err(), CounterError::InvalidArgument);
    assert!(c.visit("?a=1", "u").is_err());
    assert!(c.visit("https:///path", "u").is_err());
    assert!(c.visit(A, " ").is_err());
    assert!(c.visit(A, "").is_err());
    assert!(c.total_visits(" ").is_err());
    assert!(c.unique_visitors("").is_err());
    assert!(c.visits_by(" ", "u").is_err());
    assert!(c.visits_by(A, " ").is_err());
    assert!(c.top_visitors(" ", 1).is_err());
    assert!(c.reset(" ").is_err());

    assert_eq!(c.grand_total(), 1);
    assert_eq!(c.page_count(), 1);
}

#[test]
fn should_rank_top_pages_by_total_then_url() {
    let c = sut!(create());
    for i in 0..3 {
        c.visit("https://s.com/z", &format!("u{i}")).unwrap();
        c.visit("https://s.com/m", &format!("u{i}")).unwrap();
    }
    for i in 0..5 {
        c.visit("https://s.com/q", &format!("u{i}")).unwrap();
    }
    c.visit("https://s.com/a", "u0").unwrap();

    let all: Vec<String> = ["q:5", "m:3", "z:3", "a:1"]
        .iter()
        .map(|s| format!("https://s.com/{s}"))
        .collect();
    assert_eq!(c.top_pages(10), Ok(all.clone()));
    assert_eq!(c.top_pages(2), Ok(all[..2].to_vec()));
    assert_eq!(c.top_pages(1), Ok(all[..1].to_vec()));
}

#[test]
fn should_reject_zero_k_and_return_empty_lists_when_nothing_is_recorded() {
    let c = sut!(create());

    assert_eq!(c.top_pages(3), Ok(vec![]));
    assert_eq!(c.top_visitors(A, 3), Ok(vec![]));
    assert_eq!(c.top_pages(0), Err(CounterError::InvalidArgument));
    assert_eq!(c.top_visitors(A, 0), Err(CounterError::InvalidArgument));
}

#[test]
fn should_rank_top_visitors_by_count_then_id() {
    let c = sut!(create());
    for _ in 0..4 {
        c.visit(A, "zed").unwrap();
        c.visit(A, "amy").unwrap();
    }
    for _ in 0..7 {
        c.visit(A, "kim").unwrap();
    }
    c.visit(A, "bob").unwrap();
    c.visit(B, "other").unwrap();

    assert_eq!(
        c.top_visitors(A, 10),
        Ok(vec!["kim:7", "amy:4", "zed:4", "bob:1"].into_iter().map(String::from).collect())
    );
    assert_eq!(
        c.top_visitors("https://site.com/a/?x=1", 2),
        Ok(vec!["kim:7".to_string(), "amy:4".to_string()])
    );
    assert_eq!(c.top_visitors(B, 5), Ok(vec!["other:1".to_string()]));
}

#[test]
fn should_report_grand_total_and_page_count_across_pages() {
    let c = sut!(create());
    assert_eq!(c.grand_total(), 0);
    assert_eq!(c.page_count(), 0);

    c.visit(A, "u1").unwrap();
    c.visit(A, "u1").unwrap();
    c.visit(B, "u2").unwrap();

    assert_eq!(c.grand_total(), 3);
    assert_eq!(c.page_count(), 2);
}

#[test]
fn should_reset_one_page_and_report_whether_it_existed() {
    let c = sut!(create());
    c.visit(A, "u1").unwrap();
    c.visit(A, "u2").unwrap();
    c.visit(B, "u1").unwrap();

    assert_eq!(c.reset("https://site.com/a/?utm=1"), Ok(true));
    assert_eq!(c.reset(A), Ok(false));
    assert_eq!(c.reset("https://site.com/never"), Ok(false));

    assert_eq!(c.total_visits(A), Ok(0));
    assert_eq!(c.unique_visitors(A), Ok(0));
    assert_eq!(c.visits_by(A, "u1"), Ok(0));
    assert_eq!(c.total_visits(B), Ok(1));
    assert_eq!(c.page_count(), 1);
    assert_eq!(c.grand_total(), 1);
    assert_eq!(c.top_pages(5), Ok(vec![format!("{B}:1")]));
}

#[test]
fn should_start_a_page_from_scratch_after_it_was_reset() {
    let c = sut!(create());
    c.visit(A, "u1").unwrap();
    c.visit(A, "u1").unwrap();
    c.reset(A).unwrap();
    c.visit(A, "u1").unwrap();

    assert_eq!(c.total_visits(A), Ok(1));
    assert_eq!(c.unique_visitors(A), Ok(1));
    assert_eq!(c.visits_by(A, "u1"), Ok(1));
}

#[test]
fn should_reset_everything() {
    let c = sut!(create());
    c.visit(A, "u1").unwrap();
    c.visit(B, "u2").unwrap();
    c.reset_all();

    assert_eq!(c.page_count(), 0);
    assert_eq!(c.grand_total(), 0);
    assert_eq!(c.top_pages(3), Ok(vec![]));
    assert!(c.snapshot().is_empty());
    assert_eq!(c.reset(A), Ok(false));

    c.visit(A, "u9").unwrap();
    assert_eq!(c.grand_total(), 1);
}

#[test]
fn should_export_a_snapshot_sorted_by_url_with_total_and_unique() {
    let c = sut!(create());
    c.visit("https://s.com/b", "u1").unwrap();
    c.visit("https://s.com/b", "u1").unwrap();
    c.visit("https://s.com/a", "u1").unwrap();
    c.visit("https://s.com/a", "u2").unwrap();
    c.visit("https://s.com/a", "u3").unwrap();
    c.visit("https://s.com/a/x", "u3").unwrap();

    assert_eq!(
        c.snapshot(),
        vec!["https://s.com/a:3:3", "https://s.com/a/x:1:1", "https://s.com/b:2:1"]
    );
    assert_eq!(
        c.snapshot_detail(),
        vec![
            "https://s.com/a:u1:1",
            "https://s.com/a:u2:1",
            "https://s.com/a:u3:1",
            "https://s.com/a/x:u3:1",
            "https://s.com/b:u1:2"
        ]
    );
}

#[test]
fn should_not_change_a_snapshot_that_was_already_taken() {
    let c = sut!(create());
    c.visit(A, "u1").unwrap();
    let before = c.snapshot();
    let before_detail = c.snapshot_detail();
    c.visit(A, "u2").unwrap();
    c.visit(B, "u3").unwrap();
    c.reset(A).unwrap();

    assert_eq!(before, vec![format!("{A}:1:1")]);
    assert_eq!(before_detail, vec![format!("{A}:u1:1")]);
    assert_eq!(c.snapshot(), vec![format!("{B}:1:1")]);
}

#[test]
fn should_count_every_visit_exactly_under_sixteen_writing_threads() {
    let c = sut!(create());
    let threads = 16;
    let per_thread = 20_000;
    run_concurrently(threads, |t| {
        for i in 0..per_thread {
            c.visit(&spelling(&url_of(t, i), i), &visitor_of(t, i)).unwrap();
        }
    });

    let mut total: HashMap<String, u64> = HashMap::new();
    let mut unique: HashMap<String, HashSet<String>> = HashMap::new();
    let mut by: HashMap<(String, String), u64> = HashMap::new();
    for t in 0..threads {
        for i in 0..per_thread {
            *total.entry(url_of(t, i)).or_default() += 1;
            unique.entry(url_of(t, i)).or_default().insert(visitor_of(t, i));
            *by.entry((url_of(t, i), visitor_of(t, i))).or_default() += 1;
        }
    }
    assert_eq!(c.page_count(), total.len());
    assert_eq!(c.grand_total(), (threads * per_thread) as u64);
    for (u, n) in &total {
        assert_eq!(c.total_visits(u), Ok(*n), "{u}");
        assert_eq!(c.unique_visitors(u), Ok(unique[u].len() as u64), "{u}");
    }
    for ((u, v), n) in &by {
        assert_eq!(c.visits_by(u, v), Ok(*n), "{u} {v}");
    }
}

#[test]
fn should_take_consistent_snapshots_while_sixteen_threads_keep_visiting() {
    let c = sut!(create());
    let writers = 12;
    // Writer w alternates "x<w>" then "y<w>": in any serial prefix 0 <= x - y <= 1.
    let seen = collect_concurrently(writers + 4, |t| {
        let mut snaps: Vec<Vec<String>> = vec![];
        if t < writers {
            for i in 0..20_000 {
                c.visit(&format!("https://s.com/x{t}"), &format!("v{}", i % 50)).unwrap();
                c.visit(&format!("https://s.com/y{t}"), &format!("v{}", i % 50)).unwrap();
            }
            return snaps;
        }
        let mut last = 0;
        for _ in 0..150 {
            let snap = c.snapshot();
            let mut sum = 0;
            for line in &snap {
                assert!(num(line, 2) >= num(line, 1), "{line}");
                sum += num(line, 2);
            }
            assert!(sum >= last);
            last = sum;
            snaps.push(snap);
            std::thread::yield_now();
        }
        snaps
    });

    for snaps in &seen[writers..] {
        for snap in snaps {
            let m: HashMap<String, u64> = snap
                .iter()
                .map(|l| (l[..l.rfind(':').unwrap()].rsplit_once(':').unwrap().0.to_string(), num(l, 2)))
                .collect();
            check_pairs(&m, writers);
        }
    }
    assert_eq!(c.grand_total(), (writers * 40_000) as u64);
}

#[test]
fn should_give_detail_snapshots_whose_per_visitor_counts_add_up_while_visits_continue() {
    let c = sut!(create());
    run_concurrently(10, |t| {
        if t < 8 {
            for i in 0..20_000 {
                c.visit(&format!("https://s.com/x{t}"), &format!("v{}", i % 40)).unwrap();
                c.visit(&format!("https://s.com/y{t}"), &format!("v{}", i % 40)).unwrap();
            }
            return;
        }
        for _ in 0..100 {
            let mut sums: HashMap<String, u64> = HashMap::new();
            for line in c.snapshot_detail() {
                let no_count = &line[..line.rfind(':').unwrap()];
                let url = &no_count[..no_count.rfind(':').unwrap()];
                *sums.entry(url.to_string()).or_default() += num(&line, 1);
            }
            check_pairs(&sums, 8);
        }
    });

    for w in 0..8 {
        assert_eq!(c.total_visits(&format!("https://s.com/x{w}")), Ok(20_000));
        assert_eq!(c.unique_visitors(&format!("https://s.com/y{w}")), Ok(40));
    }
}

#[test]
fn should_handle_one_million_visits_from_a_hundred_visitors_on_one_page() {
    let c = sut!(create());
    for i in 0..1_000_000 {
        c.visit(A, &format!("visitor-{}", i % 100)).unwrap();
    }

    assert_eq!(c.total_visits(A), Ok(1_000_000));
    assert_eq!(c.unique_visitors(A), Ok(100));
    assert_eq!(c.visits_by(A, "visitor-7"), Ok(10_000));
    assert_eq!(
        c.top_visitors(A, 2),
        Ok(vec!["visitor-0:10000".to_string(), "visitor-1:10000".to_string()])
    );
}

#[test]
fn should_not_deadlock_or_lose_counts_when_readers_resetters_and_writers_mix() {
    let c = sut!(create());
    run_concurrently(16, |t| {
        for i in 0..5_000 {
            let page = format!("https://s.com/p{}", i % 10);
            match t % 4 {
                0 => c.visit(&page, &format!("v{}", i % 20)).unwrap(),
                1 => {
                    c.top_pages(3).unwrap();
                    c.grand_total();
                    c.snapshot();
                }
                2 => {
                    c.top_visitors(&page, 3).unwrap();
                    c.unique_visitors(&page).unwrap();
                    c.snapshot_detail();
                }
                _ => {
                    if i % 50 == 0 {
                        c.reset(&page).unwrap();
                    }
                }
            }
        }
    });

    let sum: u64 = c.snapshot().iter().map(|l| num(l, 2)).sum();
    assert_eq!(c.grand_total(), sum);
    c.visit(A, "after").unwrap();
    assert_eq!(c.total_visits(A), Ok(1));
}
