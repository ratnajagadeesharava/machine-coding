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
    (FakeBackend::new(), FakeClock::new(1_000_000))
}

macro_rules! cache {
    ($b:expr, $clock:expr, $cap:expr, $ttl:expr) => {
        sut!(create($b.clone(), $cap, $ttl, $clock.as_clock()))
    };
}

#[test]
fn should_fetch_from_the_backend_on_a_miss() {
    let (b, clock) = setup();
    let c = cache!(b, clock, 10, 1_000);

    assert_eq!(c.get("red shoes"), Ok(strs(&["red shoes#1", "red shoes#2"])));

    assert_eq!(c.hits(), 0);
    assert_eq!(c.misses(), 1);
    assert_eq!(c.backend_calls(), 1);
    assert_eq!(c.size(), 1);
}

#[test]
fn should_serve_the_second_identical_query_from_the_cache() {
    let (b, clock) = setup();
    let c = cache!(b, clock, 10, 1_000);
    let first = c.get("red shoes").unwrap();

    assert_eq!(c.get("red shoes"), Ok(first));

    assert_eq!(c.hits(), 1);
    assert_eq!(c.misses(), 1);
    assert_eq!(b.calls(), 1);
}

#[test]
fn should_treat_case_and_whitespace_variants_as_the_same_query() {
    let (b, clock) = setup();
    let c = cache!(b, clock, 10, 1_000);
    c.get("red shoes").unwrap();

    let expected = Ok(strs(&["red shoes#1", "red shoes#2"]));
    assert_eq!(c.get("Red  SHOES"), expected);
    assert_eq!(c.get("  red shoes\t"), expected);
    assert_eq!(c.get("RED\n \t shoes"), expected);

    assert_eq!(b.calls(), 1);
    assert_eq!(c.hits(), 3);
    assert_eq!(c.size(), 1);
}

#[test]
fn should_give_the_backend_the_normalised_query() {
    let (b, clock) = setup();
    let c = cache!(b, clock, 10, 1_000);

    c.get("  Red   SHOES ").unwrap();

    assert_eq!(b.queries(), strs(&["red shoes"]));
}

#[test]
fn should_treat_word_order_as_part_of_the_query() {
    let (b, clock) = setup();
    let c = cache!(b, clock, 10, 1_000);

    c.get("red shoes").unwrap();
    c.get("shoes red").unwrap();

    assert_eq!(b.queries(), strs(&["red shoes", "shoes red"]));
    assert_eq!(c.size(), 2);
}

#[test]
fn should_expire_an_entry_exactly_at_its_ttl() {
    let (b, clock) = setup();
    let c = cache!(b, clock, 10, 1_000);
    c.get("q").unwrap();

    clock.advance(999);
    c.get("q").unwrap();
    assert_eq!(b.calls(), 1);

    clock.advance(1);
    c.get("q").unwrap();
    assert_eq!(b.calls(), 2);
    assert_eq!(c.hits(), 1);
    assert_eq!(c.misses(), 2);
}

#[test]
fn should_not_extend_the_ttl_on_hits() {
    let (b, clock) = setup();
    let c = cache!(b, clock, 10, 1_000);
    c.get("q").unwrap();
    clock.advance(600);
    c.get("q").unwrap();
    clock.advance(400);

    c.get("q").unwrap();

    assert_eq!(b.calls(), 2);
}

#[test]
fn should_give_a_reloaded_entry_a_full_new_ttl() {
    let (b, clock) = setup();
    let c = cache!(b, clock, 10, 1_000);
    c.get("q").unwrap();
    clock.advance(1_000);
    c.get("q").unwrap();
    clock.advance(999);

    c.get("q").unwrap();

    assert_eq!(b.calls(), 2);
}

#[test]
fn should_evict_the_least_recently_used_entry_when_over_capacity() {
    let (b, clock) = setup();
    let c = cache!(b, clock, 2, 60_000);
    c.get("a").unwrap();
    c.get("b").unwrap();
    c.get("a").unwrap(); // a is now more recent than b
    c.get("c").unwrap(); // evicts b

    assert_eq!(c.size(), 2);
    let before = c.backend_calls();
    c.get("a").unwrap();
    c.get("c").unwrap();
    assert_eq!(c.backend_calls(), before);
    c.get("b").unwrap();
    assert_eq!(c.backend_calls(), before + 1);
}

#[test]
fn should_never_hold_more_entries_than_the_capacity() {
    let (b, clock) = setup();
    let c = cache!(b, clock, 5, 60_000);

    for i in 0..100 {
        c.get(&format!("query {i}")).unwrap();
        assert!(c.size() <= 5);
    }
    assert_eq!(c.size(), 5);
}

#[test]
fn should_drop_entries_when_the_index_version_changes() {
    let (b, clock) = setup();
    let c = cache!(b, clock, 10, 60_000);
    c.get("a").unwrap();
    c.get("b").unwrap();

    b.bump();
    assert_eq!(c.size(), 0);
    c.get("a").unwrap();

    assert_eq!(b.calls(), 3);
    assert_eq!(c.size(), 1);
    c.get("a").unwrap();
    assert_eq!(b.calls(), 3);
    c.get("b").unwrap();
    assert_eq!(b.calls(), 4);
}

