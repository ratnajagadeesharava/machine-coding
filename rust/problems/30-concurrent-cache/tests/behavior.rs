#![allow(unused_imports, clippy::needless_range_loop)]
use p30_concurrent_cache::{contract::*, create};
use playground_support::{collect_concurrently, eventually, run_concurrently, sut, FakeClock};
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::sync::Arc;
use std::time::Duration;

const TTL: u64 = 1_000;

fn cache(clock: &FakeClock, capacity: usize) -> Option<Arc<dyn CacheApi>> {
    create(capacity, TTL, clock.as_clock())
}

#[test]
fn should_return_none_for_missing_key() {
    let clock = FakeClock::new(5_000);
    let cache = sut!(cache(&clock, 3));

    assert_eq!(cache.get("nope"), None);
    assert_eq!(cache.size(), 0);
}

#[test]
fn should_return_stored_value_and_overwrite() {
    let clock = FakeClock::new(5_000);
    let cache = sut!(cache(&clock, 3));

    cache.put("a", "1");
    assert_eq!(cache.get("a").as_deref(), Some("1"));
    cache.put("a", "2");

    assert_eq!(cache.get("a").as_deref(), Some("2"));
    assert_eq!(cache.size(), 1);
}

#[test]
fn should_expire_exactly_at_ttl() {
    let clock = FakeClock::new(5_000);
    let cache = sut!(cache(&clock, 3));
    cache.put("a", "1");

    clock.advance(TTL - 1);
    assert_eq!(cache.get("a").as_deref(), Some("1"));

    clock.advance(1);
    assert_eq!(cache.get("a"), None);
}

#[test]
fn should_not_extend_ttl_on_read() {
    let clock = FakeClock::new(5_000);
    let cache = sut!(cache(&clock, 3));
    cache.put("a", "1");

    clock.advance(600);
    assert_eq!(cache.get("a").as_deref(), Some("1"));
    clock.advance(400);

    assert_eq!(cache.get("a"), None);
}

#[test]
fn should_restart_ttl_when_overwritten() {
    let clock = FakeClock::new(5_000);
    let cache = sut!(cache(&clock, 3));
    cache.put("a", "1");
    clock.advance(600);
    cache.put("a", "2");

    clock.advance(600); // 1200 after the first put, 600 after the second
    assert_eq!(cache.get("a").as_deref(), Some("2"));
    clock.advance(400);
    assert_eq!(cache.get("a"), None);
}

#[test]
fn should_count_only_live_entries_in_size() {
    let clock = FakeClock::new(5_000);
    let cache = sut!(cache(&clock, 3));
    cache.put("a", "1");
    clock.advance(600);
    cache.put("b", "2");
    assert_eq!(cache.size(), 2);

    clock.advance(400); // a expired (age 1000), b alive (age 400)
    assert_eq!(cache.size(), 1);

    clock.advance(600); // b expired
    assert_eq!(cache.size(), 0);
}

#[test]
fn should_evict_least_recently_used_when_full() {
    let clock = FakeClock::new(5_000);
    let cache = sut!(cache(&clock, 2));
    cache.put("a", "1");
    cache.put("b", "2");
    cache.get("a");

    cache.put("c", "3");

    assert_eq!(cache.get("b"), None);
    assert_eq!(cache.get("a").as_deref(), Some("1"));
    assert_eq!(cache.get("c").as_deref(), Some("3"));
    assert_eq!(cache.size(), 2);
}

#[test]
fn should_not_evict_when_overwriting_an_existing_key_at_capacity() {
    let clock = FakeClock::new(5_000);
    let cache = sut!(cache(&clock, 2));
    cache.put("a", "1");
    cache.put("b", "2");

    cache.put("a", "9");

    assert_eq!(cache.get("a").as_deref(), Some("9"));
    assert_eq!(cache.get("b").as_deref(), Some("2"));
}

#[test]
fn should_drop_expired_entries_before_evicting_live_ones() {
    let clock = FakeClock::new(5_000);
    let cache = sut!(cache(&clock, 2));
    cache.put("a", "1"); // t0
    clock.advance(600);
    cache.put("b", "2"); // t0+600
    cache.get("a"); // a is now the most recently used, b the least
    clock.advance(400); // a expires (age 1000); b is alive (age 400)

    cache.put("c", "3");

    assert_eq!(cache.get("b").as_deref(), Some("2"));
    assert_eq!(cache.get("c").as_deref(), Some("3"));
    assert_eq!(cache.size(), 2);
}

#[test]
fn should_compute_once_and_serve_later_calls_from_the_cache() {
    let clock = FakeClock::new(5_000);
    let cache = sut!(cache(&clock, 3));
    let calls = AtomicUsize::new(0);

    let first = cache.get_or_compute("k", &|key| {
        calls.fetch_add(1, SeqCst);
        Ok(format!("v:{key}"))
    });
    let second = cache.get_or_compute("k", &|_| {
        calls.fetch_add(1, SeqCst);
        Ok("other".to_string())
    });

    assert_eq!(first, Ok("v:k".to_string()));
    assert_eq!(second, Ok("v:k".to_string()));
    assert_eq!(calls.load(SeqCst), 1);
    assert_eq!(cache.get("k").as_deref(), Some("v:k"));
}

#[test]
fn should_recompute_an_expired_value() {
    let clock = FakeClock::new(5_000);
    let cache = sut!(cache(&clock, 3));
    cache.put("k", "old");
    clock.advance(TTL);

    assert_eq!(
        cache.get_or_compute("k", &|_| Ok("new".into())),
        Ok("new".to_string())
    );
    assert_eq!(cache.get("k").as_deref(), Some("new"));
}

