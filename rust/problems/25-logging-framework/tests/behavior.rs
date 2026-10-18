#![allow(unused_imports, clippy::needless_range_loop)]
use p25_logging_framework::{contract::*, create};
use playground_support::{sut, FakeClock};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const T0: u64 = 1_700_000_000_123;

type Lines = Arc<Mutex<Vec<String>>>;

struct Collect(Lines);

impl LogSink for Collect {
    fn write(&self, line: &str) {
        self.0.lock().unwrap().push(line.to_string());
    }
}

/// Appends a fixed tag instead of the line (used to observe delivery order).
struct Tag(Lines, &'static str);

impl LogSink for Tag {
    fn write(&self, _line: &str) {
        self.0.lock().unwrap().push(self.1.to_string());
    }
}

struct Failing;

impl LogSink for Failing {
    fn write(&self, _line: &str) {
        panic!("disk full");
    }
}

fn new_lines() -> Lines {
    Arc::new(Mutex::new(Vec::new()))
}

fn collect(lines: &Lines) -> Arc<dyn LogSink> {
    Arc::new(Collect(lines.clone()))
}

fn got(lines: &Lines) -> Vec<String> {
    lines.lock().unwrap().clone()
}

/// A logging system with one appender "main" (min level TRACE) collecting into `lines`.
fn collecting(clock: &FakeClock, lines: &Lines) -> Option<Box<dyn LoggingApi>> {
    let mut api = create(clock.as_clock())?;
    assert!(api.add_appender("main", collect(lines), "TRACE"));
    Some(api)
}

#[test]
fn should_format_line_with_timestamp_level_logger_and_message() {
    let clock = FakeClock::new(T0);
    let lines = new_lines();
    let mut api = sut!(collecting(&clock, &lines));

    api.log("app", "INFO", "started", &[]);

    assert_eq!(got(&lines), vec!["1700000000123 [INFO] app - started"]);
}

#[test]
fn should_use_the_clock_at_the_moment_of_each_call() {
    let clock = FakeClock::new(T0);
    let lines = new_lines();
    let mut api = sut!(collecting(&clock, &lines));

    api.log("app", "INFO", "one", &[]);
    clock.advance(7);
    api.log("app", "ERROR", "two", &[]);

    assert_eq!(
        got(&lines),
        vec![
            "1700000000123 [INFO] app - one",
            "1700000000130 [ERROR] app - two"
        ]
    );
}

#[test]
fn should_substitute_placeholders_in_order() {
    let clock = FakeClock::new(T0);
    let lines = new_lines();
    let mut api = sut!(collecting(&clock, &lines));

    api.log(
        "app.db",
        "WARN",
        "slow query {}ms on {}",
        &["850", "orders"],
    );

    assert_eq!(
        got(&lines),
        vec!["1700000000123 [WARN] app.db - slow query 850ms on orders"]
    );
}

#[test]
fn should_leave_unfilled_placeholders_when_arguments_are_missing() {
    let clock = FakeClock::new(T0);
    let lines = new_lines();
    let mut api = sut!(collecting(&clock, &lines));

    api.log("app", "INFO", "a={} b={} c={}", &["1"]);

    assert_eq!(
        got(&lines),
        vec!["1700000000123 [INFO] app - a=1 b={} c={}"]
    );
}

#[test]
fn should_ignore_extra_arguments() {
    let clock = FakeClock::new(T0);
    let lines = new_lines();
    let mut api = sut!(collecting(&clock, &lines));

    api.log("app", "INFO", "only {}", &["x", "y", "z"]);
    api.log("app", "INFO", "none", &["x"]);

    assert_eq!(
        got(&lines),
        vec![
            "1700000000123 [INFO] app - only x",
            "1700000000123 [INFO] app - none"
        ]
    );
}

#[test]
fn should_not_rescan_substituted_arguments_for_placeholders() {
    let clock = FakeClock::new(T0);
    let lines = new_lines();
    let mut api = sut!(collecting(&clock, &lines));

    api.log("app", "INFO", "x={} y={}", &["{}", "2"]);

    assert_eq!(got(&lines), vec!["1700000000123 [INFO] app - x={} y=2"]);
}

#[test]
fn should_copy_anything_that_is_not_exactly_a_placeholder() {
    let clock = FakeClock::new(T0);
    let lines = new_lines();
    let mut api = sut!(collecting(&clock, &lines));

    api.log("app", "INFO", "{ } { } } {{}} $1 \\ {}", &["A"]);

    assert_eq!(
        got(&lines),
        vec!["1700000000123 [INFO] app - { } { } } {A} $1 \\ {}"]
    );
}

#[test]
fn should_default_the_root_threshold_to_info() {
    let clock = FakeClock::new(T0);
    let lines = new_lines();
    let mut api = sut!(collecting(&clock, &lines));

    api.log("app", "TRACE", "t", &[]);
    api.log("app", "DEBUG", "d", &[]);
    api.log("app", "INFO", "i", &[]);
    api.log("app", "WARN", "w", &[]);
    api.log("app", "ERROR", "e", &[]);

    let out = got(&lines);
    assert_eq!(out.len(), 3);
    assert!(out[0].ends_with("[INFO] app - i"));
    assert!(out[1].ends_with("[WARN] app - w"));
    assert!(out[2].ends_with("[ERROR] app - e"));
}

#[test]
fn should_rank_levels_from_trace_to_error() {
    let clock = FakeClock::new(T0);
    let lines = new_lines();
    let mut api = sut!(collecting(&clock, &lines));
    api.set_level("root", "TRACE");
    let levels = ["TRACE", "DEBUG", "INFO", "WARN", "ERROR"];

    for (threshold, level_name) in levels.iter().enumerate() {
        lines.lock().unwrap().clear();
        api.set_level("app", level_name);
        for level in levels {
            api.log("app", level, "m", &[]);
        }
        assert_eq!(
            got(&lines).len(),
            levels.len() - threshold,
            "threshold {level_name}"
        );
    }
}

#[test]
fn should_apply_a_threshold_set_on_the_root_to_loggers_without_own_level() {
    let clock = FakeClock::new(T0);
    let lines = new_lines();
    let mut api = sut!(collecting(&clock, &lines));

    assert!(api.set_level("root", "ERROR"));
    api.log("app", "WARN", "dropped", &[]);
    api.log("app", "ERROR", "kept", &[]);

    let out = got(&lines);
    assert_eq!(out.len(), 1);
    assert!(out[0].ends_with("[ERROR] app - kept"));
}

#[test]
fn should_inherit_the_threshold_of_the_nearest_configured_ancestor() {
    let clock = FakeClock::new(T0);
    let lines = new_lines();
    let mut api = sut!(collecting(&clock, &lines));
    api.set_level("app", "ERROR");

    api.log("app.db", "WARN", "dropped", &[]);
    api.log("app.db.pool", "WARN", "dropped too", &[]);
    api.log("app.db.pool", "ERROR", "kept", &[]);

    let out = got(&lines);
    assert_eq!(out.len(), 1);
    assert!(out[0].ends_with("[ERROR] app.db.pool - kept"));
}

#[test]
fn should_let_a_more_specific_logger_override_its_ancestors() {
    let clock = FakeClock::new(T0);
    let lines = new_lines();
    let mut api = sut!(collecting(&clock, &lines));
    api.set_level("app", "ERROR");
    api.set_level("app.db", "DEBUG");

    api.log("app", "DEBUG", "dropped", &[]);
    api.log("app.db", "DEBUG", "kept", &[]);
    api.log("app.db.pool", "DEBUG", "kept too", &[]);

    let out = got(&lines);
    assert_eq!(out.len(), 2);
    assert!(out[0].ends_with("[DEBUG] app.db - kept"));
    assert!(out[1].ends_with("[DEBUG] app.db.pool - kept too"));
}

#[test]
fn should_treat_only_whole_segments_as_parents() {
    let clock = FakeClock::new(T0);
    let lines = new_lines();
    let mut api = sut!(collecting(&clock, &lines));
    api.set_level("app", "ERROR");

    api.log("application", "INFO", "kept: not a child of app", &[]);
    api.log("app.db", "INFO", "dropped: child of app", &[]);

    let out = got(&lines);
    assert_eq!(out.len(), 1);
    assert!(out[0].contains("application"));
}

#[test]
fn should_see_a_threshold_change_immediately_even_after_the_logger_was_used() {
    let clock = FakeClock::new(T0);
    let lines = new_lines();
    let mut api = sut!(collecting(&clock, &lines));
    api.log("app.db", "DEBUG", "dropped", &[]);
    assert!(got(&lines).is_empty());

    api.set_level("app", "DEBUG");
    api.log("app.db", "DEBUG", "kept", &[]);
    api.set_level("app", "WARN");
    api.log("app.db", "DEBUG", "dropped again", &[]);

    let out = got(&lines);
    assert_eq!(out.len(), 1);
    assert!(out[0].ends_with("kept"));
}

#[test]
fn should_filter_each_appender_by_its_own_minimum_level_on_top_of_the_logger_threshold() {
    let clock = FakeClock::new(T0);
    let mut api = sut!(create(clock.as_clock()));
    let (warnings, everything) = (new_lines(), new_lines());
    api.add_appender("warnings", collect(&warnings), "WARN");
    api.add_appender("everything", collect(&everything), "TRACE");
    api.set_level("root", "DEBUG");

    api.log("app", "TRACE", "below logger threshold", &[]);
    api.log("app", "DEBUG", "d", &[]);
    api.log("app", "WARN", "w", &[]);

    let w = got(&warnings);
    assert_eq!(w.len(), 1);
    assert!(w[0].ends_with("[WARN] app - w"));
    assert_eq!(got(&everything).len(), 2);
}

#[test]
fn should_deliver_to_every_appender_in_registration_order() {
    let clock = FakeClock::new(T0);
    let mut api = sut!(create(clock.as_clock()));
    let order = new_lines();
    api.add_appender("b", Arc::new(Tag(order.clone(), "b")), "TRACE");
    api.add_appender("a", Arc::new(Tag(order.clone(), "a")), "TRACE");
    api.add_appender("c", Arc::new(Tag(order.clone(), "c")), "TRACE");

    api.log("app", "INFO", "x", &[]);

    assert_eq!(got(&order), vec!["b", "a", "c"]);
}

#[test]
fn should_refuse_invalid_or_duplicate_appenders() {
    let clock = FakeClock::new(T0);
    let mut api = sut!(create(clock.as_clock()));
    let (first, second) = (new_lines(), new_lines());

    assert!(api.add_appender("x", collect(&first), "INFO"));
    assert!(!api.add_appender("x", collect(&second), "INFO"));
    assert!(!api.add_appender(" ", collect(&second), "INFO"));
    assert!(!api.add_appender("y", collect(&second), "LOUD"));

    api.log("app", "INFO", "m", &[]);
    assert_eq!(got(&first).len(), 1);
    assert!(got(&second).is_empty());
}

#[test]
fn should_stop_delivering_after_removal_and_allow_reusing_the_name() {
    let clock = FakeClock::new(T0);
    let mut api = sut!(create(clock.as_clock()));
    let (first, second) = (new_lines(), new_lines());
    api.add_appender("x", collect(&first), "INFO");

    assert!(api.remove_appender("x"));
    assert!(!api.remove_appender("x"));
    assert!(!api.remove_appender("unknown"));
    api.log("app", "INFO", "lost", &[]);
    assert!(api.add_appender("x", collect(&second), "INFO"));
    api.log("app", "INFO", "seen", &[]);

    assert!(got(&first).is_empty());
    let out = got(&second);
    assert_eq!(out.len(), 1);
    assert!(out[0].ends_with("seen"));
}

#[test]
fn should_not_show_earlier_messages_to_a_late_appender() {
    let clock = FakeClock::new(T0);
    let mut api = sut!(create(clock.as_clock()));
    api.log("app", "INFO", "before", &[]);
    let late = new_lines();

    api.add_appender("late", collect(&late), "TRACE");
    api.log("app", "INFO", "after", &[]);

    let out = got(&late);
    assert_eq!(out.len(), 1);
    assert!(out[0].ends_with("after"));
}

#[test]
fn should_keep_delivering_to_other_appenders_when_a_sink_fails() {
    let clock = FakeClock::new(T0);
    let mut api = sut!(create(clock.as_clock()));
    let (before, after) = (new_lines(), new_lines());
    api.add_appender("before", collect(&before), "TRACE");
    api.add_appender("broken", Arc::new(Failing), "TRACE");
    api.add_appender("after", collect(&after), "TRACE");

    api.log("app", "INFO", "one", &[]);
    api.log("app", "INFO", "two", &[]);

    assert_eq!(got(&before).len(), 2);
    assert_eq!(got(&after).len(), 2);
}

#[test]
fn should_ignore_invalid_levels_and_blank_logger_names() {
    let clock = FakeClock::new(T0);
    let lines = new_lines();
    let mut api = sut!(collecting(&clock, &lines));

    assert!(!api.set_level("app", "LOUD"));
    assert!(!api.set_level("app", "info"));
    assert!(!api.set_level(" ", "INFO"));
    api.log("app", "LOUD", "x", &[]);
    api.log("app", "info", "x", &[]);
    api.log("", "INFO", "x", &[]);
    api.log("  ", "INFO", "x", &[]);

    assert!(got(&lines).is_empty());
    api.log("app", "INFO", "valid", &[]);
    assert_eq!(got(&lines).len(), 1);
}

#[test]
fn should_go_nowhere_without_appenders() {
    let clock = FakeClock::new(T0);
    let mut api = sut!(create(clock.as_clock()));

    api.log("app", "ERROR", "nobody listens", &[]);
}

#[test]
fn should_be_fast_when_most_messages_are_below_the_threshold() {
    let clock = FakeClock::new(T0);
    let lines = new_lines();
    let mut api = sut!(collecting(&clock, &lines));
    api.set_level("a.b.c.d.e.f", "WARN");
    let start = Instant::now();

    for _ in 0..300_000 {
        api.log("a.b.c.d.e.f.g", "DEBUG", "item {} of {}", &["x", "y"]);
    }
    for _ in 0..100_000 {
        api.log("a.b.c.d.e.f.g", "ERROR", "item {} of {}", &["x", "y"]);
    }

    assert_eq!(got(&lines).len(), 100_000);
    assert!(start.elapsed() < Duration::from_secs(20));
}
