#![allow(unused_imports, clippy::needless_range_loop)]
use p30_concurrent_cache::{contract::*, create};
use playground_support::{collect_concurrently, eventually, sut, FakeClock};
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::sync::Arc;

const TTL: u64 = 1_000;

fn cache(clock: &FakeClock, capacity: usize) -> Option<Arc<dyn CacheApi>> {
    create(capacity, TTL, clock.as_clock())
}

#[test]
fn ttl_of_a_computed_value_starts_when_the_loader_finishes() {
    let clock = FakeClock::new(0);
    let cache = sut!(cache(&clock, 3));

    let _ = cache.get_or_compute("k", &|_| {
        clock.advance(600);
        Ok("v".to_string())
    });

    clock.advance(999);
    assert_eq!(cache.get("k").as_deref(), Some("v"));
    clock.advance(1);
    assert_eq!(cache.get("k"), None);
}

#[test]
fn a_failed_reload_of_an_expired_key_leaves_it_absent() {
    let clock = FakeClock::new(0);
    let cache = sut!(cache(&clock, 3));
    cache.put("k", "old");
    clock.advance(TTL);

    let r = cache.get_or_compute("k", &|_| Err("nope".to_string()));

    assert_eq!(r, Err("nope".to_string()));
    assert_eq!(cache.get("k"), None);
    assert_eq!(cache.size(), 0);
}

#[test]
fn a_computed_value_inserted_into_a_full_cache_evicts_the_least_recently_used_entry() {
    let clock = FakeClock::new(0);
    let cache = sut!(cache(&clock, 2));
    cache.put("a", "1");
    cache.put("b", "2");

    let _ = cache.get_or_compute("c", &|_| Ok("3".to_string()));

    assert_eq!(cache.get("a"), None);
    assert_eq!(cache.get("b").as_deref(), Some("2"));
    assert_eq!(cache.get("c").as_deref(), Some("3"));
}

#[test]
fn concurrent_callers_after_expiry_share_one_reload() {
    let clock = FakeClock::new(0);
    let cache = sut!(cache(&clock, 5));
    cache.put("k", "old");
    clock.advance(TTL);
    let threads = 8;
    let entered = AtomicUsize::new(0);
    let calls = AtomicUsize::new(0);

    let results = collect_concurrently(threads, |_| {
        entered.fetch_add(1, SeqCst);
        cache.get_or_compute("k", &|_| {
            calls.fetch_add(1, SeqCst);
            eventually(5_000, || entered.load(SeqCst) == threads);
            Ok("new".to_string())
        })
    });

    assert_eq!(calls.load(SeqCst), 1);
    assert!(results.iter().all(|r| r == &Ok("new".to_string())));
    assert_eq!(cache.get("k").as_deref(), Some("new"));
}

#[test]
fn should_work_with_capacity_one() {
    let clock = FakeClock::new(0);
    let cache = sut!(cache(&clock, 1));

    cache.put("a", "1");
    cache.put("b", "2");

    assert_eq!(cache.get("a"), None);
    assert_eq!(cache.get("b").as_deref(), Some("2"));
    assert_eq!(cache.size(), 1);
    assert_eq!(
        cache.get_or_compute("c", &|_| Ok("3".to_string())),
        Ok("3".to_string())
    );
    assert_eq!(cache.get("b"), None);
    assert_eq!(cache.size(), 1);
}
