#![allow(unused_imports, clippy::needless_range_loop)]
use p21_key_value_store::{contract::*, create};
use playground_support::{sut, FakeClock};
use std::time::{Duration, Instant};

const NOW: u64 = 1_000_000;

fn store(clock: &FakeClock) -> Option<Box<dyn KeyValueStoreApi>> {
    create(clock.as_clock())
}

fn some(v: &str) -> Option<String> {
    Some(v.to_string())
}

#[test]
fn should_store_overwrite_and_read_values() {
    let clock = FakeClock::new(NOW);
    let mut kv = sut!(store(&clock));

    kv.put("a", "1");
    assert_eq!(kv.get("a"), some("1"));
    kv.put("a", "2");

    assert_eq!(kv.get("a"), some("2"));
    assert_eq!(kv.get("missing"), None);
    assert_eq!(kv.size(), 1);
}

#[test]
fn should_report_whether_delete_removed_a_key() {
    let clock = FakeClock::new(NOW);
    let mut kv = sut!(store(&clock));
    kv.put("a", "1");

    assert!(kv.delete("a"));
    assert!(!kv.delete("a"));
    assert!(!kv.delete("never"));
    assert_eq!(kv.get("a"), None);
    assert_eq!(kv.size(), 0);
}

#[test]
fn should_expire_key_exactly_when_ttl_elapses() {
    let clock = FakeClock::new(NOW);
    let mut kv = sut!(store(&clock));

    assert!(kv.put_with_ttl("session", "abc", 1000));

    clock.advance(999);
    assert_eq!(kv.get("session"), some("abc"));
    clock.advance(1);
    assert_eq!(kv.get("session"), None);
}

#[test]
fn should_treat_expired_key_as_missing_everywhere() {
    let clock = FakeClock::new(NOW);
    let mut kv = sut!(store(&clock));
    kv.put_with_ttl("k1", "v", 100);
    kv.put("k2", "v");

    clock.advance(100);

    assert!(!kv.delete("k1"));
    assert_eq!(kv.keys_with_prefix("k"), vec!["k2"]);
    assert_eq!(kv.size(), 1);
}

#[test]
fn should_drop_ttl_when_overwritten_by_plain_put() {
    let clock = FakeClock::new(NOW);
    let mut kv = sut!(store(&clock));
    kv.put_with_ttl("k", "v1", 100);

    kv.put("k", "v2");
    clock.advance(10_000);

    assert_eq!(kv.get("k"), some("v2"));
}

#[test]
fn should_restart_countdown_when_put_with_ttl_again() {
    let clock = FakeClock::new(NOW);
    let mut kv = sut!(store(&clock));
    kv.put_with_ttl("k", "v1", 1000);
    clock.advance(600);

    kv.put_with_ttl("k", "v2", 1000);
    clock.advance(600);
    assert_eq!(kv.get("k"), some("v2"));

    clock.advance(399);
    assert_eq!(kv.get("k"), some("v2"));
    clock.advance(1);
    assert_eq!(kv.get("k"), None);
}

#[test]
fn should_refuse_non_positive_ttl_without_changing_anything() {
    let clock = FakeClock::new(NOW);
    let mut kv = sut!(store(&clock));
    kv.put("k", "keep");

    assert!(!kv.put_with_ttl("k", "x", 0));
    assert!(!kv.put_with_ttl("k", "x", -5));
    assert!(!kv.put_with_ttl("new", "x", 0));

    assert_eq!(kv.get("k"), some("keep"));
    assert_eq!(kv.get("new"), None);
    assert_eq!(kv.size(), 1);
}

#[test]
fn should_allow_reusing_an_expired_key() {
    let clock = FakeClock::new(NOW);
    let mut kv = sut!(store(&clock));
    kv.put_with_ttl("k", "old", 10);
    clock.advance(10);

    kv.put("k", "new");

    assert_eq!(kv.get("k"), some("new"));
    assert_eq!(kv.size(), 1);
}

