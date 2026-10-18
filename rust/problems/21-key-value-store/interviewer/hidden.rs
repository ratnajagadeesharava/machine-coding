#![allow(unused_imports, clippy::needless_range_loop)]
use p21_key_value_store::{contract::*, create};
use playground_support::{sut, FakeClock};

fn store(clock: &FakeClock) -> Option<Box<dyn KeyValueStoreApi>> {
    create(clock.as_clock())
}

#[test]
fn delete_in_committed_inner_transaction_should_be_undone_by_outer_rollback() {
    let clock = FakeClock::new(5_000);
    let mut kv = sut!(store(&clock));
    kv.put("k", "v");

    kv.begin();
    kv.begin();
    kv.delete("k");
    kv.commit();
    assert_eq!(kv.get("k"), None);
    kv.rollback();

    assert_eq!(kv.get("k"), Some("v".to_string()));
    assert_eq!(kv.size(), 1);
}

#[test]
fn plain_put_inside_transaction_should_remove_ttl_once_committed() {
    let clock = FakeClock::new(5_000);
    let mut kv = sut!(store(&clock));
    kv.put_with_ttl("k", "v1", 1000);

    kv.begin();
    kv.put("k", "v2");
    kv.commit();
    clock.advance(5000);

    assert_eq!(kv.get("k"), Some("v2".to_string()));
}

#[test]
fn ttl_started_inside_transaction_should_not_be_extended_by_commit() {
    let clock = FakeClock::new(5_000);
    let mut kv = sut!(store(&clock));

    kv.begin();
    kv.begin();
    kv.put_with_ttl("k", "v", 100);
    clock.advance(100);
    kv.commit();
    kv.commit();

    assert_eq!(kv.get("k"), None);
    assert_eq!(kv.size(), 0);
    assert!(kv.keys_with_prefix("").is_empty());
}

#[test]
fn ttl_put_inside_rolled_back_transaction_should_leave_old_permanent_value_alone() {
    let clock = FakeClock::new(5_000);
    let mut kv = sut!(store(&clock));
    kv.put("k", "v1");

    kv.begin();
    kv.put_with_ttl("k", "v2", 10);
    kv.rollback();
    clock.advance(10_000);

    assert_eq!(kv.get("k"), Some("v1".to_string()));
}

#[test]
fn prefix_listing_and_size_should_merge_all_open_layers() {
    let clock = FakeClock::new(5_000);
    let mut kv = sut!(store(&clock));
    kv.put("p:b", "1");
    kv.put("p:d", "1");
    kv.put("q:x", "1");

    kv.begin();
    kv.put("p:a", "2");
    kv.delete("p:b");
    kv.begin();
    kv.put("p:c", "3");
    kv.delete("p:d");
    kv.put("p:b", "back");

    assert_eq!(kv.keys_with_prefix("p:"), vec!["p:a", "p:b", "p:c"]);
    assert_eq!(kv.size(), 4);

    kv.rollback();
    assert_eq!(kv.keys_with_prefix("p:"), vec!["p:a", "p:d"]);
    assert_eq!(kv.size(), 3);
}
