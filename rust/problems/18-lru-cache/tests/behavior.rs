#![allow(unused_imports, clippy::needless_range_loop)]
use p18_lru_cache::{contract::*, create};
use playground_support::sut;
use std::time::{Duration, Instant};

fn k(s: &str) -> String {
    s.to_string()
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
fn should_evict_the_least_recently_used_entry_on_overflow() {
    let mut c = sut!(create(2));

    c.put(k("a"), 1);
    c.put(k("b"), 2);
    c.put(k("c"), 3);

    assert_eq!(c.get(&k("a")), None);
    assert_eq!(c.get(&k("b")), Some(2));
    assert_eq!(c.get(&k("c")), Some(3));
    assert_eq!(c.len(), 2);
}

#[test]
fn should_treat_get_as_a_use() {
    let mut c = sut!(create(2));
    c.put(k("a"), 1);
    c.put(k("b"), 2);

    assert_eq!(c.get(&k("a")), Some(1));
    c.put(k("c"), 3);

    assert_eq!(c.get(&k("b")), None);
    assert_eq!(c.get(&k("a")), Some(1));
    assert_eq!(c.get(&k("c")), Some(3));
}

#[test]
fn should_replace_value_and_refresh_when_putting_an_existing_key() {
    let mut c = sut!(create(2));
    c.put(k("a"), 1);
    c.put(k("b"), 2);

    c.put(k("a"), 10);
    c.put(k("c"), 3);

    assert_eq!(c.get(&k("a")), Some(10));
    assert_eq!(c.get(&k("b")), None);
    assert_eq!(c.get(&k("c")), Some(3));
}

#[test]
fn should_not_evict_anything_when_updating_a_key_at_full_capacity() {
    let mut c = sut!(create(2));
    c.put(k("a"), 1);
    c.put(k("b"), 2);

    c.put(k("b"), 20);
    c.put(k("a"), 10);

    assert_eq!(c.len(), 2);
    assert_eq!(c.get(&k("a")), Some(10));
    assert_eq!(c.get(&k("b")), Some(20));
}

#[test]
fn should_not_disturb_recency_when_looking_up_absent_keys() {
    let mut c = sut!(create(2));
    c.put(k("a"), 1);
    c.put(k("b"), 2);

    assert_eq!(c.get(&k("zzz")), None);
    c.put(k("c"), 3);

    assert_eq!(c.get(&k("a")), None);
    assert_eq!(c.get(&k("b")), Some(2));
}

#[test]
fn should_remove_entries_and_report_whether_they_existed() {
    let mut c = sut!(create(3));
    c.put(k("a"), 1);
    c.put(k("b"), 2);

    assert!(c.remove(&k("a")));
    assert!(!c.remove(&k("a")));
    assert!(!c.remove(&k("never")));

    assert_eq!(c.get(&k("a")), None);
    assert_eq!(c.len(), 1);
}

#[test]
fn should_free_the_slot_of_a_removed_entry_immediately() {
    let mut c = sut!(create(2));
    c.put(k("a"), 1);
    c.put(k("b"), 2);

    c.remove(&k("a"));
    c.put(k("c"), 3);

    assert_eq!(c.get(&k("b")), Some(2));
    assert_eq!(c.get(&k("c")), Some(3));
    assert_eq!(c.len(), 2);
}

#[test]
fn should_work_with_capacity_one() {
    let mut c = sut!(create(1));

    c.put(k("a"), 1);
    c.put(k("a"), 2);
    assert_eq!(c.get(&k("a")), Some(2));
    c.put(k("b"), 3);

    assert_eq!(c.get(&k("a")), None);
    assert_eq!(c.get(&k("b")), Some(3));
    assert_eq!(c.len(), 1);
}

#[test]
fn should_evict_in_use_order_across_many_inserts() {
    let mut c = sut!(create(3));
    for i in 0..10 {
        c.put(format!("k{i}"), i);
    }

    assert_eq!(c.len(), 3);
    assert_eq!(c.get(&k("k6")), None);
    assert_eq!(c.get(&k("k7")), Some(7));
    assert_eq!(c.get(&k("k8")), Some(8));
    assert_eq!(c.get(&k("k9")), Some(9));
}

#[test]
fn should_agree_with_a_simple_model_on_random_operations() {
    let capacity = 5;
    let mut c = sut!(create(capacity));
    let mut model: Vec<(String, i32)> = Vec::new(); // front = least recently used
    let mut seed = 7u64;
    let mut next = |bound: usize| {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((seed >> 33) as usize) % bound
    };

    for step in 0..5000 {
        let key = format!("k{}", next(9));
        match next(3) {
            0 => {
                let expected = model.iter().position(|e| e.0 == key).map(|p| {
                    let e = model.remove(p);
                    let v = e.1;
                    model.push(e);
                    v
                });
                assert_eq!(c.get(&key), expected, "step {step} get {key}");
            }
            1 => {
                match model.iter().position(|e| e.0 == key) {
                    Some(p) => {
                        model.remove(p);
                    }
                    None if model.len() == capacity => {
                        model.remove(0);
                    }
                    None => {}
                }
                model.push((key.clone(), step));
                c.put(key, step);
            }
            _ => {
                let expected = match model.iter().position(|e| e.0 == key) {
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
    for (key, value) in &model {
        assert_eq!(c.get(key), Some(*value));
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