#[test]
fn should_list_keys_by_prefix_in_ascending_order() {
    let clock = FakeClock::new(NOW);
    let mut kv = sut!(store(&clock));
    for k in ["user:2", "user:10", "order:1", "user:1", "usr", "u"] {
        kv.put(k, "v");
    }

    assert_eq!(
        kv.keys_with_prefix("user:"),
        vec!["user:1", "user:10", "user:2"]
    );
    assert_eq!(
        kv.keys_with_prefix("u"),
        vec!["u", "user:1", "user:10", "user:2", "usr"]
    );
    assert_eq!(
        kv.keys_with_prefix(""),
        vec!["order:1", "u", "user:1", "user:10", "user:2", "usr"]
    );
    assert!(kv.keys_with_prefix("zzz").is_empty());
}

#[test]
fn should_count_only_live_keys() {
    let clock = FakeClock::new(NOW);
    let mut kv = sut!(store(&clock));
    kv.put("a", "1");
    kv.put("b", "2");
    kv.put_with_ttl("c", "3", 50);
    assert_eq!(kv.size(), 3);

    clock.advance(50);
    kv.delete("a");

    assert_eq!(kv.size(), 1);
}

#[test]
fn should_fail_commit_and_rollback_without_open_transaction() {
    let clock = FakeClock::new(NOW);
    let mut kv = sut!(store(&clock));

    assert!(!kv.commit());
    assert!(!kv.rollback());

    kv.begin();
    assert!(kv.rollback());
    assert!(!kv.rollback());
    kv.begin();
    assert!(kv.commit());
    assert!(!kv.commit());
}

#[test]
fn should_discard_writes_and_deletes_on_rollback() {
    let clock = FakeClock::new(NOW);
    let mut kv = sut!(store(&clock));
    kv.put("a", "1");
    kv.put("b", "2");

    kv.begin();
    kv.put("a", "changed");
    kv.delete("b");
    kv.put("c", "3");
    assert!(kv.rollback());

    assert_eq!(kv.get("a"), some("1"));
    assert_eq!(kv.get("b"), some("2"));
    assert_eq!(kv.get("c"), None);
    assert_eq!(kv.keys_with_prefix(""), vec!["a", "b"]);
    assert_eq!(kv.size(), 2);
}

#[test]
fn should_show_transaction_writes_and_deletes_to_reads_inside_it() {
    let clock = FakeClock::new(NOW);
    let mut kv = sut!(store(&clock));
    kv.put("a", "1");
    kv.put("b", "2");

    kv.begin();
    kv.put("a", "changed");
    kv.delete("b");
    kv.put("c", "3");

    assert_eq!(kv.get("a"), some("changed"));
    assert_eq!(kv.get("b"), None);
    assert_eq!(kv.get("c"), some("3"));
    assert_eq!(kv.keys_with_prefix(""), vec!["a", "c"]);
    assert_eq!(kv.size(), 2);
    assert!(!kv.delete("b"));
}

#[test]
fn should_keep_changes_when_outermost_transaction_commits() {
    let clock = FakeClock::new(NOW);
    let mut kv = sut!(store(&clock));
    kv.put("a", "1");
    kv.put("b", "2");

    kv.begin();
    kv.put("a", "changed");
    kv.delete("b");
    kv.put("c", "3");
    assert!(kv.commit());

    assert_eq!(kv.get("a"), some("changed"));
    assert_eq!(kv.get("b"), None);
    assert_eq!(kv.get("c"), some("3"));
    assert!(!kv.rollback());
    assert_eq!(kv.size(), 2);
}

#[test]
fn should_roll_back_only_the_innermost_transaction() {
    let clock = FakeClock::new(NOW);
    let mut kv = sut!(store(&clock));

    kv.begin();
    kv.put("outer", "1");
    kv.begin();
    kv.put("inner", "2");
    kv.put("outer", "overwritten");
    assert!(kv.rollback());

    assert_eq!(kv.get("inner"), None);
    assert_eq!(kv.get("outer"), some("1"));
    assert!(kv.commit());
    assert_eq!(kv.get("outer"), some("1"));
}

