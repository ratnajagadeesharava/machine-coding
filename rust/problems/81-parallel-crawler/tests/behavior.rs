#![allow(unused_imports, clippy::needless_range_loop)]
use p81_parallel_crawler::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// A fake web: a link graph with optional latency and broken pages. Records how it is being used.
#[derive(Default)]
struct Web {
    graph: HashMap<String, Vec<String>>,
    broken: HashSet<String>,
    fetches: Mutex<HashMap<String, usize>>,
    total_fetches: AtomicUsize,
    running: AtomicUsize,
    max_running: AtomicUsize,
    latency_ms: u64,
}

impl Web {
    fn link(&mut self, from: &str, to: &[&str]) {
        self.graph
            .entry(from.to_string())
            .or_default()
            .extend(to.iter().map(|s| s.to_string()));
        for t in to {
            self.graph.entry(t.to_string()).or_default();
        }
    }

    fn reachable_from(&self, start: &str) -> HashSet<String> {
        let mut seen = HashSet::from([start.to_string()]);
        let mut todo = VecDeque::from([start.to_string()]);
        while let Some(u) = todo.pop_front() {
            if self.broken.contains(&u) {
                continue;
            }
            for v in self.graph.get(&u).into_iter().flatten() {
                if seen.insert(v.clone()) {
                    todo.push_back(v.clone());
                }
            }
        }
        seen
    }

    fn assert_each_fetched_once(&self, expected: &HashSet<String>) {
        let fetches = self.fetches.lock().unwrap();
        let fetched: HashSet<String> = fetches.keys().cloned().collect();
        assert!(&fetched == expected, "fetched pages differ: got {} expected {}", fetched.len(), expected.len());
        for (url, n) in fetches.iter() {
            assert_eq!(*n, 1, "fetches of {url}");
        }
    }
}

impl Fetcher for Web {
    fn links(&self, url: &str) -> Result<Vec<String>, FetchError> {
        *self.fetches.lock().unwrap().entry(url.to_string()).or_insert(0) += 1;
        self.total_fetches.fetch_add(1, Ordering::SeqCst);
        self.max_running
            .fetch_max(self.running.fetch_add(1, Ordering::SeqCst) + 1, Ordering::SeqCst);
        if self.latency_ms > 0 {
            thread::sleep(Duration::from_millis(self.latency_ms));
        }
        let result = if self.broken.contains(url) {
            Err(FetchError)
        } else {
            Ok(self.graph.get(url).cloned().unwrap_or_default())
        };
        self.running.fetch_sub(1, Ordering::SeqCst);
        result
    }
}

fn set(items: &[&str]) -> HashSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

#[test]
fn should_return_just_the_start_page_when_it_has_no_links() {
    let c = sut!(create());
    let mut web = Web::default();
    web.link("a", &[]);
    let web = Arc::new(web);

    let reached = c.crawl("a", 4, web.clone()).unwrap();

    assert_eq!(reached, set(&["a"]));
    web.assert_each_fetched_once(&set(&["a"]));
}

#[test]
fn should_follow_cycles_diamonds_self_links_and_duplicate_links() {
    let c = sut!(create());
    let mut web = Web::default();
    web.link("a", &["b", "c", "b", "a"]);
    web.link("b", &["d"]);
    web.link("c", &["d", "a"]);
    web.link("d", &["a", "e"]);
    web.link("lonely", &["a"]);
    let web = Arc::new(web);

    let reached = c.crawl("a", 3, web.clone()).unwrap();

    assert_eq!(reached, set(&["a", "b", "c", "d", "e"]));
    web.assert_each_fetched_once(&set(&["a", "b", "c", "d", "e"]));
}

#[test]
fn should_finish_a_long_chain_where_only_one_worker_is_busy_at_a_time() {
    let c = sut!(create());
    let mut web = Web { latency_ms: 1, ..Web::default() };
    let n = 200;
    for i in 0..n - 1 {
        web.link(&format!("p{i}"), &[&format!("p{}", i + 1)]);
    }
    let web = Arc::new(web);

    let reached = c.crawl("p0", 8, web.clone()).unwrap();

    assert_eq!(reached.len(), n);
    web.assert_each_fetched_once(&reached);
}

#[test]
fn should_crawl_a_binary_tree() {
    let c = sut!(create());
    let mut web = Web::default();
    let depth = 10;
    for i in 1..(1usize << depth) {
        web.link(&format!("n{i}"), &[&format!("n{}", 2 * i), &format!("n{}", 2 * i + 1)]);
    }
    let web = Arc::new(web);

    let reached = c.crawl("n1", 8, web.clone()).unwrap();

    assert_eq!(reached.len(), (1 << (depth + 1)) - 1);
    web.assert_each_fetched_once(&reached);
}

