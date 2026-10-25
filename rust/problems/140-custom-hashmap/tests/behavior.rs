#![allow(unused_imports, clippy::needless_range_loop)]
use p140_custom_hashmap::{contract::*, create};
use playground_support::{run_concurrently, sut, watchdog};
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Hasher built from a closure.
struct FnHasher(Box<dyn Fn(&str) -> u64 + Send + Sync>);

impl KeyHasher for FnHasher {
    fn hash(&self, key: &str) -> u64 {
        (self.0)(key)
    }
}

fn hasher(f: impl Fn(&str) -> u64 + Send + Sync + 'static) -> Arc<dyn KeyHasher> {
    Arc::new(FnHasher(Box::new(f)))
}

/// A well-mixed hash (FNV-1a followed by a murmur3 finaliser).
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
    hasher(mix)
}

/// Everything collides.
fn same() -> Arc<dyn KeyHasher> {
    hasher(|_| 42)
}

/// Huge values, including u64::MAX.
fn huge() -> Arc<dyn KeyHasher> {
    hasher(|k| match k.len() % 3 {
        0 => u64::MAX,
        1 => u64::MAX - (mix(k) & 0xffff),
        _ => 1 << 63 | mix(k),
    })
}

/// Only two distinct values.
fn extreme() -> Arc<dyn KeyHasher> {
    hasher(|k| if k.len() % 2 == 0 { u64::MAX } else { 0 })
}

struct Counting {
    calls: Arc<AtomicU64>,
}

impl KeyHasher for Counting {
    fn hash(&self, key: &str) -> u64 {
        self.calls.fetch_add(1, Ordering::SeqCst);
        mix(key)
    }
}

fn counting() -> (Arc<dyn KeyHasher>, Arc<AtomicU64>) {
    let calls = Arc::new(AtomicU64::new(0));
    (
        Arc::new(Counting {
            calls: calls.clone(),
        }),
        calls,
    )
}

macro_rules! map {
    () => {
        sut!(create(16, 75, good())).unwrap()
    };
    ($c:expr, $l:expr, $h:expr) => {
        sut!(create($c, $l, $h)).unwrap()
    };
}

fn s(v: &str) -> Option<String> {
    Some(v.to_string())
}

