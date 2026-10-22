#![allow(unused_imports, clippy::needless_range_loop)]
use p118_search_result_cache::{contract::*, create};
use playground_support::{eventually, run_concurrently, sut, FakeClock};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

type Handler = Arc<dyn Fn(&str) -> Result<Vec<String>, SearchError> + Send + Sync>;

/// Counting fake backend: records every query, the version is bumped by the tests, behaviour is swappable.
struct FakeBackend {
    version: AtomicU64,
    queries: Mutex<Vec<String>>,
    handler: Mutex<Handler>,
}

impl FakeBackend {
    fn new() -> Arc<Self> {
        Arc::new(FakeBackend {
            version: AtomicU64::new(1),
            queries: Mutex::new(Vec::new()),
            handler: Mutex::new(Arc::new(|q: &str| Ok(vec![format!("{q}#1"), format!("{q}#2")]))),
        })
    }
    fn set(&self, f: impl Fn(&str) -> Result<Vec<String>, SearchError> + Send + Sync + 'static) {
        *self.handler.lock().unwrap() = Arc::new(f);
    }
    fn bump(&self) {
        self.version.fetch_add(1, Ordering::SeqCst);
    }
    fn calls(&self) -> usize {
        self.queries.lock().unwrap().len()
    }
    fn queries(&self) -> Vec<String> {
        self.queries.lock().unwrap().clone()
    }
}

impl SearchBackend for FakeBackend {
    fn search(&self, q: &str) -> Result<Vec<String>, SearchError> {
        self.queries.lock().unwrap().push(q.to_string());
        let h = self.handler.lock().unwrap().clone();
        h(q)
    }
    fn index_version(&self) -> u64 {
        self.version.load(Ordering::SeqCst)
    }
}

/// Lets a backend call block until the test releases it.
struct Gate {
    state: Mutex<(bool, bool)>, // (entered, released)
    cv: Condvar,
}

impl Gate {
    fn new() -> Arc<Self> {
        Arc::new(Gate { state: Mutex::new((false, false)), cv: Condvar::new() })
    }
    fn pass(&self) {
        let mut s = self.state.lock().unwrap();
        s.0 = true;
        self.cv.notify_all();
        let (_g, timeout) = self.cv.wait_timeout_while(s, Duration::from_secs(20), |s| !s.1).unwrap();
        assert!(!timeout.timed_out(), "gate never released");
    }
    fn wait_entered(&self) -> bool {
        let s = self.state.lock().unwrap();
        let (s, _) = self.cv.wait_timeout_while(s, Duration::from_secs(10), |s| !s.0).unwrap();
        s.0
    }
    fn release(&self) {
        self.state.lock().unwrap().1 = true;
        self.cv.notify_all();
    }
}

fn ok(items: &[&str]) -> Result<Vec<String>, SearchError> {
    Ok(items.iter().map(|s| s.to_string()).collect())
}

