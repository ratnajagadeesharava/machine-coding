#![allow(unused_imports, clippy::needless_range_loop)]
use p111_config_management::{contract::*, create};
use playground_support::{run_concurrently, sut, FakeClock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const START: u64 = 10_000;

fn config(clock: &FakeClock) -> Option<Arc<dyn ConfigService>> {
    create(clock.as_clock())
}

fn view(ns: &str, key: &str, value: &str, version: u64, updated_at: u64) -> String {
    format!("{ns}/{key}={value} v{version} t{updated_at}")
}

fn version_of(view: &str) -> u64 {
    let at = view.rfind(" v").unwrap();
    let rest = &view[at + 2..];
    rest[..rest.find(' ').unwrap()].parse().unwrap()
}

#[test]
fn should_report_invalid_input_before_a_version_mismatch_or_a_missing_config() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app", "a", "1").unwrap();
    let bad = Err(ConfigError::InvalidArgument);

    assert_eq!(c.update("app", "a", &"x".repeat(257), 99), bad);
    assert_eq!(c.update("app", "Bad Key", "x", 1), bad);
    assert_eq!(c.update("app", "missing", &"x".repeat(257), 1), bad);
    assert_eq!(c.update(" ", "missing", "x", 1), bad);
    assert_eq!(c.history("app", "a").unwrap().len(), 1);
}

#[test]
fn should_keep_the_timestamps_of_old_versions_when_rolling_back_later() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app", "a", "one").unwrap();
    clock.advance(100);
    c.update("app", "a", "two", 1).unwrap();
    clock.advance(100);

    assert_eq!(c.rollback("app", "a", 1), Ok(3));
    clock.advance(100);
    assert_eq!(c.rollback("app", "a", 2), Ok(4));
    assert_eq!(c.rollback("app", "a", 3), Ok(5));

    assert_eq!(
        c.history("app", "a").unwrap(),
        vec![
            view("app", "a", "one", 1, 10_000),
            view("app", "a", "two", 2, 10_100),
            view("app", "a", "one", 3, 10_200),
            view("app", "a", "two", 4, 10_300),
            view("app", "a", "one", 5, 10_300),
        ]
    );
}

#[test]
fn should_round_trip_values_that_look_like_the_view_format() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    let ns = "weird ns/with=chars";
    let values = [
        "a=b",
        "x v9 t9",
        "line1\nline2",
        "päth/with space ✓",
        " leading and trailing ",
        "/",
        "a/b=c v1 t2",
    ];
    for (i, v) in values.iter().enumerate() {
        c.create(ns, &format!("k{i}"), v).unwrap();
    }
    for (i, v) in values.iter().enumerate() {
        assert_eq!(
            c.get(ns, &format!("k{i}")),
            Ok(Some(view(ns, &format!("k{i}"), v, 1, START)))
        );
    }
    assert_eq!(c.list(ns).unwrap().len(), values.len());
}

#[test]
fn should_keep_versions_contiguous_when_updates_and_rollbacks_race_on_one_key() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app", "hot", "seed").unwrap();
    let updates = AtomicUsize::new(0);
    let rollbacks = AtomicUsize::new(0);

    run_concurrently(8, |t| {
        for i in 0..300usize {
            let h = c.history("app", "hot").unwrap();
            if t % 2 == 0 {
                let v = version_of(h.last().unwrap());
                if c.update("app", "hot", &format!("u{t}-{i}"), v).unwrap() {
                    updates.fetch_add(1, Ordering::SeqCst);
                }
            } else {
                let to = 1 + (i % h.len()) as u64;
                c.rollback("app", "hot", to).unwrap();
                rollbacks.fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    let history = c.history("app", "hot").unwrap();
    assert_eq!(
        history.len(),
        1 + updates.load(Ordering::SeqCst) + rollbacks.load(Ordering::SeqCst)
    );
    for (i, h) in history.iter().enumerate() {
        assert_eq!(version_of(h), i as u64 + 1);
    }
    assert_eq!(
        c.get("app", "hot"),
        Ok(Some(history.last().unwrap().clone()))
    );
}

#[test]
fn should_always_list_a_sorted_consistent_snapshot_while_configs_are_being_added() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    let writers = 4;
    let per_writer = 250;

    run_concurrently(writers + 2, |t| {
        if t < writers {
            for i in 0..per_writer {
                c.create("big", &format!("k{t}-{i:04}"), "v").unwrap();
            }
            return;
        }
        for _ in 0..200 {
            let seen = c.list("big").unwrap();
            let mut sorted = seen.clone();
            sorted.sort();
            assert_eq!(seen, sorted);
        }
    });

    assert_eq!(c.list("big").unwrap().len(), writers * per_writer);
}

#[test]
fn should_handle_delete_and_recreate_racing_without_breaking_version_rules() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app", "flip", "0").unwrap();

    run_concurrently(6, |t| {
        for _ in 0..400 {
            match t % 3 {
                0 => {
                    c.delete("app", "flip").unwrap();
                }
                1 => match c.create("app", "flip", "c") {
                    Ok(()) | Err(ConfigError::AlreadyExists) => {}
                    Err(e) => panic!("unexpected {e:?}"),
                },
                _ => match c.update("app", "flip", "u", 1) {
                    Ok(_) | Err(ConfigError::NotFound) => {}
                    Err(e) => panic!("unexpected {e:?}"),
                },
            }
        }
    });

    let history = c.history("app", "flip").unwrap();
    for (i, h) in history.iter().enumerate() {
        assert_eq!(version_of(h), i as u64 + 1);
    }
    assert_eq!(c.get("app", "flip").unwrap().is_some(), !history.is_empty());
}

#[test]
fn should_keep_get_cheap_after_long_histories() {
    let clock = FakeClock::new(START);
    let c = sut!(config(&clock));
    c.create("app", "long", "0").unwrap();
    for i in 1..=5_000u64 {
        assert_eq!(c.update("app", "long", &format!("v{i}"), i), Ok(true));
    }

    let _guard = playground_support::watchdog(20_000);
    for _ in 0..20_000 {
        c.get("app", "long").unwrap();
    }

    assert_eq!(
        c.get("app", "long"),
        Ok(Some(view("app", "long", "v5000", 5_001, START)))
    );
    assert_eq!(c.history("app", "long").unwrap().len(), 5_001);
    assert_eq!(c.rollback("app", "long", 2_500), Ok(5_002));
    assert_eq!(
        c.get("app", "long"),
        Ok(Some(view("app", "long", "v2499", 5_002, START)))
    );
}