#[test]
fn should_fetch_each_page_once_in_a_heavily_linked_grid() {
    let c = sut!(create());
    let mut web = Web::default();
    let size = 30;
    for x in 0..size {
        for y in 0..size {
            web.link(
                &format!("{x}:{y}"),
                &[
                    &format!("{}:{y}", (x + 1) % size),
                    &format!("{}:{y}", (x + size - 1) % size),
                    &format!("{x}:{}", (y + 1) % size),
                    &format!("{x}:{}", (y + size - 1) % size),
                ],
            );
        }
    }
    let web = Arc::new(web);

    let reached = c.crawl("0:0", 16, web.clone()).unwrap();

    assert_eq!(reached.len(), size * size);
    web.assert_each_fetched_once(&reached);
}

#[test]
fn should_count_broken_pages_as_reached_and_keep_going() {
    let c = sut!(create());
    let mut web = Web::default();
    web.link("a", &["b", "c"]);
    web.link("b", &["d"]);
    web.link("c", &["d"]);
    web.link("d", &["e"]);
    web.link("x", &["y"]);
    web.broken.insert("b".into());
    web.broken.insert("d".into());
    let web = Arc::new(web);

    let reached = c.crawl("a", 4, web.clone()).unwrap();

    // d is reached through b and c; it is broken, so e stays unknown
    assert_eq!(reached, set(&["a", "b", "c", "d"]));
    web.assert_each_fetched_once(&set(&["a", "b", "c", "d"]));
}

#[test]
fn should_never_run_more_fetches_than_workers_but_use_them() {
    let c = sut!(create());
    let mut web = Web { latency_ms: 10, ..Web::default() };
    let leaves: Vec<String> = (0..100).map(|i| format!("leaf{i}")).collect();
    web.link("root", &leaves.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    let web = Arc::new(web);

    let reached = c.crawl("root", 4, web.clone()).unwrap();

    assert_eq!(reached.len(), 101);
    let max = web.max_running.load(Ordering::SeqCst);
    assert!(max <= 4, "more concurrent fetches than workers: {max}");
    assert!(max >= 2, "workers were not used in parallel: {max}");
}

#[test]
fn should_fetch_sequentially_with_one_worker() {
    let c = sut!(create());
    let mut web = Web { latency_ms: 2, ..Web::default() };
    web.link("root", &["a", "b", "c", "d"]);
    web.link("a", &["b"]);
    web.link("c", &["d"]);
    let web = Arc::new(web);

    let reached = c.crawl("root", 1, web.clone()).unwrap();

    assert_eq!(reached.len(), 5);
    assert_eq!(web.max_running.load(Ordering::SeqCst), 1);
}

#[test]
fn should_not_fetch_anything_after_crawl_returns() {
    let c = sut!(create());
    let mut web = Web { latency_ms: 3, ..Web::default() };
    for i in 0..60 {
        web.link("r", &[&format!("c{i}")]);
        web.link(&format!("c{i}"), &[&format!("c{}", (i + 1) % 60)]);
    }
    let web = Arc::new(web);

    c.crawl("r", 8, web.clone()).unwrap();
    let at_return = web.total_fetches.load(Ordering::SeqCst);
    thread::sleep(Duration::from_millis(150));

    assert_eq!(
        web.total_fetches.load(Ordering::SeqCst),
        at_return,
        "a worker kept fetching after crawl() returned"
    );
}

#[test]
fn should_reject_zero_workers() {
    let c = sut!(create());
    let mut web = Web::default();
    web.link("a", &[]);

    assert_eq!(c.crawl("a", 0, Arc::new(web)), Err(InvalidWorkers));
}

#[test]
fn should_allow_several_crawls_at_once() {
    let c = sut!(create());

    run_concurrently(4, |t| {
        let mut web = Web::default();
        for i in 0..100 {
            let me = format!("s{t}-{i}");
            web.link(&me, &[&format!("s{t}-{}", (i * 7 + 1) % 100), &format!("s{t}-{}", (i + 1) % 100)]);
        }
        let start = format!("s{t}-0");
        let expected = web.reachable_from(&start);
        let web = Arc::new(web);

        let reached = c.crawl(&start, 4, web.clone()).unwrap();

        assert_eq!(reached, expected);
        web.assert_each_fetched_once(&reached);
    });
}

#[test]
fn should_crawl_random_graphs_correctly_over_100_runs() {
    let c = sut!(create());
    for run in 0..100u64 {
        let mut s = (run + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let mut rnd = move |n: usize| {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            (s % n as u64) as usize
        };
        let nodes = 20 + rnd(200);
        let mut web = Web::default();
        for i in 0..nodes {
            let out: Vec<String> = (0..rnd(4)).map(|_| format!("u{}", rnd(nodes))).collect();
            web.link(&format!("u{i}"), &out.iter().map(|s| s.as_str()).collect::<Vec<_>>());
            web.graph.entry(format!("u{i}")).or_default();
            if rnd(15) == 0 {
                web.broken.insert(format!("u{i}"));
            }
        }
        let expected = web.reachable_from("u0");
        let workers = 1 + rnd(12);
        let web = Arc::new(web);

        let reached = c.crawl("u0", workers, web.clone()).unwrap();

        assert_eq!(reached, expected, "run {run}");
        web.assert_each_fetched_once(&expected);
    }
}
