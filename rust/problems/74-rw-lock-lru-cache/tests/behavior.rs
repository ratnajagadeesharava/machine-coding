#![allow(unused_imports, clippy::needless_range_loop)]
use p74_rw_lock_lru_cache::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

fn value(key: u64) -> String {
    format!("v{key}")
}

/// Tiny xorshift so the tests need no external crates.
struct Rng(u64);
impl Rng {
    fn new(seed: usize) -> Self {
        Rng(0x9E37_79B9_7F4A_7C15 ^ (seed as u64 + 1).wrapping_mul(0xD1B5_4A32_D192_ED03))
    }
    fn next(&mut self, n: u64) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0 % n
    }
}

/// In a quiet moment: how many keys of the universe are present, read with `peek` (no side effects).
fn count_present(c: &Arc<dyn SharedCache>, universe: u64) -> usize {
    let mut present = 0;
    for k in 0..universe {
        if let Some(v) = c.peek(k) {
            assert_eq!(v, value(k));
            present += 1;
        }
    }
    present
}

#[test]
fn should_evict_the_least_recently_used_entry() {
    let c = sut!(create(2));
    c.put(1, "a".into());
    c.put(2, "b".into());

    c.put(3, "c".into());

    assert_eq!(c.peek(1), None);
    assert_eq!(c.peek(2), Some("b".into()));
    assert_eq!(c.peek(3), Some("c".into()));
    assert_eq!(c.len(), 2);
}

#[test]
fn should_treat_get_as_a_use() {
    let c = sut!(create(2));
    c.put(1, "a".into());
    c.put(2, "b".into());

    assert_eq!(c.get(1), Some("a".into()));
    c.put(3, "c".into());

    assert_eq!(c.peek(2), None);
    assert_eq!(c.peek(1), Some("a".into()));
}

#[test]
fn should_not_treat_peek_as_a_use() {
    let c = sut!(create(2));
    c.put(1, "a".into());
    c.put(2, "b".into());

    assert_eq!(c.peek(1), Some("a".into()));
    c.put(3, "c".into());

    assert_eq!(c.peek(1), None);
    assert_eq!(c.peek(2), Some("b".into()));
}

#[test]
fn should_replace_an_existing_key_and_refresh_it() {
    let c = sut!(create(2));
    c.put(1, "a".into());
    c.put(2, "b".into());

    c.put(1, "a2".into());
    c.put(3, "c".into());

    assert_eq!(c.peek(1), Some("a2".into()));
    assert_eq!(c.peek(2), None);
    assert_eq!(c.len(), 2);
}

#[test]
fn should_remove_entries() {
    let c = sut!(create(3));
    c.put(1, "a".into());

    assert!(c.remove(1));
    assert!(!c.remove(1));

    assert_eq!(c.get(1), None);
    assert_eq!(c.len(), 0);
}

#[test]
fn should_work_with_capacity_one() {
    let c = sut!(create(1));

    c.put(1, "a".into());
    c.put(2, "b".into());

    assert_eq!(c.peek(1), None);
    assert_eq!(c.peek(2), Some("b".into()));
}

#[test]
fn should_lose_no_updates_when_many_threads_insert_distinct_keys() {
    let c = sut!(create(100_000));

    run_concurrently(8, |t| {
        for i in 0..1_000u64 {
            let key = t as u64 * 1_000 + i;
            c.put(key, value(key));
        }
    });

    assert_eq!(c.len(), 8_000);
    assert_eq!(count_present(&c, 8_000), 8_000);
}