fn strs(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn setup() -> (Arc<FakeBackend>, FakeClock) {
    (FakeBackend::new(), FakeClock::new(5_000))
}

macro_rules! cache {
    ($b:expr, $clock:expr, $cap:expr, $ttl:expr) => {
        sut!(create($b.clone(), $cap, $ttl, $clock.as_clock()))
    };
}

#[test]
fn should_tag_an_entry_with_the_version_read_before_the_backend_call() {
    let (b, clock) = setup();
    let b2 = b.clone();
    b.set(move |_| {
        b2.bump(); // the index changes while the search runs
        ok(&["maybe-stale"])
    });
    let c = cache!(b, clock, 10, 60_000);

    assert_eq!(c.get("q"), Ok(strs(&["maybe-stale"])));

    assert_eq!(c.size(), 0);
    b.set(|_| ok(&["fresh"]));
    assert_eq!(c.get("q"), Ok(strs(&["fresh"])));
    assert_eq!(c.backend_calls(), 2);
    assert_eq!(c.get("q"), Ok(strs(&["fresh"])));
    assert_eq!(c.backend_calls(), 2);
}

#[test]
fn should_not_store_or_share_a_call_that_was_invalidated_while_in_flight() {
    let (b, clock) = setup();
    let gate = Gate::new();
    let g = gate.clone();
    let n = AtomicUsize::new(0);
    b.set(move |_| {
        if n.fetch_add(1, Ordering::SeqCst) == 0 {
            g.pass();
            ok(&["stale"])
        } else {
            ok(&["fresh"])
        }
    });
    let c = cache!(b, clock, 10, 60_000);

    std::thread::scope(|s| {
        let old = {
            let c = &c;
            s.spawn(move || c.get("q"))
        };
        assert!(gate.wait_entered());

        c.invalidate("Q").unwrap();
        let after = {
            let c = &c;
            s.spawn(move || c.get("q"))
        };
        assert!(eventually(10_000, || after.is_finished()));
        assert_eq!(after.join().unwrap(), Ok(strs(&["fresh"])));

        gate.release();
        assert_eq!(old.join().unwrap(), Ok(strs(&["stale"])));
    });

    assert_eq!(c.get("q"), Ok(strs(&["fresh"])));
    assert_eq!(c.backend_calls(), 2);
}

#[test]
fn should_call_the_backend_exactly_once_per_query_under_heavy_overlap() {
    let (b, clock) = setup();
    b.set(|q| {
        std::thread::sleep(Duration::from_millis(1));
        Ok(vec![format!("{q}#1")])
    });
    let c = cache!(b, clock, 1_000, 600_000);
    let threads = 32;
    let keys = 500;

    run_concurrently(threads, |_| {
        for k in 0..keys {
            let q = format!("Query  {k}");
            assert_eq!(c.get(&q), Ok(vec![format!("query {k}#1")]));
        }
    });

    assert_eq!(c.backend_calls(), keys as u64);
    assert_eq!(b.calls(), keys);
    assert_eq!(c.hits() + c.misses(), (threads * keys) as u64);
    assert_eq!(c.size(), keys);
}

#[test]
fn should_give_every_waiter_an_equal_error_to_the_one_the_backend_returned() {
    let (b, clock) = setup();
    let gate = Gate::new();
    let g = gate.clone();
    b.set(move |_| {
        g.pass();
        Err(SearchError("boom".into()))
    });
    let c = cache!(b, clock, 10, 60_000);

    std::thread::scope(|s| {
        let handles: Vec<_> = (0..4)
            .map(|_| {
                let c = &c;
                s.spawn(move || c.get("q"))
            })
            .collect();
        assert!(gate.wait_entered());
        assert!(eventually(10_000, || c.misses() == 4), "all callers arrived");
        gate.release();
        for h in handles {
            assert_eq!(h.join().unwrap(), Err(CacheError::Backend(SearchError("boom".into()))));
        }
    });
    assert_eq!(b.calls(), 1);
}

#[test]
fn should_keep_a_failing_query_from_affecting_others() {
    let (b, clock) = setup();
    b.set(|q| {
        if q == "bad" {
            Err(SearchError("bad query".into()))
        } else {
            Ok(vec![format!("{q}#1")])
        }
    });
    let c = cache!(b, clock, 10, 60_000);

    run_concurrently(8, |_| {
        for i in 0..200 {
            assert!(matches!(c.get("bad"), Err(CacheError::Backend(_))));
            let q = format!("good {}", i % 5);
            assert_eq!(c.get(&q), Ok(vec![format!("{q}#1")]));
        }
    });

    assert_eq!(c.size(), 5);
    b.set(|_| ok(&["recovered"]));
    assert_eq!(c.get("bad"), Ok(strs(&["recovered"])));
}

#[test]
fn should_order_eviction_by_use_and_not_by_age_or_ttl() {
    let (b, clock) = setup();
    let c = cache!(b, clock, 3, 10_000);
    for q in ["a", "b", "c", "a", "b"] {
        c.get(q).unwrap(); // recency now: c (oldest), a, b
    }
    c.get("d").unwrap(); // evicts c
    let calls = c.backend_calls();

    for q in ["a", "b", "d"] {
        c.get(q).unwrap();
    }
    assert_eq!(c.backend_calls(), calls);

    c.get("c").unwrap(); // reload of c evicts a (the least recently used)
    assert_eq!(c.backend_calls(), calls + 1);
    for q in ["b", "d", "c"] {
        c.get(q).unwrap();
    }
    assert_eq!(c.backend_calls(), calls + 1);
    c.get("a").unwrap();
    assert_eq!(c.backend_calls(), calls + 2);
}

#[test]
fn should_never_serve_a_result_older_than_the_index_version_seen_before_the_get() {
    let (b, clock) = setup();
    let b2 = b.clone();
    b.set(move |_| Ok(vec![format!("v{}", b2.version.load(Ordering::SeqCst))]));
    let c = cache!(b, clock, 5, 600_000);

    for i in 0..1_000 {
        b.bump();
        let expected = vec![format!("v{}", b.version.load(Ordering::SeqCst))];
        let q = format!("q{}", i % 3);
        assert_eq!(c.get(&q), Ok(expected.clone()));
        assert_eq!(c.get(&q), Ok(expected));
    }
}

#[test]
fn should_survive_a_tiny_capacity_under_many_distinct_queries() {
    let (b, clock) = setup();
    b.set(|q| Ok(vec![format!("{q}#1")]));
    let c = cache!(b, clock, 1, 1_000);

    run_concurrently(16, |t| {
        for i in 0..2_000 {
            let q = format!("k{}", (i + t) % 40);
            assert_eq!(c.get(&q), Ok(vec![format!("{q}#1")]));
            assert!(c.size() <= 1);
            if i % 500 == 0 {
                clock.advance(300);
                b.bump();
            }
        }
    });

    assert_eq!(c.hits() + c.misses(), 16 * 2_000);
}

#[test]
fn should_lower_case_non_ascii_queries() {
    let (b, clock) = setup();
    let c = cache!(b, clock, 10, 60_000);

    c.get("  \u{c9}COLE   Primaire ").unwrap();
    c.get("\u{e9}cole primaire").unwrap();

    assert_eq!(b.queries(), strs(&["\u{e9}cole primaire"]));
    assert_eq!(c.hits(), 1);
}
