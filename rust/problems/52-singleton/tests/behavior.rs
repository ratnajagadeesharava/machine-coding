use p52_singleton::AppConfig;
use std::collections::HashSet;
use std::sync::{Arc, Barrier, Mutex, MutexGuard};
use std::thread;

const SOURCE: &str = include_str!("../src/lib.rs");

// The singleton is process-wide and tests run in parallel: take turns.
static TURN: Mutex<()> = Mutex::new(());

fn exclusive() -> MutexGuard<'static, ()> {
    let guard = TURN.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    // A poisoned or panicking earlier test must not leak its instance into this one.
    let _ = std::panic::catch_unwind(AppConfig::reset_for_tests);
    guard
}

#[test]
fn should_always_return_the_same_instance() {
    let _turn = exclusive();
    AppConfig::reset_for_tests();

    let a = AppConfig::instance();
    let b = AppConfig::instance();

    assert!(Arc::ptr_eq(&a, &b));
    assert_eq!(AppConfig::instances_created(), 1);
}

#[test]
fn should_share_state_between_every_caller() {
    let _turn = exclusive();
    AppConfig::reset_for_tests();
    AppConfig::instance().set("mode", "fast");

    assert_eq!(AppConfig::instance().get("mode").as_deref(), Some("fast"));
    assert_eq!(AppConfig::instance().get("missing"), None);
}

#[test]
fn should_create_the_instance_lazily() {
    let _turn = exclusive();
    AppConfig::reset_for_tests();
    assert_eq!(AppConfig::instances_created(), 0);

    AppConfig::instance();
    assert_eq!(AppConfig::instances_created(), 1);
    AppConfig::instance();
    assert_eq!(AppConfig::instances_created(), 1);
}

#[test]
fn should_create_only_one_instance_even_when_many_threads_ask_at_the_same_time() {
    let _turn = exclusive();
    let threads = 16;
    for round in 0..300 {
        AppConfig::reset_for_tests();
        let barrier = Arc::new(Barrier::new(threads));
        let addresses: HashSet<usize> = thread::scope(|scope| {
            let handles: Vec<_> = (0..threads)
                .map(|_| {
                    let barrier = barrier.clone();
                    scope.spawn(move || {
                        barrier.wait();
                        Arc::as_ptr(&AppConfig::instance()) as usize
                    })
                })
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });

        assert_eq!(addresses.len(), 1, "distinct instances seen in round {round}");
        assert_eq!(AppConfig::instances_created(), 1, "constructor runs in round {round}");
    }
}

#[test]
fn should_keep_the_state_consistent_under_concurrent_writes() {
    let _turn = exclusive();
    AppConfig::reset_for_tests();
    let threads = 32;

    thread::scope(|scope| {
        for id in 0..threads {
            scope.spawn(move || {
                for k in 0..100 {
                    AppConfig::instance().set(&format!("key-{id}-{k}"), &format!("v{id}"));
                }
            });
        }
    });

    for id in 0..threads {
        for k in 0..100 {
            assert_eq!(AppConfig::instance().get(&format!("key-{id}-{k}")), Some(format!("v{id}")));
        }
    }
}

#[test]
fn should_give_a_fresh_instance_after_a_reset() {
    let _turn = exclusive();
    AppConfig::reset_for_tests();
    let first = AppConfig::instance();
    first.set("a", "1");

    AppConfig::reset_for_tests();
    let second = AppConfig::instance();

    assert!(!Arc::ptr_eq(&first, &second));
    assert_eq!(second.get("a"), None);
    assert_eq!(AppConfig::instances_created(), 1);
}

#[test]
fn should_offer_no_other_way_to_construct_it() {
    let _turn = exclusive();
    assert!(AppConfig::instance().get("x").is_none());

    let code: Vec<&str> = SOURCE.lines().map(str::trim).filter(|l| !l.starts_with("//")).collect();
    let joined = code.join("\n");
    for forbidden in ["pub fn new", "Default", "Clone"] {
        assert!(!joined.contains(forbidden), "AppConfig must not expose: {forbidden}");
    }
    let public_fields = code.iter().filter(|l| l.starts_with("pub ") && l.ends_with(',')).count();
    assert_eq!(public_fields, 0, "AppConfig must not have public fields");
}