#[test]
fn should_never_exceed_capacity_and_stay_consistent_under_mixed_load() {
    let capacity = 50;
    let universe = 500;
    let c = sut!(create(capacity));
    let oversize = AtomicUsize::new(0);
    let wrong_values = AtomicUsize::new(0);

    run_concurrently(12, |t| {
        let mut rnd = Rng::new(t);
        for _ in 0..20_000 {
            let key = rnd.next(universe);
            match rnd.next(10) {
                0..=3 => c.put(key, value(key)),
                4..=6 => {
                    if c.get(key).is_some_and(|v| v != value(key)) {
                        wrong_values.fetch_add(1, Ordering::SeqCst);
                    }
                }
                7..=8 => {
                    if c.peek(key).is_some_and(|v| v != value(key)) {
                        wrong_values.fetch_add(1, Ordering::SeqCst);
                    }
                }
                _ => {
                    c.remove(key);
                }
            }
            if c.len() > capacity {
                oversize.fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    assert_eq!(oversize.load(Ordering::SeqCst), 0, "len() above capacity");
    assert_eq!(
        wrong_values.load(Ordering::SeqCst),
        0,
        "a key returned another key's value"
    );
    assert_eq!(
        count_present(&c, universe),
        c.len(),
        "len() disagrees with what is actually stored"
    );
}

#[test]
fn should_stay_full_after_many_concurrent_inserts_of_new_keys() {
    let capacity = 50;
    let c = sut!(create(capacity));

    run_concurrently(8, |t| {
        for i in 0..5_000u64 {
            let key = t as u64 * 5_000 + i;
            c.put(key, value(key));
        }
    });

    assert_eq!(c.len(), capacity);
    assert_eq!(count_present(&c, 40_000), capacity);
}

#[test]
fn should_survive_many_threads_reading_the_same_keys_at_once() {
    let capacity = 100;
    let c = sut!(create(capacity));
    for k in 0..capacity as u64 {
        c.put(k, value(k));
    }
    let misses = AtomicUsize::new(0);

    run_concurrently(16, |t| {
        let mut rnd = Rng::new(t);
        for _ in 0..20_000 {
            // get() reorders the usage list even though it "only reads": it must be safe in parallel
            if c.get(rnd.next(capacity as u64)).is_none() {
                misses.fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    assert_eq!(misses.load(Ordering::SeqCst), 0);
    assert_eq!(c.len(), capacity);
}

#[test]
fn should_keep_a_hot_key_alive_while_other_threads_only_read() {
    let c = sut!(create(10));
    const HOT: u64 = u64::MAX;
    c.put(HOT, "hot".into());
    let writer_done = AtomicBool::new(false);
    let hot_misses = AtomicUsize::new(0);

    run_concurrently(5, |t| {
        if t == 0 {
            // the only writer: touches the hot key after every insert
            for i in 0..20_000u64 {
                c.put(i, value(i));
                if c.get(HOT).is_none() {
                    hot_misses.fetch_add(1, Ordering::SeqCst);
                    c.put(HOT, "hot".into());
                }
            }
            writer_done.store(true, Ordering::SeqCst);
            return;
        }
        while !writer_done.load(Ordering::SeqCst) {
            c.peek(HOT);
            c.get(HOT);
        }
    });

    assert_eq!(
        hot_misses.load(Ordering::SeqCst),
        0,
        "the most recently used key was evicted"
    );
}

#[test]
fn should_stay_correct_over_100_runs() {
    for run in 0..100usize {
        let capacity = 5 + run % 10;
        let universe = 40;
        let c = sut!(create(capacity));
        let oversize = AtomicUsize::new(0);

        run_concurrently(8, |t| {
            let mut rnd = Rng::new(t + run * 8);
            for _ in 0..1_000 {
                let key = rnd.next(universe);
                match rnd.next(4) {
                    0 => c.put(key, value(key)),
                    1 => {
                        c.remove(key);
                    }
                    2 => {
                        c.get(key);
                    }
                    _ => {
                        c.peek(key);
                    }
                }
                if c.len() > capacity {
                    oversize.fetch_add(1, Ordering::SeqCst);
                }
            }
        });

        assert_eq!(oversize.load(Ordering::SeqCst), 0, "run {run}");
        assert_eq!(count_present(&c, universe), c.len(), "run {run}");
    }
}
