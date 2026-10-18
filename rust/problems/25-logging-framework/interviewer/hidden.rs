#![allow(unused_imports, clippy::needless_range_loop)]
use p25_logging_framework::{contract::*, create};
use playground_support::{sut, FakeClock};
use std::sync::{Arc, Mutex};

struct Collect(Arc<Mutex<Vec<String>>>);

impl LogSink for Collect {
    fn write(&self, line: &str) {
        self.0.lock().unwrap().push(line.to_string());
    }
}

fn logging(clock: &FakeClock, lines: &Arc<Mutex<Vec<String>>>) -> Option<Box<dyn LoggingApi>> {
    let mut api = create(clock.as_clock())?;
    api.add_appender("main", Arc::new(Collect(lines.clone())), "TRACE");
    Some(api)
}

fn got(lines: &Arc<Mutex<Vec<String>>>) -> Vec<String> {
    lines.lock().unwrap().clone()
}

#[test]
fn a_deep_logger_should_inherit_through_missing_intermediate_loggers() {
    let clock = FakeClock::new(42);
    let lines = Arc::new(Mutex::new(Vec::new()));
    let mut api = sut!(logging(&clock, &lines));
    api.set_level("app", "ERROR");

    api.log("app.db.pool.conn", "WARN", "dropped", &[]);
    api.log("app.db.pool.conn", "ERROR", "kept", &[]);

    assert_eq!(got(&lines), vec!["42 [ERROR] app.db.pool.conn - kept"]);
}

#[test]
fn a_sibling_with_a_common_prefix_should_not_inherit_from_the_other_sibling() {
    let clock = FakeClock::new(42);
    let lines = Arc::new(Mutex::new(Vec::new()));
    let mut api = sut!(logging(&clock, &lines));
    api.set_level("app.db", "ERROR");

    api.log("app.dbx", "INFO", "kept", &[]);
    api.log("app.db.pool", "INFO", "dropped", &[]);

    assert_eq!(got(&lines), vec!["42 [INFO] app.dbx - kept"]);
}

#[test]
fn adjacent_placeholders_and_templates_without_placeholders_should_work() {
    let clock = FakeClock::new(42);
    let lines = Arc::new(Mutex::new(Vec::new()));
    let mut api = sut!(logging(&clock, &lines));

    api.log("app", "INFO", "{}{}{}", &["a", "b", "c"]);
    api.log("app", "INFO", "{}{}", &["a"]);
    api.log("app", "INFO", "plain", &["ignored"]);
    api.log("app", "INFO", "", &[]);

    assert_eq!(
        got(&lines),
        vec![
            "42 [INFO] app - abc",
            "42 [INFO] app - a{}",
            "42 [INFO] app - plain",
            "42 [INFO] app - "
        ]
    );
}

#[test]
fn an_argument_containing_a_placeholder_should_be_copied_verbatim() {
    let clock = FakeClock::new(42);
    let lines = Arc::new(Mutex::new(Vec::new()));
    let mut api = sut!(logging(&clock, &lines));

    api.log("app", "INFO", "{} then {}", &["{} {}", "end"]);

    assert_eq!(got(&lines), vec!["42 [INFO] app - {} {} then end"]);
}

#[test]
fn setting_the_same_level_twice_should_keep_the_last_and_the_root_name_should_be_usable() {
    let clock = FakeClock::new(42);
    let lines = Arc::new(Mutex::new(Vec::new()));
    let mut api = sut!(logging(&clock, &lines));
    api.set_level("root", "ERROR");
    api.set_level("root", "TRACE");
    api.set_level("app", "WARN");
    api.set_level("app", "DEBUG");

    api.log("app.x", "DEBUG", "kept", &[]);
    api.log("other", "TRACE", "kept too", &[]);

    assert_eq!(
        got(&lines),
        vec!["42 [DEBUG] app.x - kept", "42 [TRACE] other - kept too"]
    );
}
