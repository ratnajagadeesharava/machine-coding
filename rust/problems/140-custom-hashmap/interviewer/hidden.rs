#![allow(unused_imports, clippy::needless_range_loop)]
use p140_custom_hashmap::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut, watchdog};
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

struct FnHasher(Box<dyn Fn(&str) -> u64 + Send + Sync>);

impl KeyHasher for FnHasher {
    fn hash(&self, key: &str) -> u64 {
        (self.0)(key)
    }
}

fn mix(key: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in key.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
    h ^ (h >> 33)
}

fn good() -> Arc<dyn KeyHasher> {
    Arc::new(FnHasher(Box::new(mix)))
}

fn same() -> Arc<dyn KeyHasher> {
    Arc::new(FnHasher(Box::new(|_| u64::MAX - 7)))
}

macro_rules! map {
    ($c:expr, $l:expr, $h:expr) => {
        sut!(create($c, $l, $h)).unwrap()
    };
}


/// xorshift, deterministic
fn next(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

#[test]
fn should_remove_two_hundred_thousand_keys_in_random_order_without_scanning_the_whole_map() {
    let _guard = watchdog(30_000);
    let m = map!(16, 75, good());
    let n = 200_000usize;
    for i in 0..n {
        m.put(&format!("key-{i}"), "v");
    }
    let mut order: Vec<usize> = (0..n).collect();
    let mut st = 88172645463325252u64;
    for i in (1..n).rev() {
        let j = (next(&mut st) % (i as u64 + 1)) as usize;
        order.swap(i, j);
    }
    for i in order {
        assert!(m.remove(&format!("key-{i}")).is_some());
    }
    assert_eq!(m.size(), 0);
    assert!(m.keys().is_empty());
    assert_eq!(m.capacity(), 524_288);
}

#[test]
fn should_not_grow_when_the_same_few_keys_are_added_and_removed_over_and_over() {
    let m = map!(16, 75, good());
    for round in 0..100_000 {
        m.put(&format!("k{}", round % 5), &format!("v{round}"));
        if round % 5 == 4 {
            for j in 0..5 {
                assert!(m.remove(&format!("k{j}")).is_some());
            }
        }
    }
    assert_eq!(m.capacity(), 16);
    assert!(m.is_empty());
    m.put("b", "1");
    m.put("a", "2");
    assert_eq!(m.keys(), vec!["b".to_string(), "a".to_string()]);
}

#[test]
fn should_keep_order_and_growth_rule_when_everything_collides_and_half_the_keys_are_removed() {
    let m = map!(1, 50, same());
    for i in 0..3_000 {
        m.put(&format!("k{i}"), &format!("v{i}"));
    }
    assert_eq!(m.capacity(), 8192);
    for i in (0..3_000).step_by(2) {
        m.remove(&format!("k{i}"));
    }
    let expected: Vec<String> = (1..3_000).step_by(2).map(|i| format!("k{i}")).collect();
    assert_eq!(m.keys(), expected);
    assert_eq!(m.entries()[0], "k1=v1");
    assert_eq!(m.capacity(), 8192);
}

#[test]
fn should_let_exactly_one_thread_win_put_if_absent_for_each_key() {
    let m = map!(16, 75, good());
    let keys = 2_000;
    let winners = AtomicUsize::new(0);

    run_concurrently(8, |t| {
        for i in 0..keys {
            if m.put_if_absent(&format!("key-{i}"), &format!("t{t}")).is_none() {
                winners.fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    assert_eq!(winners.load(Ordering::SeqCst), keys);
    assert_eq!(m.size(), keys);
    for i in 0..keys {
        assert!(m.get(&format!("key-{i}")).is_some());
    }
}

#[test]
fn should_stay_internally_consistent_under_concurrent_put_remove_and_get_on_overlapping_keys() {
    let m = map!(2, 75, good());

    run_concurrently(8, |t| {
        let mut st = 0x9E3779B97F4A7C15u64 ^ (t as u64 + 1);
        for _ in 0..40_000 {
            let k = format!("k{}", next(&mut st) % 300);
            match next(&mut st) % 4 {
                0 => {
                    m.put(&k, &format!("t{t}"));
                }
                1 => {
                    m.remove(&k);
                }
                2 => {
                    m.get(&k);
                }
                _ => {
                    m.put_if_absent(&k, &format!("t{t}"));
                }
            }
        }
    });

    let keys = m.keys();
    assert_eq!(keys.len(), m.size());
    let distinct: HashSet<&String> = keys.iter().collect();
    assert_eq!(distinct.len(), keys.len());
    assert_eq!(m.entries().len(), keys.len());
    for k in &keys {
        assert!(m.contains_key(k));
    }
    for i in 0..300 {
        let k = format!("k{i}");
        assert_eq!(m.contains_key(&k), keys.contains(&k));
    }
    assert!(m.capacity() <= 1024);
}

#[test]
fn should_call_the_hasher_exactly_once_per_operation_even_under_contention() {
    struct Counting(Arc<AtomicU64>);
    impl KeyHasher for Counting {
        fn hash(&self, key: &str) -> u64 {
            self.0.fetch_add(1, Ordering::SeqCst);
            mix(key)
        }
    }
    let calls = Arc::new(AtomicU64::new(0));
    let m = map!(2, 75, Arc::new(Counting(calls.clone())));
    let ops = AtomicU64::new(0);

    run_concurrently(8, |t| {
        for i in 0..5_000usize {
            let k = format!("k{}", (i * 31 + t) % 700);
            m.put(&k, "v");
            m.get(&k);
            m.remove(&k);
            m.size();
            m.keys();
            ops.fetch_add(3, Ordering::SeqCst);
        }
    });

    assert_eq!(calls.load(Ordering::SeqCst), ops.load(Ordering::SeqCst));
}

#[test]
fn should_return_atomic_snapshots_of_keys_while_other_threads_insert() {
    let m = map!(2, 75, good());
    let per_thread = 3_000;
    let snapshots: Mutex<Vec<Vec<String>>> = Mutex::new(Vec::new());

    run_concurrently(6, |t| {
        if t >= 4 {
            for _ in 0..100 {
                let snap = m.keys();
                snapshots.lock().unwrap().push(snap);
                assert!(m.entries().len() <= 4 * per_thread);
            }
            return;
        }
        for i in 0..per_thread {
            m.put(&format!("t{t}-{i:05}"), "v");
        }
    });

    for snap in snapshots.lock().unwrap().iter() {
        let distinct: HashSet<&String> = snap.iter().collect();
        assert_eq!(distinct.len(), snap.len());
        let mut last = [-1i64; 4];
        for k in snap {
            let t = (k.as_bytes()[1] - b'0') as usize;
            let i: i64 = k[3..].parse().unwrap();
            assert!(i > last[t], "per-thread insertion order inside a snapshot");
            last[t] = i;
        }
    }
    assert_eq!(m.size(), 4 * per_thread);
}

#[test]
fn should_be_usable_as_a_general_purpose_map_after_clear_and_reuse() {
    let m = map!(3, 40, good());
    for round in 0..5 {
        for i in 0..100 {
            m.put(&format!("r{round}-{i}"), "v");
        }
        assert_eq!(m.size(), 100);
        assert_eq!(m.keys()[0], format!("r{round}-0"));
        m.clear();
        assert_eq!(m.get(&format!("r{round}-0")), None);
    }
    assert_eq!(m.capacity(), 256);
    assert_eq!(m.put("again", "1"), None);
}
