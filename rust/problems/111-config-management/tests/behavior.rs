#![allow(unused_imports, clippy::needless_range_loop)]
use p111_config_management::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut, FakeClock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const START: u64 = 1_000;

fn config(clock: &FakeClock) -> Option<Arc<dyn ConfigService>> {
    create(clock.as_clock())
}

fn view(ns: &str, key: &str, value: &str, version: u64, updated_at: u64) -> String {
    format!("{ns}/{key}={value} v{version} t{updated_at}")
}

fn current_version(c: &Arc<dyn ConfigService>, ns: &str, key: &str) -> u64 {
    let v = c.get(ns, key).unwrap().unwrap();
    let at = v.rfind(" v").unwrap();
    let rest = &v[at + 2..];
    rest[..rest.find(' ').unwrap()].parse().unwrap()
}

#[test]
fn should_create_a_config_at_version_one_stamped_with_the_clock() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app/web", "db.host", "localhost").unwrap();

    assert_eq!(
        c.get("app/web", "db.host"),
        Ok(Some(view("app/web", "db.host", "localhost", 1, START)))
    );
}

#[test]
fn should_return_none_for_a_config_that_does_not_exist() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app", "a", "1").unwrap();

    assert_eq!(c.get("app", "b"), Ok(None));
    assert_eq!(c.get("other", "a"), Ok(None));
}

#[test]
fn should_reject_creating_a_duplicate_and_keep_the_original() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app", "a", "1").unwrap();
    clock.advance(10);

    assert_eq!(c.create("app", "a", "2"), Err(ConfigError::AlreadyExists));
    assert_eq!(c.get("app", "a"), Ok(Some(view("app", "a", "1", 1, START))));
    assert_eq!(c.history("app", "a").unwrap().len(), 1);
}

#[test]
fn should_update_when_the_expected_version_matches() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app", "a", "1").unwrap();
    clock.advance(500);

    assert_eq!(c.update("app", "a", "2", 1), Ok(true));
    assert_eq!(
        c.get("app", "a"),
        Ok(Some(view("app", "a", "2", 2, START + 500)))
    );
    assert_eq!(c.update("app", "a", "3", 2), Ok(true));
    assert_eq!(
        c.get("app", "a"),
        Ok(Some(view("app", "a", "3", 3, START + 500)))
    );
}

#[test]
fn should_return_false_and_change_nothing_when_the_expected_version_is_stale() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app", "a", "1").unwrap();
    assert_eq!(c.update("app", "a", "2", 1), Ok(true));
    clock.advance(100);

    assert_eq!(c.update("app", "a", "stale", 1), Ok(false));
    assert_eq!(c.update("app", "a", "future", 3), Ok(false));
    assert_eq!(c.update("app", "a", "zero", 0), Ok(false));
    assert_eq!(c.get("app", "a"), Ok(Some(view("app", "a", "2", 2, START))));
    assert_eq!(c.history("app", "a").unwrap().len(), 2);
}

#[test]
fn should_bump_the_version_on_every_successful_update_even_when_the_value_is_unchanged() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app", "a", "same").unwrap();

    assert_eq!(c.update("app", "a", "same", 1), Ok(true));
    assert_eq!(c.update("app", "a", "same", 2), Ok(true));

    assert_eq!(
        c.get("app", "a"),
        Ok(Some(view("app", "a", "same", 3, START)))
    );
    assert_eq!(c.history("app", "a").unwrap().len(), 3);
}

#[test]
fn should_fail_to_update_a_config_that_does_not_exist() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));

    assert_eq!(
        c.update("app", "missing", "x", 1),
        Err(ConfigError::NotFound)
    );
    assert_eq!(c.get("app", "missing"), Ok(None));
}

#[test]
fn should_list_a_namespace_sorted_by_key() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app", "zeta", "z").unwrap();
    c.create("app", "alpha", "a").unwrap();
    c.create("app", "mid.key", "m").unwrap();
    c.create("app", "mid-key", "d").unwrap();
    c.create("app", "1st", "one").unwrap();
    c.update("app", "alpha", "a2", 1).unwrap();

    assert_eq!(
        c.list("app").unwrap(),
        vec![
            view("app", "1st", "one", 1, START),
            view("app", "alpha", "a2", 2, START),
            view("app", "mid-key", "d", 1, START),
            view("app", "mid.key", "m", 1, START),
            view("app", "zeta", "z", 1, START),
        ]
    );
}