fn strings(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

/// The growth rule from the README, restated as a model.
fn model_capacity(initial: usize, lf: u64, inserts: u64) -> usize {
    let mut cap = initial.next_power_of_two() as u64;
    for size in 1..=inserts {
        if size * 100 > cap * lf {
            cap *= 2;
        }
    }
    cap as usize
}

#[test]
fn should_store_and_return_values_and_previous_values() {
    let m = map!();

    assert_eq!(m.put("a", "1"), None);
    assert_eq!(m.put("b", "2"), None);
    assert_eq!(m.get("a"), s("1"));
    assert_eq!(m.put("a", "one"), s("1"));
    assert_eq!(m.get("a"), s("one"));
    assert_eq!(m.get("missing"), None);
    assert!(m.contains_key("a"));
    assert!(!m.contains_key("zzz"));
    assert_eq!(m.size(), 2);
}

#[test]
fn should_treat_keys_as_case_sensitive_and_allow_the_empty_string_as_key_and_value() {
    let m = map!();
    m.put("Key", "upper");
    m.put("key", "lower");
    m.put("", "empty-key");
    m.put("blank-value", "");

    assert_eq!(m.size(), 4);
    assert_eq!(m.get("Key"), s("upper"));
    assert_eq!(m.get("key"), s("lower"));
    assert_eq!(m.get(""), s("empty-key"));
    assert_eq!(m.get("blank-value"), s(""));
    assert!(m.contains_key(""));
    assert_eq!(m.remove(""), s("empty-key"));
    assert!(!m.contains_key(""));
}

#[test]
fn should_reject_invalid_creation_parameters() {
    let _valid = map!();

    assert_eq!(
        sut!(create(0, 75, good())).err(),
        Some(MapError::InvalidArgument)
    );
    assert_eq!(
        sut!(create(1_048_577, 75, good())).err(),
        Some(MapError::InvalidArgument)
    );
    assert_eq!(
        sut!(create(16, 9, good())).err(),
        Some(MapError::InvalidArgument)
    );
    assert_eq!(
        sut!(create(16, 101, good())).err(),
        Some(MapError::InvalidArgument)
    );
    assert_eq!(sut!(create(1, 10, good())).unwrap().capacity(), 1);
    assert_eq!(
        sut!(create(1_048_576, 100, good())).unwrap().capacity(),
        1_048_576
    );
}

#[test]
fn should_remove_and_report_the_previous_value() {
    let m = map!();
    m.put("a", "1");
    m.put("b", "2");

    assert_eq!(m.remove("a"), s("1"));
    assert_eq!(m.remove("a"), None);
    assert_eq!(m.remove("never"), None);
    assert_eq!(m.get("a"), None);
    assert_eq!(m.get("b"), s("2"));
    assert_eq!(m.size(), 1);
}

#[test]
fn should_report_size_and_emptiness_and_clear_without_shrinking() {
    let m = map!(2, 75, good());
    assert!(m.is_empty());
    assert_eq!(m.size(), 0);

    for i in 0..50 {
        m.put(&format!("k{i}"), "v");
    }
    let cap = m.capacity();
    assert!(!m.is_empty());
    assert_eq!(m.size(), 50);

    m.clear();
    assert!(m.is_empty());
    assert_eq!(m.size(), 0);
    assert!(m.keys().is_empty());
    assert_eq!(m.get("k1"), None);
    assert_eq!(m.capacity(), cap);

    m.put("z", "1");
    m.put("a", "2");
    assert_eq!(m.keys(), strings(&["z", "a"]));
}

#[test]
fn should_start_at_the_smallest_power_of_two_at_least_the_initial_capacity() {
    let cases = [
        (1, 1),
        (2, 2),
        (3, 4),
        (5, 8),
        (16, 16),
        (17, 32),
        (100, 128),
        (1000, 1024),
        (1024, 1024),
    ];
    for (initial, expected) in cases {
        let m = map!(initial, 75, good());
        assert_eq!(m.capacity(), expected, "initial {initial}");
    }
}

#[test]
fn should_follow_the_documented_capacity_table_for_default_parameters() {
    let m = map!(16, 75, good());
    let checkpoints = [
        (1, 16),
        (12, 16),
        (13, 32),
        (24, 32),
        (25, 64),
        (48, 64),
        (49, 128),
        (96, 128),
        (97, 256),
    ];
    let mut inserted = 0;
    for (n, cap) in checkpoints {
        while inserted < n {
            m.put(&format!("key-{inserted}"), "v");
            inserted += 1;
        }
        assert_eq!(m.capacity(), cap, "after {n} inserts");
    }
}

#[test]
fn should_double_at_most_once_per_insertion_and_follow_the_rule_for_other_load_factors() {
    // capacity 1, load factor 10%: every one of the first 7 insertions is over the limit and doubles once
    let m = map!(1, 10, good());
    let expected = [2, 4, 8, 16, 32, 64, 128, 128, 128];
    for (i, cap) in expected.iter().enumerate() {
        m.put(&format!("k{i}"), "v");
        assert_eq!(m.capacity(), *cap, "after {} inserts", i + 1);
    }

    // load factor 100%: grows when size exceeds capacity
    let full = map!(4, 100, good());
    for i in 0..4 {
        full.put(&format!("k{i}"), "v");
    }
    assert_eq!(full.capacity(), 4);
    full.put("k4", "v");
    assert_eq!(full.capacity(), 8);

    for lf in [10u32, 33, 50, 75, 90, 100] {
        let x = map!(7, lf, good());
        for i in 1..=600u64 {
            x.put(&format!("n{i}"), "v");
            if i % 50 == 0 || i < 20 {
                assert_eq!(
                    x.capacity(),
                    model_capacity(7, lf as u64, i),
                    "lf {lf} after {i} inserts"
                );
            }
        }
    }
}

#[test]
fn should_grow_only_when_a_new_key_is_inserted_and_never_shrink() {
    let m = map!(4, 100, good());
    for i in 0..4 {
        m.put(&format!("k{i}"), "v0");
    }
    for round in 0..10 {
        for i in 0..4 {
            m.put(&format!("k{i}"), &format!("v{round}"));
            m.put_if_absent(&format!("k{i}"), "ignored");
        }
    }
    assert_eq!(m.capacity(), 4);

    m.put("k4", "v");
    assert_eq!(m.capacity(), 8);

    for i in 0..5 {
        m.remove(&format!("k{i}"));
    }
    assert_eq!(m.size(), 0);
    assert_eq!(m.capacity(), 8);

    // removing and re-adding the same keys: growth depends on size only
    for i in 0..8 {
        m.put(&format!("k{i}"), "v");
    }
    assert_eq!(m.capacity(), 8);
    m.put("k8", "v");
    assert_eq!(m.capacity(), 16);
}

#[test]
fn should_list_keys_in_insertion_order_and_keep_them_stable_across_resizes() {
    let m = map!(1, 75, good());
    let mut expected = Vec::new();
    for i in 0..300 {
        let k = format!("key-{}-{}", (i * 7919) % 1000, i);
        m.put(&k, "v");
        expected.push(k);
    }
    assert!(m.capacity() > 300 * 100 / 75 / 2);
    assert_eq!(m.keys(), expected);
}

#[test]
fn should_keep_the_position_on_reput_and_move_to_the_end_on_remove_then_put() {
    let m = map!();
    m.put("a", "1");
    m.put("b", "2");
    m.put("c", "3");

    m.put("a", "again");
    assert_eq!(m.keys(), strings(&["a", "b", "c"]));

    m.put_if_absent("b", "ignored");
    assert_eq!(m.keys(), strings(&["a", "b", "c"]));

    m.remove("a");
    m.put("a", "new");
    assert_eq!(m.keys(), strings(&["b", "c", "a"]));

    m.remove("c");
    assert_eq!(m.keys(), strings(&["b", "a"]));
    m.put("d", "4");
    m.put("c", "5");
    assert_eq!(m.keys(), strings(&["b", "a", "d", "c"]));
}

#[test]
fn should_list_entries_as_key_equals_value_in_the_same_order_as_keys() {
    let m = map!();
    m.put("x", "1");
    m.put("", "empty");
    m.put("k=v", "a=b");
    m.put("blank", "");
    m.put("x", "2");

    assert_eq!(m.entries(), strings(&["x=2", "=empty", "k=v=a=b", "blank="]));
    assert_eq!(m.keys(), strings(&["x", "", "k=v", "blank"]));
}

#[test]
fn should_return_snapshots_from_keys_and_entries() {
    let m = map!();
    m.put("a", "1");
    let keys = m.keys();
    let entries = m.entries();

    m.put("b", "2");
    m.remove("a");

    assert_eq!(keys, strings(&["a"]));
    assert_eq!(entries, strings(&["a=1"]));
    assert_eq!(m.keys(), strings(&["b"]));
}

#[test]
fn should_put_if_absent_only_when_the_key_is_missing() {
    let m = map!();

    assert_eq!(m.put_if_absent("a", "1"), None);
    assert_eq!(m.put_if_absent("a", "2"), s("1"));
    assert_eq!(m.get("a"), s("1"));
    assert_eq!(m.size(), 1);
    m.remove("a");
    assert_eq!(m.put_if_absent("a", "3"), None);
    assert_eq!(m.get("a"), s("3"));
}

#[test]
fn should_return_the_default_from_get_or_default_without_storing_it() {
    let m = map!();
    m.put("a", "1");

    assert_eq!(m.get_or_default("a", "d"), "1");
    assert_eq!(m.get_or_default("b", "d"), "d");
    assert_eq!(m.get_or_default("b", ""), "");
    assert!(!m.contains_key("b"));
    assert_eq!(m.size(), 1);
}

#[test]
fn should_call_the_supplied_hasher_exactly_once_per_keyed_operation_and_never_for_the_others() {
    let (h, calls) = counting();
    let m = map!(2, 50, h);
    let delta = |before: u64| calls.load(Ordering::SeqCst) - before;

    let before = calls.load(Ordering::SeqCst);
    m.put("a", "1");
    assert_eq!(delta(before), 1, "put new");
    let before = calls.load(Ordering::SeqCst);
    m.put("a", "2");
    assert_eq!(delta(before), 1, "put existing");
    let before = calls.load(Ordering::SeqCst);
    m.put_if_absent("b", "1");
    m.put_if_absent("b", "1");
    assert_eq!(delta(before), 2, "two put_if_absent");
    let before = calls.load(Ordering::SeqCst);
    m.get("a");
    m.get("nope");
    m.get_or_default("a", "d");
    m.get_or_default("nope", "d");
    m.contains_key("a");
    m.contains_key("nope");
    assert_eq!(delta(before), 6, "six reads");
    let before = calls.load(Ordering::SeqCst);
    m.remove("a");
    m.remove("nope");
    assert_eq!(delta(before), 2, "two removes");

    let before = calls.load(Ordering::SeqCst);
    m.size();
    m.is_empty();
    m.capacity();
    m.keys();
    m.entries();
    m.clear();
    assert_eq!(delta(before), 0, "size/is_empty/capacity/keys/entries/clear");

    // growing must not ask the hasher again for the keys that are already stored
    for i in 0..200 {
        let before = calls.load(Ordering::SeqCst);
        m.put(&format!("grow-{i}"), "v");
        assert_eq!(delta(before), 1, "put #{i} (may resize)");
    }
    assert!(m.capacity() > 200);
}

#[test]
fn should_stay_correct_with_hashers_returning_huge_values() {
    for h in [huge(), extreme()] {
        let m = map!(4, 75, h);
        for i in 0..2_000 {
            assert_eq!(m.put(&format!("key-{i}"), &format!("v{i}")), None);
        }
        assert_eq!(m.size(), 2_000);
        for i in 0..2_000 {
            assert_eq!(m.get(&format!("key-{i}")), Some(format!("v{i}")));
        }
        for i in (0..2_000).step_by(2) {
            assert_eq!(m.remove(&format!("key-{i}")), Some(format!("v{i}")));
        }
        assert_eq!(m.size(), 1_000);
        assert!(!m.contains_key("key-0"));
        assert!(m.contains_key("key-1"));
        assert_eq!(m.capacity(), model_capacity(4, 75, 2_000));
    }
}

#[test]
fn should_keep_colliding_keys_apart_when_every_key_has_the_same_hash() {
    let m = map!(2, 75, same());
    m.put("Aa", "1");
    m.put("BB", "2");
    m.put("c", "3");

    assert_eq!(m.get("Aa"), s("1"));
    assert_eq!(m.get("BB"), s("2"));
    assert_eq!(m.remove("BB"), s("2"));
    assert_eq!(m.get("Aa"), s("1"));
    assert_eq!(m.get("c"), s("3"));
    assert!(!m.contains_key("BB"));
    assert_eq!(m.put("Aa", "9"), s("1"));
    assert_eq!(m.keys(), strings(&["Aa", "c"]));
    assert_eq!(m.capacity(), model_capacity(2, 75, 3));
}

#[test]
fn should_handle_five_thousand_keys_that_all_collide_within_the_documented_bound() {
    let _guard = watchdog(20_000);
    let m = map!(16, 75, same());
    for i in 0..5_000 {
        assert_eq!(m.put(&format!("key-{i}"), &format!("v{i}")), None);
    }
    assert_eq!(m.size(), 5_000);
    for i in 0..5_000 {
        assert_eq!(m.get(&format!("key-{i}")), Some(format!("v{i}")));
    }
    for i in (0..5_000).step_by(2) {
        assert_eq!(m.remove(&format!("key-{i}")), Some(format!("v{i}")));
    }
    assert_eq!(m.size(), 2_500);
    assert!(!m.contains_key("key-2"));
    assert_eq!(m.get("key-4999"), s("v4999"));
    assert_eq!(m.capacity(), model_capacity(16, 75, 5_000));
    assert_eq!(m.keys()[0], "key-1");
}

#[test]
fn should_handle_one_million_puts_and_gets_with_a_good_hasher_within_the_documented_bound() {
    let _guard = watchdog(60_000);
    let m = map!(16, 75, good());
    let n = 1_000_000;
    for i in 0..n {
        m.put(&format!("key-{i}"), &format!("v{i}"));
    }
    assert_eq!(m.size(), n);
    for i in 0..n {
        assert_eq!(m.get(&format!("key-{i}")), Some(format!("v{i}")));
    }
    assert_eq!(m.get(&format!("key-{n}")), None);
    assert_eq!(m.capacity(), 2_097_152);
    assert_eq!(m.capacity(), model_capacity(16, 75, n as u64));
}

#[test]
fn should_lose_nothing_when_eight_threads_put_disjoint_keys() {
    let m = map!(16, 75, good());
    let per_thread = 5_000;

    run_concurrently(8, |t| {
        for i in 0..per_thread {
            m.put(&format!("t{t}-{i}"), &format!("v{i}"));
        }
    });

    assert_eq!(m.size(), 8 * per_thread);
    assert_eq!(m.capacity(), model_capacity(16, 75, 8 * per_thread as u64));
    for t in 0..8 {
        for i in (0..per_thread).step_by(97) {
            assert_eq!(m.get(&format!("t{t}-{i}")), Some(format!("v{i}")));
        }
    }
    let distinct: HashSet<String> = m.keys().into_iter().collect();
    assert_eq!(distinct.len(), 8 * per_thread);
}