#[test]
fn should_merge_inner_commit_into_parent_only() {
    let clock = FakeClock::new(NOW);
    let mut kv = sut!(store(&clock));

    kv.begin();
    kv.put("x", "1");
    kv.begin();
    kv.put("x", "2");
    assert!(kv.commit());

    assert_eq!(kv.get("x"), some("2"));
    assert!(kv.rollback());
    assert_eq!(kv.get("x"), None);
    assert_eq!(kv.size(), 0);
}

#[test]
fn should_make_inner_commit_permanent_only_when_outermost_commits() {
    let clock = FakeClock::new(NOW);
    let mut kv = sut!(store(&clock));

    kv.begin();
    kv.begin();
    kv.put("x", "deep");
    kv.commit();
    assert_eq!(kv.get("x"), some("deep"));
    kv.commit();

    assert_eq!(kv.get("x"), some("deep"));
    assert!(!kv.rollback());
}

#[test]
fn should_keep_ttl_deadline_from_the_put_call_when_transaction_commits_later() {
    let clock = FakeClock::new(NOW);
    let mut kv = sut!(store(&clock));

    kv.begin();
    kv.put_with_ttl("k", "v", 1000);
    clock.advance(600);
    kv.commit();

    clock.advance(399);
    assert_eq!(kv.get("k"), some("v"));
    clock.advance(1);
    assert_eq!(kv.get("k"), None);
}

#[test]
fn should_hide_key_that_expires_while_transaction_is_open() {
    let clock = FakeClock::new(NOW);
    let mut kv = sut!(store(&clock));
    kv.begin();
    kv.put_with_ttl("k", "v", 100);
    kv.put("stay", "v");

    clock.advance(100);

    assert_eq!(kv.get("k"), None);
    assert_eq!(kv.keys_with_prefix(""), vec!["stay"]);
    assert_eq!(kv.size(), 1);
    assert!(kv.rollback());
    assert_eq!(kv.size(), 0);
}

#[test]
fn should_restore_overwritten_value_with_its_original_deadline_on_rollback() {
    let clock = FakeClock::new(NOW);
    let mut kv = sut!(store(&clock));
    kv.put_with_ttl("k", "v1", 1000);
    clock.advance(500);

    kv.begin();
    kv.put("k", "v2");
    kv.rollback();

    assert_eq!(kv.get("k"), some("v1"));
    clock.advance(499);
    assert_eq!(kv.get("k"), some("v1"));
    clock.advance(1);
    assert_eq!(kv.get("k"), None);
}

#[test]
fn should_not_resurrect_key_that_expired_during_rolled_back_transaction() {
    let clock = FakeClock::new(NOW);
    let mut kv = sut!(store(&clock));
    kv.put_with_ttl("k", "v", 100);

    kv.begin();
    kv.delete("k");
    clock.advance(200);
    kv.rollback();

    assert_eq!(kv.get("k"), None);
    assert_eq!(kv.size(), 0);
}

#[test]
fn should_handle_large_stores_quickly() {
    let clock = FakeClock::new(NOW);
    let mut kv = sut!(store(&clock));
    let start = Instant::now();

    for i in 0..50_000 {
        kv.put(&format!("key:{i:05}"), &format!("v{i}"));
    }
    for i in 0..50_000 {
        assert_eq!(kv.get(&format!("key:{i:05}")), Some(format!("v{i}")));
    }
    for i in 0..100 {
        assert_eq!(kv.keys_with_prefix(&format!("key:{:04}", i * 10)).len(), 10);
        assert_eq!(kv.size(), 50_000);
    }
    kv.begin();
    for i in 0..50_000 {
        kv.delete(&format!("key:{i:05}"));
    }
    assert_eq!(kv.size(), 0);
    kv.rollback();
    assert_eq!(kv.size(), 50_000);

    assert!(start.elapsed() < Duration::from_secs(10));
}