#[test]
fn should_keep_namespaces_isolated_from_each_other() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app/web", "timeout", "30").unwrap();
    c.create("app/api", "timeout", "60").unwrap();
    c.create("app/api", "retries", "3").unwrap();

    assert_eq!(
        c.get("app/web", "timeout"),
        Ok(Some(view("app/web", "timeout", "30", 1, START)))
    );
    assert_eq!(c.list("app/web").unwrap().len(), 1);
    assert_eq!(c.list("app/api").unwrap().len(), 2);
    assert!(c.list("app").unwrap().is_empty());
    assert_eq!(c.update("app/api", "timeout", "90", 1), Ok(true));
    assert_eq!(
        c.get("app/web", "timeout"),
        Ok(Some(view("app/web", "timeout", "30", 1, START)))
    );
}

#[test]
fn should_treat_namespaces_as_case_sensitive() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("App", "a", "upper").unwrap();
    c.create("app", "a", "lower").unwrap();

    assert_eq!(
        c.get("App", "a"),
        Ok(Some(view("App", "a", "upper", 1, START)))
    );
    assert_eq!(
        c.get("app", "a"),
        Ok(Some(view("app", "a", "lower", 1, START)))
    );
}

#[test]
fn should_return_an_empty_list_for_an_unknown_namespace() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));

    assert!(c.list("nobody").unwrap().is_empty());
    assert!(c.history("nobody", "a").unwrap().is_empty());
}

#[test]
fn should_return_history_oldest_first_including_the_current_version() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app", "a", "one").unwrap();
    clock.advance(10);
    c.update("app", "a", "two", 1).unwrap();
    clock.advance(10);
    c.update("app", "a", "three", 2).unwrap();

    assert_eq!(
        c.history("app", "a").unwrap(),
        vec![
            view("app", "a", "one", 1, START),
            view("app", "a", "two", 2, START + 10),
            view("app", "a", "three", 3, START + 20),
        ]
    );
}

#[test]
fn should_return_a_history_snapshot_that_later_changes_do_not_alter() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app", "a", "one").unwrap();
    let before = c.history("app", "a").unwrap();

    c.update("app", "a", "two", 1).unwrap();

    assert_eq!(before, vec![view("app", "a", "one", 1, START)]);
    assert_eq!(c.history("app", "a").unwrap().len(), 2);
}

#[test]
fn should_rollback_by_creating_a_new_version_with_the_old_value() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app", "a", "one").unwrap();
    c.update("app", "a", "two", 1).unwrap();
    c.update("app", "a", "three", 2).unwrap();
    clock.advance(250);

    assert_eq!(c.rollback("app", "a", 1), Ok(4));
    assert_eq!(
        c.get("app", "a"),
        Ok(Some(view("app", "a", "one", 4, START + 250)))
    );
    assert_eq!(
        c.history("app", "a").unwrap(),
        vec![
            view("app", "a", "one", 1, START),
            view("app", "a", "two", 2, START),
            view("app", "a", "three", 3, START),
            view("app", "a", "one", 4, START + 250),
        ]
    );
}

#[test]
fn should_allow_rolling_back_to_the_current_version() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app", "a", "one").unwrap();

    assert_eq!(c.rollback("app", "a", 1), Ok(2));
    assert_eq!(
        c.get("app", "a"),
        Ok(Some(view("app", "a", "one", 2, START)))
    );
}

#[test]
fn should_reject_rollback_to_a_version_that_never_existed_or_of_a_missing_config() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app", "a", "one").unwrap();
    c.update("app", "a", "two", 1).unwrap();

    assert_eq!(c.rollback("app", "a", 0), Err(ConfigError::InvalidArgument));
    assert_eq!(c.rollback("app", "a", 3), Err(ConfigError::InvalidArgument));
    assert_eq!(c.rollback("app", "nope", 1), Err(ConfigError::NotFound));
    assert_eq!(
        c.get("app", "a"),
        Ok(Some(view("app", "a", "two", 2, START)))
    );
    assert_eq!(c.history("app", "a").unwrap().len(), 2);
}

