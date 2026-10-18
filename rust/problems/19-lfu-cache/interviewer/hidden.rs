#![allow(unused_imports, clippy::needless_range_loop)]
use p19_lfu_cache::{contract::*, create};
use playground_support::sut;

fn k(s: &str) -> String {
    s.to_string()
}

#[test]
fn an_evicted_key_should_come_back_with_count_one() {
    let mut c = sut!(create(2));
    c.put(k("a"), 1);
    c.put(k("b"), 2);
    c.get(&k("a"));
    c.get(&k("b")); // a: 2, b: 2 (a older)
    c.put(k("c"), 3); // evicts a
    c.put(k("a"), 1); // evicts c (count 1); a is back at count 1
    c.put(k("d"), 4); // a (count 1) must go, not b (count 2)
    assert_eq!(c.get(&k("b")), Some(2));
    assert_eq!(c.get(&k("d")), Some(4));
    assert_eq!(c.get(&k("a")), None);
}

#[test]
fn frequency_should_beat_recency() {
    let mut c = sut!(create(2));
    c.put(k("old"), 1);
    c.get(&k("old"));
    c.put(k("fresh"), 2); // newer but count 1
    c.put(k("newer"), 3); // evicts fresh
    assert_eq!(c.get(&k("fresh")), None);
    assert_eq!(c.get(&k("old")), Some(1));
    assert_eq!(c.get(&k("newer")), Some(3));
}

#[test]
fn tie_break_should_follow_the_order_of_most_recent_gets_and_puts() {
    let mut c = sut!(create(3));
    c.put(k("a"), 1);
    c.put(k("b"), 2);
    c.put(k("c"), 3);
    c.get(&k("a"));
    c.get(&k("b"));
    c.get(&k("c")); // all at count 2; recency a < b < c
    c.get(&k("a")); // a: 3
    c.put(k("d"), 4); // lowest count 2 is shared by b and c; b is older
    assert_eq!(c.get(&k("b")), None);
    assert_eq!(c.get(&k("a")), Some(1));
    assert_eq!(c.get(&k("c")), Some(3));
    assert_eq!(c.get(&k("d")), Some(4));
}

#[test]
fn repeated_puts_of_the_same_key_should_make_it_popular() {
    let mut c = sut!(create(2));
    c.put(k("a"), 1);
    c.put(k("a"), 2);
    c.put(k("a"), 3);
    c.put(k("b"), 10);
    c.put(k("c"), 11); // evicts b
    assert_eq!(c.get(&k("a")), Some(3));
    assert_eq!(c.get(&k("b")), None);
    assert_eq!(c.get(&k("c")), Some(11));
}

#[test]
fn removing_the_only_low_count_entry_should_make_the_next_lowest_the_victim() {
    let mut c = sut!(create(3));
    c.put(k("a"), 1);
    c.put(k("b"), 2);
    c.put(k("c"), 3);
    c.get(&k("a"));
    c.get(&k("a")); // a: 3
    c.get(&k("b")); // b: 2, c: 1
    assert!(c.remove(&k("c")));
    c.put(k("d"), 4); // room available, nothing evicted
    assert_eq!(c.len(), 3);
    c.put(k("e"), 5); // d (count 1) goes
    assert_eq!(c.get(&k("d")), None);
    assert_eq!(c.get(&k("a")), Some(1));
    assert_eq!(c.get(&k("b")), Some(2));
    assert_eq!(c.get(&k("e")), Some(5));
}
