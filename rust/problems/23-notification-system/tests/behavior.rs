#![allow(unused_imports, clippy::needless_range_loop)]
use p23_notification_system::{contract::*, create};
use playground_support::sut;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type Call = (String, String, String); // channel, address, body

/// Records every attempt and refuses the first N attempts per channel when scripted to.
#[derive(Default)]
struct FakeGateway {
    calls: Mutex<Vec<Call>>,
    refusals_left: Mutex<HashMap<String, usize>>,
}

impl FakeGateway {
    fn refuse_first(&self, channel: &str, attempts: usize) {
        self.refusals_left
            .lock()
            .unwrap()
            .insert(channel.to_string(), attempts);
    }

    fn always_refuse(&self, channel: &str) {
        self.refuse_first(channel, usize::MAX);
    }

    fn calls(&self) -> Vec<Call> {
        self.calls.lock().unwrap().clone()
    }

    fn channels_called(&self) -> Vec<String> {
        self.calls().into_iter().map(|c| c.0).collect()
    }

    fn bodies(&self) -> Vec<String> {
        self.calls().into_iter().map(|c| c.2).collect()
    }
}

impl DeliveryGateway for FakeGateway {
    fn deliver(&self, channel: &str, address: &str, body: &str) -> bool {
        self.calls.lock().unwrap().push((
            channel.to_string(),
            address.to_string(),
            body.to_string(),
        ));
        let mut left = self.refusals_left.lock().unwrap();
        match left.get_mut(channel) {
            Some(n) if *n > 0 => {
                *n -= 1;
                false
            }
            _ => true,
        }
    }
}

fn call(channel: &str, address: &str, body: &str) -> Call {
    (channel.to_string(), address.to_string(), body.to_string())
}

fn system(gateway: &Arc<FakeGateway>) -> Option<Box<dyn NotificationApi>> {
    let mut api = create(gateway.clone())?;
    api.register_user(
        "u1",
        &[
            ("email", "a@x.com"),
            ("sms", "555-0100"),
            ("push", "device-1"),
        ],
    );
    api.define_template("otp", "Your code is {code}");
    Some(api)
}

#[test]
fn should_deliver_on_the_first_channel_with_the_rendered_body() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    api.define_template("welcome", "Hi {name}, your code is {code}.");

    let result = api.send(
        "u1",
        "security",
        "welcome",
        &[("name", "Asha"), ("code", "42")],
        &["email", "sms"],
    );

    assert_eq!(result, "DELIVERED:email");
    assert_eq!(
        gw.calls(),
        vec![call("email", "a@x.com", "Hi Asha, your code is 42.")]
    );
}

#[test]
fn should_follow_the_callers_channel_order() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));

    let result = api.send("u1", "security", "otp", &[("code", "1")], &["sms", "email"]);

    assert_eq!(result, "DELIVERED:sms");
    assert_eq!(gw.calls(), vec![call("sms", "555-0100", "Your code is 1")]);
}

#[test]
fn should_retry_the_same_channel_up_to_three_attempts() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    gw.refuse_first("email", 2);

    let result = api.send("u1", "security", "otp", &[("code", "1")], &["email", "sms"]);

    assert_eq!(result, "DELIVERED:email");
    assert_eq!(gw.channels_called(), vec!["email", "email", "email"]);
}

#[test]
fn should_fall_back_to_the_next_channel_after_three_refused_attempts() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    gw.always_refuse("email");

    let result = api.send("u1", "security", "otp", &[("code", "1")], &["email", "sms"]);

    assert_eq!(result, "DELIVERED:sms");
    assert_eq!(gw.channels_called(), vec!["email", "email", "email", "sms"]);
}

#[test]
fn should_report_failed_when_every_channel_refuses_all_attempts() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    gw.always_refuse("email");
    gw.always_refuse("sms");

    let result = api.send("u1", "security", "otp", &[("code", "1")], &["email", "sms"]);

    assert_eq!(result, "FAILED");
    assert_eq!(gw.calls().len(), 6);
}

#[test]
fn should_give_every_send_a_fresh_set_of_attempts() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    gw.refuse_first("email", 3);

    assert_eq!(
        api.send("u1", "security", "otp", &[("code", "1")], &["email"]),
        "FAILED"
    );
    assert_eq!(gw.calls().len(), 3);

    assert_eq!(
        api.send("u1", "security", "otp", &[("code", "2")], &["email"]),
        "DELIVERED:email"
    );
    assert_eq!(gw.calls().len(), 4);
}

