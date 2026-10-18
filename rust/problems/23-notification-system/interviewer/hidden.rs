#![allow(unused_imports, clippy::needless_range_loop)]
use p23_notification_system::{contract::*, create};
use playground_support::sut;
use std::sync::{Arc, Mutex};

/// Accepts everything except the channels listed in `refused`; records channel and body.
struct Gateway {
    log: Mutex<Vec<String>>,
    refused: Vec<&'static str>,
}

impl Gateway {
    fn new(refused: &[&'static str]) -> Arc<Self> {
        Arc::new(Gateway {
            log: Mutex::new(Vec::new()),
            refused: refused.to_vec(),
        })
    }

    fn log(&self) -> Vec<String> {
        self.log.lock().unwrap().clone()
    }
}

impl DeliveryGateway for Gateway {
    fn deliver(&self, channel: &str, _address: &str, body: &str) -> bool {
        self.log.lock().unwrap().push(format!("{channel}|{body}"));
        !self.refused.contains(&channel)
    }
}

fn system(gateway: &Arc<Gateway>) -> Option<Box<dyn NotificationApi>> {
    let mut api = create(gateway.clone())?;
    api.register_user("u", &[("email", "e"), ("sms", "s"), ("push", "p")]);
    Some(api)
}

#[test]
fn parameter_values_should_never_be_expanded_again() {
    let gw = Gateway::new(&[]);
    let mut api = sut!(system(&gw));
    api.define_template("t", "Hi {name}");

    api.send(
        "u",
        "c",
        "t",
        &[("name", "{code}"), ("code", "1")],
        &["email"],
    );

    assert_eq!(gw.log(), vec!["email|Hi {code}"]);
}

#[test]
fn braces_that_are_not_placeholders_should_stay_literal() {
    let gw = Gateway::new(&[]);
    let mut api = sut!(system(&gw));
    api.define_template("t", "{} and {not valid} and {a}");

    let result = api.send("u", "c", "t", &[("a", "x")], &["email"]);

    assert_eq!(result, "DELIVERED:email");
    assert_eq!(gw.log(), vec!["email|{} and {not valid} and x"]);
}

#[test]
fn skipped_channel_between_failing_and_succeeding_ones_should_cost_no_calls() {
    let gw = Gateway::new(&["email"]);
    let mut api = sut!(system(&gw));
    api.define_template("t", "m");
    api.set_preference("u", "sms", "c", false);

    let result = api.send("u", "c", "t", &[], &["email", "sms", "push"]);

    assert_eq!(result, "DELIVERED:push");
    assert_eq!(gw.log(), vec!["email|m", "email|m", "email|m", "push|m"]);
}

#[test]
fn preference_set_for_unregistered_user_should_not_apply_to_a_later_registration() {
    let gw = Gateway::new(&[]);
    let mut api = sut!(system(&gw));
    api.define_template("t", "m");
    api.set_preference("ghost", "email", "c", false);

    api.register_user("ghost", &[("email", "g")]);

    assert_eq!(
        api.send("ghost", "c", "t", &[], &["email"]),
        "DELIVERED:email"
    );
}

#[test]
fn checks_should_run_in_the_order_unknown_user_invalid_suppressed() {
    let gw = Gateway::new(&[]);
    let mut api = sut!(system(&gw));
    api.define_template("t", "{x}");
    api.set_preference("u", "email", "c", false);
    api.register_user("empty", &[]);

    assert_eq!(api.send("nobody", "c", "missing", &[], &[]), "UNKNOWN_USER");
    assert_eq!(api.send("u", "c", "t", &[], &["email"]), "INVALID");
    assert_eq!(
        api.send("u", "c", "t", &[("x", "1")], &["email"]),
        "SUPPRESSED"
    );
    assert_eq!(
        api.send("empty", "c", "t", &[("x", "1")], &["email"]),
        "SUPPRESSED"
    );
    assert!(gw.log().is_empty());
}
