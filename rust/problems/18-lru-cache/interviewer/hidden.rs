#![allow(unused_imports, clippy::needless_range_loop)]
use p18_lru_cache::{contract::*, create};
use playground_support::sut;

fn k(s: &str) -> String {
    s.to_string()
}

#[test]
fn capacity_one_should_keep_only_the_newest_key() {
    let mut c = sut!(create(1));
    c.put(k("a"), 1);
    c.put(k("b"), 2);
    c.put(k("c"), 3);
    assert_eq!(c.len(), 1);
    assert_eq!(c.get(&k("a")), None);
    assert_eq!(c.get(&k("b")), None);
    assert_eq!(c.get(&k("c")), Some(3));
}

#[test]
fn a_reinserted_removed_key_should_be_the_most_recently_used() {
    let mut c = sut!(create(2));
    c.put(k("a"), 1);
    c.put(k("b"), 2);
    c.remove(&k("a"));
    c.put(k("a"), 5); // order now: b, a
    c.put(k("c"), 3); // evicts b
    assert_eq!(c.get(&k("b")), None);
    assert_eq!(c.get(&k("a")), Some(5));
    assert_eq!(c.get(&k("c")), Some(3));
}

#[test]
fn repeated_gets_of_the_same_key_should_keep_it_alive_against_a_stream_of_inserts() {
    let mut c = sut!(create(3));
    c.put(k("hot"), 1);
    for i in 0..50 {
        c.put(format!("cold{i}"), i);
        assert_eq!(c.get(&k("hot")), Some(1));
    }
    assert_eq!(c.len(), 3);
    assert_eq!(c.get(&k("cold48")), Some(48));
    assert_eq!(c.get(&k("cold49")), Some(49));
    assert_eq!(c.get(&k("cold47")), None);
}

#[test]
fn get_order_should_determine_who_is_evicted_next() {
    let mut c = sut!(create(3));
    c.put(k("a"), 1);
    c.put(k("b"), 2);
    c.put(k("c"), 3);
    c.get(&k("a"));
    c.get(&k("c"));
    c.get(&k("b")); // use order oldest..newest: a, c, b
    c.put(k("d"), 4); // evicts a
    c.put(k("e"), 5); // evicts c
    assert_eq!(c.get(&k("a")), None);
    assert_eq!(c.get(&k("c")), None);
    assert_eq!(c.get(&k("b")), Some(2));
    assert_eq!(c.get(&k("d")), Some(4));
    assert_eq!(c.get(&k("e")), Some(5));
}

#[test]
fn removing_everything_should_leave_an_empty_reusable_cache() {
    let mut c = sut!(create(2));
    c.put(k("a"), 1);
    c.put(k("b"), 2);
    assert!(c.remove(&k("a")));
    assert!(c.remove(&k("b")));
    assert_eq!(c.len(), 0);
    c.put(k("x"), 1);
    c.put(k("y"), 2);
    assert_eq!(c.len(), 2);
    assert_eq!(c.get(&k("x")), Some(1));
}