#[test]
fn should_return_fresh_results_after_the_index_changed() {
    let (b, clock) = setup();
    let c = cache!(b, clock, 10, 60_000);
    b.set(|_| ok(&["old"]));
    assert_eq!(c.get("q"), Ok(strs(&["old"])));

    b.set(|_| ok(&["new"]));
    assert_eq!(c.get("q"), Ok(strs(&["old"])));
    b.bump();

    assert_eq!(c.get("q"), Ok(strs(&["new"])));
}

#[test]
fn should_invalidate_one_query_by_its_normalised_form() {
    let (b, clock) = setup();
    let c = cache!(b, clock, 10, 60_000);
    c.get("red shoes").unwrap();
    c.get("blue hat").unwrap();

    c.invalidate("  RED   Shoes ").unwrap();

    assert_eq!(c.size(), 1);
    c.get("blue hat").unwrap();
    assert_eq!(b.calls(), 2);
    c.get("red shoes").unwrap();
    assert_eq!(b.calls(), 3);
}

#[test]
fn should_invalidate_everything() {
    let (b, clock) = setup();
    let c = cache!(b, clock, 10, 60_000);
    c.get("a").unwrap();
    c.get("b").unwrap();

    c.invalidate_all();

    assert_eq!(c.size(), 0);
    c.get("a").unwrap();
    c.get("b").unwrap();
    assert_eq!(b.calls(), 4);
}

#[test]
fn should_accept_invalidate_of_a_query_that_is_not_cached() {
    let (b, clock) = setup();
    let c = cache!(b, clock, 10, 60_000);

    c.invalidate("never seen").unwrap();
    c.invalidate_all();

    assert_eq!(c.size(), 0);
    assert_eq!(b.calls(), 0);
}

#[test]
fn should_not_count_expired_entries_in_the_size() {
    let (b, clock) = setup();
    let c = cache!(b, clock, 10, 1_000);
    c.get("a").unwrap();
    clock.advance(600);
    c.get("b").unwrap();
    assert_eq!(c.size(), 2);

    clock.advance(400);

    assert_eq!(c.size(), 1);
}

#[test]
fn should_reject_blank_queries_without_counting_them() {
    let (b, clock) = setup();
    let c = cache!(b, clock, 10, 1_000);

    assert_eq!(c.get(""), Err(CacheError::InvalidQuery));
    assert_eq!(c.get(" \t\n "), Err(CacheError::InvalidQuery));
    assert_eq!(c.invalidate("  "), Err(CacheError::InvalidQuery));

    assert_eq!(c.hits() + c.misses() + c.backend_calls(), 0);
    assert_eq!(b.calls(), 0);
}

#[test]
fn should_reject_invalid_configuration() {
    let (b, clock) = setup();
    let _valid = cache!(b, clock, 10, 1_000); // proves the factory is wired

    assert!(create(b.clone(), 0, 1_000, clock.as_clock()).is_none());
    assert!(create(b.clone(), 10, 0, clock.as_clock()).is_none());
}

#[test]
fn should_not_let_callers_corrupt_cached_results() {
    let (b, clock) = setup();
    let c = cache!(b, clock, 10, 1_000);
    let mut miss = c.get("q").unwrap();
    miss.push("x".to_string());
    miss[0] = "changed".to_string();
    let mut hit = c.get("q").unwrap();
    hit.clear();

    assert_eq!(c.get("q"), Ok(strs(&["q#1", "q#2"])));
}

#[test]
fn should_cache_an_empty_result() {
    let (b, clock) = setup();
    b.set(|_| Ok(Vec::new()));
    let c = cache!(b, clock, 10, 1_000);

    assert_eq!(c.get("nothing matches"), Ok(Vec::new()));
    assert_eq!(c.get("nothing matches"), Ok(Vec::new()));

    assert_eq!(b.calls(), 1);
    assert_eq!(c.hits(), 1);
}

#[test]
fn should_propagate_a_backend_failure_and_not_cache_it() {
    let (b, clock) = setup();
    let fail = Arc::new(AtomicBool::new(true));
    let f = fail.clone();
    b.set(move |_| {
        if f.load(Ordering::SeqCst) {
            Err(SearchError("backend down".into()))
        } else {
            ok(&["ok"])
        }
    });
    let c = cache!(b, clock, 10, 1_000);

    assert_eq!(c.get("q"), Err(CacheError::Backend(SearchError("backend down".into()))));
    assert_eq!(c.size(), 0);
    assert_eq!(c.misses(), 1);
    assert_eq!(c.backend_calls(), 1);

    assert!(matches!(c.get("q"), Err(CacheError::Backend(_))));
    assert_eq!(c.backend_calls(), 2);

    fail.store(false, Ordering::SeqCst);
    assert_eq!(c.get("q"), Ok(strs(&["ok"])));
    assert_eq!(c.get("q"), Ok(strs(&["ok"])));
    assert_eq!(c.backend_calls(), 3);
    assert_eq!(c.hits(), 1);
}

