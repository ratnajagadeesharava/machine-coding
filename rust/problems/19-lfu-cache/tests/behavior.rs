#![allow(unused_imports, clippy::needless_range_loop)]
use p19_lfu_cache::{contract::*, create};
use playground_support::sut;
use std::time::{Duration, Instant};

fn k(s: &str) -> String {
    s.to_string()
}

#[test]
fn should_follow_the_classic_eviction_sequence() {
    let mut c = sut!(create(2));

    c.put(k("1"), 1);
    c.put(k("2"), 2);
    assert_eq!(c.get(&k("1")), Some(1));
    c.put(k("3"), 3); // evicts 2 (count 1)
    assert_eq!(c.get(&k("2")), None);
    assert_eq!(c.get(&k("3")), Some(3));
    c.put(k("4"), 4); // 1 and 3 tie at count 2; 1 is least recently used
    assert_eq!(c.get(&k("1")), None);
    assert_eq!(c.get(&k("3")), Some(3));
    assert_eq!(c.get(&k("4")), Some(4));
}

#[test]
fn should_return_stored_values_and_none_for_absent_keys() {
    let mut c = sut!(create(3));

    c.put(k("a"), 1);
    c.put(k("b"), 2);

    assert_eq!(c.get(&k("a")), Some(1));
    assert_eq!(c.get(&k("b")), Some(2));
    assert_eq!(c.get(&k("z")), None);
    assert_eq!(c.len(), 2);
}

#[test]
fn should_evict_the_least_frequently_used_entry() {
    let mut c = sut!(create(2));
    c.put(k("a"), 1);
    c.put(k("b"), 2);
    c.get(&k("a"));

    c.put(k("c"), 3);

    assert_eq!(c.get(&k("b")), None);
    assert_eq!(c.get(&k("a")), Some(1));
    assert_eq!(c.get(&k("c")), Some(3));
}

#[test]
fn should_break_ties_by_least_recent_use() {
    let mut c = sut!(create(2));
    c.put(k("a"), 1);
    c.put(k("b"), 2);

    c.put(k("c"), 3);

    assert_eq!(c.get(&k("a")), None);
    assert_eq!(c.get(&k("b")), Some(2));
    assert_eq!(c.get(&k("c")), Some(3));
}

#[test]
fn should_count_put_of_an_existing_key_as_a_use_and_replace_the_value() {
    let mut c = sut!(create(2));
    c.put(k("a"), 1);
    c.put(k("b"), 2);

    c.put(k("a"), 10);
    c.put(k("c"), 3);

    assert_eq!(c.get(&k("b")), None);
    assert_eq!(c.get(&k("a")), Some(10));
    assert_eq!(c.get(&k("c")), Some(3));
    assert_eq!(c.len(), 2);
}

#[test]
fn should_not_evict_when_updating_an_existing_key_at_full_capacity() {
    let mut c = sut!(create(2));
    c.put(k("a"), 1);
    c.put(k("b"), 2);

    c.put(k("a"), 5);
    c.put(k("b"), 6);

    assert_eq!(c.len(), 2);
    assert_eq!(c.get(&k("a")), Some(5));
    assert_eq!(c.get(&k("b")), Some(6));
}

#[test]
fn should_start_new_keys_at_count_one_so_a_fresh_key_is_the_next_victim() {
    let mut c = sut!(create(2));
    c.put(k("a"), 1);
    c.get(&k("a"));
    c.get(&k("a")); // a: count 3
    c.put(k("b"), 2); // b: count 1

    c.put(k("c"), 3); // evicts b, not a

    assert_eq!(c.get(&k("b")), None);
    assert_eq!(c.get(&k("a")), Some(1));
    assert_eq!(c.get(&k("c")), Some(3));
}

#[test]
fn should_forget_the_count_of_a_removed_key() {
    let mut c = sut!(create(2));
    c.put(k("a"), 1);
    c.get(&k("a"));
    c.get(&k("a")); // a: count 3
    assert!(c.remove(&k("a")));
    c.put(k("a"), 1); // a: count 1 again
    c.put(k("b"), 2); // b: count 1, a is less recently used

    c.put(k("c"), 3); // tie at 1 -> evicts a

    assert_eq!(c.get(&k("a")), None);
    assert_eq!(c.get(&k("b")), Some(2));
    assert_eq!(c.get(&k("c")), Some(3));
}