#[test]
fn should_skip_channels_the_user_has_no_address_for() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    api.register_user("u2", &[("sms", "555-0199")]);

    let result = api.send("u2", "security", "otp", &[("code", "1")], &["email", "sms"]);

    assert_eq!(result, "DELIVERED:sms");
    assert_eq!(gw.channels_called(), vec!["sms"]);
}

#[test]
fn should_skip_channels_switched_off_for_the_category_only() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    api.set_preference("u1", "email", "marketing", false);

    assert_eq!(
        api.send(
            "u1",
            "marketing",
            "otp",
            &[("code", "1")],
            &["email", "sms"]
        ),
        "DELIVERED:sms"
    );
    assert_eq!(
        api.send("u1", "security", "otp", &[("code", "1")], &["email", "sms"]),
        "DELIVERED:email"
    );
    assert_eq!(gw.channels_called(), vec!["sms", "email"]);
}

#[test]
fn should_allow_switching_a_channel_back_on() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    api.set_preference("u1", "email", "marketing", false);
    api.set_preference("u1", "email", "marketing", true);

    let result = api.send("u1", "marketing", "otp", &[("code", "1")], &["email"]);

    assert_eq!(result, "DELIVERED:email");
}

#[test]
fn should_suppress_without_calling_the_gateway_when_nothing_is_eligible() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    api.set_preference("u1", "email", "marketing", false);
    api.set_preference("u1", "sms", "marketing", false);

    assert_eq!(
        api.send(
            "u1",
            "marketing",
            "otp",
            &[("code", "1")],
            &["email", "sms"]
        ),
        "SUPPRESSED"
    );
    assert_eq!(
        api.send("u1", "security", "otp", &[("code", "1")], &["fax"]),
        "SUPPRESSED"
    );

    assert!(gw.calls().is_empty());
}

#[test]
fn should_return_invalid_without_calling_the_gateway() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));

    assert_eq!(
        api.send("u1", "security", "otp", &[], &["email"]),
        "INVALID"
    );
    assert_eq!(
        api.send("u1", "security", "otp", &[("other", "x")], &["email"]),
        "INVALID"
    );
    assert_eq!(
        api.send("u1", "security", "no-such-template", &[], &["email"]),
        "INVALID"
    );
    assert_eq!(
        api.send("u1", "security", "otp", &[("code", "1")], &[]),
        "INVALID"
    );

    assert!(gw.calls().is_empty());
}

#[test]
fn should_return_unknown_user_without_calling_the_gateway() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));

    assert_eq!(
        api.send("ghost", "security", "otp", &[("code", "1")], &["email"]),
        "UNKNOWN_USER"
    );

    assert!(gw.calls().is_empty());
}

#[test]
fn should_replace_every_occurrence_and_ignore_extra_parameters() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    api.define_template("echo", "{a}-{a}-{b}");

    api.send(
        "u1",
        "security",
        "echo",
        &[("a", "1"), ("b", "2"), ("unused", "9")],
        &["email"],
    );

    assert_eq!(gw.bodies(), vec!["1-1-2"]);
}

#[test]
fn should_use_the_latest_template_definition() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    api.define_template("otp", "New code: {code}");

    api.send("u1", "security", "otp", &[("code", "7")], &["email"]);

    assert_eq!(gw.bodies(), vec!["New code: 7"]);
}

#[test]
fn should_try_a_channel_listed_twice_only_once() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    gw.always_refuse("email");

    let result = api.send(
        "u1",
        "security",
        "otp",
        &[("code", "1")],
        &["email", "email", "sms"],
    );

    assert_eq!(result, "DELIVERED:sms");
    assert_eq!(gw.channels_called(), vec!["email", "email", "email", "sms"]);
}

#[test]
fn should_replace_addresses_but_keep_preferences_when_registering_again() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    api.set_preference("u1", "email", "marketing", false);

    api.register_user("u1", &[("email", "new@x.com")]);

    assert_eq!(
        api.send("u1", "marketing", "otp", &[("code", "1")], &["email"]),
        "SUPPRESSED"
    );
    assert_eq!(
        api.send("u1", "security", "otp", &[("code", "1")], &["email", "sms"]),
        "DELIVERED:email"
    );
    assert_eq!(
        gw.calls(),
        vec![call("email", "new@x.com", "Your code is 1")]
    );
}

#[test]
fn should_keep_preferences_per_user() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    api.register_user("u2", &[("email", "b@x.com")]);
    api.set_preference("u1", "email", "marketing", false);

    assert_eq!(
        api.send("u2", "marketing", "otp", &[("code", "1")], &["email"]),
        "DELIVERED:email"
    );
    assert_eq!(
        api.send("u1", "marketing", "otp", &[("code", "1")], &["email"]),
        "SUPPRESSED"
    );
}