#[test]
fn should_call_the_backend_once_for_concurrent_identical_queries() {
    let (b, clock) = setup();
    let gate = Gate::new();
    let g = gate.clone();
    b.set(move |_| {
        g.pass();
        ok(&["d1", "d2"])
    });
    let c = cache!(b, clock, 10, 60_000);
    let spellings = ["red shoes", "Red Shoes", "  red   SHOES "];

    std::thread::scope(|s| {
        let handles: Vec<_> = (0..8)
            .map(|i| {
                let c = &c;
                s.spawn(move || c.get(spellings[i % 3]))
            })
            .collect();
        assert!(gate.wait_entered());
        assert!(eventually(10_000, || c.misses() == 8), "all 8 callers reached the cache");
        assert_eq!(b.calls(), 1);

        gate.release();
        for h in handles {
            assert_eq!(h.join().unwrap(), Ok(strs(&["d1", "d2"])));
        }
    });

    assert_eq!(c.backend_calls(), 1);
    assert_eq!(c.misses(), 8);
    assert_eq!(c.hits(), 0);
    assert_eq!(c.size(), 1);
    c.get("red shoes").unwrap();
    assert_eq!(c.hits(), 1);
}

#[test]
fn should_propagate_one_failure_to_every_waiter_and_retry_on_the_next_call() {
    let (b, clock) = setup();
    let gate = Gate::new();
    let fail = Arc::new(AtomicBool::new(true));
    let (g, f) = (gate.clone(), fail.clone());
    b.set(move |_| {
        g.pass();
        if f.load(Ordering::SeqCst) {
            Err(SearchError("boom".into()))
        } else {
            ok(&["ok"])
        }
    });
    let c = cache!(b, clock, 10, 60_000);

    std::thread::scope(|s| {
        let handles: Vec<_> = (0..6)
            .map(|_| {
                let c = &c;
                s.spawn(move || c.get("q"))
            })
            .collect();
        assert!(gate.wait_entered());
        assert!(eventually(10_000, || c.misses() == 6), "all 6 callers reached the cache");
        gate.release();
        for h in handles {
            assert_eq!(h.join().unwrap(), Err(CacheError::Backend(SearchError("boom".into()))));
        }
    });
    assert_eq!(b.calls(), 1);
    assert_eq!(c.size(), 0);

    fail.store(false, Ordering::SeqCst);
    assert_eq!(c.get("q"), Ok(strs(&["ok"])));
    assert_eq!(b.calls(), 2);
}

#[test]
fn should_not_block_other_queries_while_one_backend_call_is_slow() {
    let (b, clock) = setup();
    let gate = Gate::new();
    let g = gate.clone();
    b.set(move |q| {
        if q == "slow" {
            g.pass();
        }
        ok(&[q])
    });
    let c = cache!(b, clock, 10, 60_000);
    c.get("cached").unwrap();

    std::thread::scope(|s| {
        let slow = {
            let c = &c;
            s.spawn(move || c.get("slow"))
        };
        assert!(gate.wait_entered());

        let fast = {
            let c = &c;
            s.spawn(move || c.get("fast"))
        };
        let cached = {
            let c = &c;
            s.spawn(move || c.get("cached"))
        };
        assert!(eventually(10_000, || fast.is_finished() && cached.is_finished()));
        assert_eq!(fast.join().unwrap(), Ok(strs(&["fast"])));
        assert_eq!(cached.join().unwrap(), Ok(strs(&["cached"])));
        assert!(!slow.is_finished());

        gate.release();
        assert_eq!(slow.join().unwrap(), Ok(strs(&["slow"])));
    });
}

#[test]
fn should_stay_consistent_under_mixed_concurrent_load() {
    let (b, clock) = setup();
    b.set(|q| Ok(vec![format!("{q}#doc")]));
    let c = cache!(b, clock, 10, 50);
    let threads = 8;
    let per_thread = 4_000;
    let gets = AtomicUsize::new(0);

    run_concurrently(threads, |t| {
        for i in 0..per_thread {
            if t == 0 && i % 100 == 0 {
                b.bump();
            }
            if t == 1 && i % 50 == 0 {
                clock.advance(10);
            }
            if t == 2 && i % 700 == 0 {
                c.invalidate_all();
            }
            let key = format!("q{}", (i * 7 + t) % 30);
            let spelling = if i % 2 == 0 { key.clone() } else { format!("  {}  ", key.to_uppercase()) };
            let r = c.get(&spelling).unwrap();
            gets.fetch_add(1, Ordering::SeqCst);
            assert_eq!(r, vec![format!("{key}#doc")]);
            assert!(c.size() <= 10);
        }
    });

    assert_eq!(c.hits() + c.misses(), gets.load(Ordering::SeqCst) as u64);
    assert!(c.backend_calls() <= c.misses());
    assert_eq!(c.backend_calls(), b.calls() as u64);
    assert!(c.size() <= 10);
}