#[test]
fn should_count_compute_hits_as_use() {
    let clock = FakeClock::new(5_000);
    let cache = sut!(cache(&clock, 2));
    cache.put("a", "1");
    cache.put("b", "2");

    let hit = cache.get_or_compute("a", &|_| panic!("loader must not run on a hit"));
    assert_eq!(hit, Ok("1".to_string()));
    cache.put("c", "3");

    assert_eq!(cache.get("a").as_deref(), Some("1"));
    assert_eq!(cache.get("b"), None);
}

#[test]
fn should_propagate_loader_failure_cache_nothing_and_retry_next_time() {
    let clock = FakeClock::new(5_000);
    let cache = sut!(cache(&clock, 3));
    let calls = AtomicUsize::new(0);

    let failed = cache.get_or_compute("k", &|_| {
        calls.fetch_add(1, SeqCst);
        Err("boom".to_string())
    });
    assert_eq!(failed, Err("boom".to_string()));
    assert_eq!(cache.get("k"), None);
    assert_eq!(cache.size(), 0);

    let value = cache.get_or_compute("k", &|_| {
        calls.fetch_add(1, SeqCst);
        Ok("ok".to_string())
    });

    assert_eq!(value, Ok("ok".to_string()));
    assert_eq!(calls.load(SeqCst), 2);
}

#[test]
fn should_run_the_loader_once_for_concurrent_callers_of_the_same_key() {
    let clock = FakeClock::new(5_000);
    let cache = sut!(cache(&clock, 10));
    let threads = 8;
    let entered = AtomicUsize::new(0);
    let calls = AtomicUsize::new(0);

    let results = collect_concurrently(threads, |_| {
        entered.fetch_add(1, SeqCst);
        cache.get_or_compute("k", &|key| {
            calls.fetch_add(1, SeqCst);
            eventually(5_000, || entered.load(SeqCst) == threads);
            Ok(format!("v:{key}"))
        })
    });

    assert_eq!(calls.load(SeqCst), 1);
    assert_eq!(results.len(), threads);
    assert!(results.iter().all(|r| r == &Ok("v:k".to_string())));
}

#[test]
fn should_share_a_loader_failure_with_concurrent_callers() {
    let clock = FakeClock::new(5_000);
    let cache = sut!(cache(&clock, 10));
    let threads = 6;
    let entered = AtomicUsize::new(0);
    let calls = AtomicUsize::new(0);

    let outcomes = collect_concurrently(threads, |_| {
        entered.fetch_add(1, SeqCst);
        cache.get_or_compute("k", &|_| {
            calls.fetch_add(1, SeqCst);
            eventually(5_000, || entered.load(SeqCst) == threads);
            std::thread::sleep(Duration::from_millis(150)); // let every caller attach to this computation
            Err("boom".to_string())
        })
    });

    assert!(outcomes.iter().all(|r| r == &Err("boom".to_string())));
    assert_eq!(calls.load(SeqCst), 1);
    assert_eq!(cache.get("k"), None);
    assert_eq!(
        cache.get_or_compute("k", &|_| Ok("recovered".into())),
        Ok("recovered".to_string())
    );
}

#[test]
fn should_compute_different_keys_in_parallel() {
    let clock = FakeClock::new(5_000);
    let cache = sut!(cache(&clock, 10));
    let running = AtomicUsize::new(0);

    let results = collect_concurrently(2, |i| {
        cache.get_or_compute(&format!("k{i}"), &|key| {
            running.fetch_add(1, SeqCst);
            if eventually(5_000, || running.load(SeqCst) == 2) {
                Ok(format!("v:{key}"))
            } else {
                Err("loaders for different keys did not overlap".to_string())
            }
        })
    });

    assert_eq!(
        results,
        vec![Ok("v:k0".to_string()), Ok("v:k1".to_string())]
    );
}

#[test]
fn should_allow_a_loader_to_use_the_cache_for_other_keys() {
    let clock = FakeClock::new(5_000);
    let cache = sut!(cache(&clock, 10));
    cache.put("c", "cv");

    let result = cache.get_or_compute("a", &|key| {
        let b = cache.get_or_compute("b", &|_| Ok("bv".to_string()))?;
        let c = cache.get("c").unwrap_or_else(|| "?".to_string());
        Ok(format!("{key}{b}{c}"))
    });

    assert_eq!(result, Ok("abvcv".to_string()));
    assert_eq!(cache.size(), 3);
}

#[test]
fn should_stay_within_capacity_under_mixed_concurrent_load() {
    let clock = FakeClock::new(5_000);
    let cache = sut!(cache(&clock, 10));

    run_concurrently(16, |t| {
        for i in 0..500usize {
            let key = format!("k{}", i % 50);
            let expected = format!("v:{key}");
            match (t + i) % 3 {
                0 => cache.put(&key, &expected),
                1 => {
                    if let Some(v) = cache.get(&key) {
                        assert_eq!(v, expected);
                    }
                }
                _ => {
                    let v = cache.get_or_compute(&key, &|k| Ok(format!("v:{k}")));
                    assert_eq!(v, Ok(expected));
                }
            }
            assert!(cache.size() <= 10);
        }
    });

    let size = cache.size();
    assert!((1..=10).contains(&size));
    let present = (0..50)
        .filter(|i| cache.get(&format!("k{i}")).is_some())
        .count();
    assert_eq!(present, size);
}