#[test]
fn should_delete_a_config_and_report_whether_it_existed() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app", "a", "1").unwrap();
    c.create("app", "b", "2").unwrap();

    assert_eq!(c.delete("app", "a"), Ok(true));
    assert_eq!(c.delete("app", "a"), Ok(false));
    assert_eq!(c.delete("app", "never"), Ok(false));
    assert_eq!(c.delete("nobody", "a"), Ok(false));
    assert_eq!(c.get("app", "a"), Ok(None));
    assert!(c.history("app", "a").unwrap().is_empty());
    assert_eq!(
        c.list("app").unwrap(),
        vec![view("app", "b", "2", 1, START)]
    );
    assert_eq!(c.update("app", "a", "x", 1), Err(ConfigError::NotFound));
}

#[test]
fn should_start_over_at_version_one_with_fresh_history_when_recreated_after_delete() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app", "a", "old").unwrap();
    c.update("app", "a", "older", 1).unwrap();
    c.delete("app", "a").unwrap();
    clock.advance(70);

    c.create("app", "a", "new").unwrap();

    assert_eq!(
        c.get("app", "a"),
        Ok(Some(view("app", "a", "new", 1, START + 70)))
    );
    assert_eq!(
        c.history("app", "a").unwrap(),
        vec![view("app", "a", "new", 1, START + 70)]
    );
    assert_eq!(c.update("app", "a", "x", 1), Ok(true));
}

#[test]
fn should_reject_blank_namespaces_and_blank_keys_on_every_operation() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app", "a", "1").unwrap();
    let bad = Err(ConfigError::InvalidArgument);

    for ns in ["", "   ", "\t"] {
        assert_eq!(c.create(ns, "a", "v"), bad);
        assert_eq!(c.update(ns, "a", "v", 1), Err(ConfigError::InvalidArgument));
        assert_eq!(c.get(ns, "a"), Err(ConfigError::InvalidArgument));
        assert_eq!(c.delete(ns, "a"), Err(ConfigError::InvalidArgument));
        assert_eq!(c.list(ns), Err(ConfigError::InvalidArgument));
        assert_eq!(c.history(ns, "a"), Err(ConfigError::InvalidArgument));
        assert_eq!(c.rollback(ns, "a", 1), Err(ConfigError::InvalidArgument));
    }
    for key in ["", "  "] {
        assert_eq!(c.create("app", key, "v"), bad);
        assert_eq!(
            c.update("app", key, "v", 1),
            Err(ConfigError::InvalidArgument)
        );
        assert_eq!(c.get("app", key), Err(ConfigError::InvalidArgument));
        assert_eq!(c.delete("app", key), Err(ConfigError::InvalidArgument));
        assert_eq!(c.history("app", key), Err(ConfigError::InvalidArgument));
        assert_eq!(c.rollback("app", key, 1), Err(ConfigError::InvalidArgument));
    }
    assert_eq!(c.list("app").unwrap().len(), 1);
}

#[test]
fn should_reject_keys_with_characters_outside_lowercase_alphanumeric_dot_underscore_dash() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));

    for bad in [
        "Upper",
        "has space",
        "slash/key",
        "colon:key",
        "eq=ual",
        "ünï",
        "tab\tkey",
        "a,b",
    ] {
        assert_eq!(
            c.create("app", bad, "v"),
            Err(ConfigError::InvalidArgument),
            "{bad}"
        );
    }
    for ok in ["a", "0", "a.b", "a_b", "a-b", "_", ".", "-", "x1.y2_z3-w4"] {
        c.create("app", ok, "v").unwrap();
    }
    assert_eq!(c.list("app").unwrap().len(), 9);
}