#[test]
fn should_remove_entries_and_free_their_slots() {
    let mut c = sut!(create(2));
    c.put(k("a"), 1);
    c.put(k("b"), 2);

    assert!(c.remove(&k("a")));
    assert!(!c.remove(&k("a")));
    assert!(!c.remove(&k("never")));
    c.put(k("c"), 3);

    assert_eq!(c.len(), 2);
    assert_eq!(c.get(&k("b")), Some(2));
    assert_eq!(c.get(&k("c")), Some(3));
}

#[test]
fn should_not_disturb_counts_when_looking_up_absent_keys() {
    let mut c = sut!(create(2));
    c.put(k("a"), 1);
    c.put(k("b"), 2);

    assert_eq!(c.get(&k("zzz")), None);
    c.put(k("c"), 3);

    assert_eq!(c.get(&k("a")), None);
    assert_eq!(c.get(&k("b")), Some(2));
}

#[test]
fn should_work_with_capacity_one() {
    let mut c = sut!(create(1));

    c.put(k("a"), 1);
    c.get(&k("a"));
    c.put(k("b"), 2);

    assert_eq!(c.get(&k("a")), None);
    assert_eq!(c.get(&k("b")), Some(2));
    assert_eq!(c.len(), 1);
}

#[test]
fn should_reset_the_lowest_count_to_one_after_an_insert() {
    let mut c = sut!(create(2));
    c.put(k("a"), 1);
    c.put(k("b"), 2);
    for _ in 0..4 {
        c.get(&k("a"));
    }
    for _ in 0..4 {
        c.get(&k("b"));
    } // both at count 5, a less recently used

    c.put(k("c"), 3); // evicts a
    c.put(k("d"), 4); // c (count 1) must be the victim, not b

    assert_eq!(c.get(&k("c")), None);
    assert_eq!(c.get(&k("b")), Some(2));
    assert_eq!(c.get(&k("d")), Some(4));
    assert_eq!(c.get(&k("a")), None);
}

struct Entry {
    value: i32,
    count: u32,
    last_use: u64,
}

#[test]
fn should_agree_with_a_simple_model_on_random_operations() {
    let capacity = 3;
    let mut c = sut!(create(capacity));
    let mut model: Vec<(String, Entry)> = Vec::new();
    let mut tick = 0u64;
    let mut seed = 11u64;
    let mut next = |bound: usize| {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((seed >> 33) as usize) % bound
    };

    for step in 0..6000 {
        let key = format!("k{}", next(7));
        match next(3) {
            0 => {
                let mut expected = None;
                if let Some((_, e)) = model.iter_mut().find(|(name, _)| *name == key) {
                    tick += 1;
                    e.count += 1;
                    e.last_use = tick;
                    expected = Some(e.value);
                }
                assert_eq!(c.get(&key), expected, "step {step} get {key}");
            }
            1 => {
                tick += 1;
                if let Some((_, e)) = model.iter_mut().find(|(name, _)| *name == key) {
                    e.value = step;
                    e.count += 1;
                    e.last_use = tick;
                } else {
                    if model.len() == capacity {
                        let victim = (0..model.len())
                            .min_by_key(|&i| (model[i].1.count, model[i].1.last_use))
                            .unwrap();
                        model.remove(victim);
                    }
                    model.push((
                        key.clone(),
                        Entry {
                            value: step,
                            count: 1,
                            last_use: tick,
                        },
                    ));
                }
                c.put(key, step);
            }
            _ => {
                let expected = match model.iter().position(|(name, _)| *name == key) {
                    Some(p) => {
                        model.remove(p);
                        true
                    }
                    None => false,
                };
                assert_eq!(c.remove(&key), expected, "step {step} remove {key}");
            }
        }
        assert_eq!(c.len(), model.len(), "step {step} len");
    }
}

#[test]
fn should_stay_fast_with_hundreds_of_thousands_of_entries() {
    let mut c = sut!(create(100_000));
    let start = Instant::now();

    for i in 0..200_000 {
        c.put(format!("k{i}"), i);
    }
    assert_eq!(c.len(), 100_000);
    assert_eq!(c.get(&k("k0")), None);
    assert_eq!(c.get(&k("k99999")), None);
    assert_eq!(c.get(&k("k199999")), Some(199_999));

    for i in 0..1_000_000usize {
        let key = format!("k{}", i * 7919 % 150_000);
        if i % 4 == 0 {
            c.put(key, i as i32);
        } else {
            c.get(&key);
        }
    }
    assert_eq!(c.len(), 100_000);

    assert!(start.elapsed() < Duration::from_secs(10));
}