#[test]
fn should_accept_values_up_to_256_characters_and_reject_longer_ones() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    let max = "v".repeat(256);

    c.create("app", "max", &max).unwrap();
    c.create("app", "empty", "").unwrap();
    assert_eq!(
        c.get("app", "max"),
        Ok(Some(view("app", "max", &max, 1, START)))
    );
    assert_eq!(
        c.get("app", "empty"),
        Ok(Some(view("app", "empty", "", 1, START)))
    );

    assert_eq!(
        c.create("app", "long", &"v".repeat(257)),
        Err(ConfigError::InvalidArgument)
    );
    assert_eq!(
        c.update("app", "max", &"v".repeat(257), 1),
        Err(ConfigError::InvalidArgument)
    );
    assert_eq!(c.get("app", "long"), Ok(None));
    assert_eq!(
        c.get("app", "max"),
        Ok(Some(view("app", "max", &max, 1, START)))
    );
}

#[test]
fn should_not_change_anything_when_an_update_fails_validation() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app", "a", "good").unwrap();
    clock.advance(5);

    assert_eq!(
        c.update("app", "a", &"x".repeat(300), 1),
        Err(ConfigError::InvalidArgument)
    );

    assert_eq!(
        c.get("app", "a"),
        Ok(Some(view("app", "a", "good", 1, START)))
    );
    assert_eq!(c.history("app", "a").unwrap().len(), 1);
    assert_eq!(c.update("app", "a", "next", 1), Ok(true));
}

#[test]
fn should_stamp_each_version_with_the_clock_at_the_moment_it_was_written() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app", "a", "1").unwrap();
    clock.set(5_000);
    c.update("app", "a", "2", 1).unwrap();
    clock.set(9_000);

    assert_eq!(c.get("app", "a"), Ok(Some(view("app", "a", "2", 2, 5_000))));
    assert_eq!(
        c.history("app", "a").unwrap()[0],
        view("app", "a", "1", 1, START)
    );
}

#[test]
fn should_let_exactly_one_of_many_threads_create_the_same_config() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    let created = AtomicUsize::new(0);
    let rejected = AtomicUsize::new(0);

    run_concurrently(16, |t| {
        match c.create("app", "contested", &format!("writer-{t}")) {
            Ok(()) => created.fetch_add(1, Ordering::SeqCst),
            Err(ConfigError::AlreadyExists) => rejected.fetch_add(1, Ordering::SeqCst),
            Err(e) => panic!("unexpected {e:?}"),
        };
    });

    assert_eq!(created.load(Ordering::SeqCst), 1);
    assert_eq!(rejected.load(Ordering::SeqCst), 15);
    assert_eq!(c.history("app", "contested").unwrap().len(), 1);
}

#[test]
fn should_let_exactly_one_thread_win_when_all_use_the_same_expected_version() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app", "a", "start").unwrap();

    let results = collect_concurrently(16, |t| {
        c.update("app", "a", &format!("writer-{t}"), 1).unwrap()
    });

    assert_eq!(results.iter().filter(|r| **r).count(), 1);
    assert_eq!(c.history("app", "a").unwrap().len(), 2);
    assert!(c.get("app", "a").unwrap().unwrap().contains(" v2 "));
}

#[test]
fn should_lose_no_update_when_threads_retry_on_version_conflicts() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app", "counter", "0").unwrap();
    let threads = 8;
    let per_thread = 250;

    run_concurrently(threads, |t| {
        for i in 0..per_thread {
            loop {
                let version = current_version(&c, "app", "counter");
                if c.update("app", "counter", &format!("t{t}-{i}"), version)
                    .unwrap()
                {
                    break;
                }
            }
        }
    });

    let expected = 1 + (threads * per_thread) as u64;
    assert_eq!(current_version(&c, "app", "counter"), expected);
    let history = c.history("app", "counter").unwrap();
    assert_eq!(history.len() as u64, expected);
    for (i, h) in history.iter().enumerate() {
        assert!(h.contains(&format!(" v{} ", i + 1)), "{h}");
    }
}

#[test]
fn should_create_configs_in_different_namespaces_concurrently_without_losing_any() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));

    run_concurrently(8, |t| {
        for i in 0..200 {
            c.create(&format!("ns-{}", i % 4), &format!("k-{t}-{i}"), "v")
                .unwrap();
        }
    });

    let total: usize = (0..4)
        .map(|n| c.list(&format!("ns-{n}")).unwrap().len())
        .sum();
    assert_eq!(total, 8 * 200);
}
