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

#[test]
fn should_send_only_once_when_a_duplicated_channel_is_delivered() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));

    let result = api.send(
        "u1",
        "security",
        "otp",
        &[("code", "1")],
        &["email", "email", "sms"],
    );

    assert_eq!(result, "DELIVERED:email");
    assert_eq!(gw.calls(), vec![call("email", "a@x.com", "Your code is 1")]);
}

#[test]
fn should_fail_after_three_attempts_when_the_only_channel_is_listed_twice() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    gw.always_refuse("email");

    let result = api.send("u1", "security", "otp", &[("code", "1")], &["email", "email"]);

    assert_eq!(result, "FAILED");
    assert_eq!(gw.channels_called(), vec!["email", "email", "email"]);
}

#[test]
fn should_keep_the_first_position_of_a_duplicated_channel() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    gw.always_refuse("sms");
    gw.always_refuse("email");

    let result = api.send(
        "u1",
        "security",
        "otp",
        &[("code", "1")],
        &["sms", "email", "sms", "push"],
    );

    assert_eq!(result, "DELIVERED:push");
    assert_eq!(
        gw.channels_called(),
        vec!["sms", "sms", "sms", "email", "email", "email", "push"]
    );
}

#[test]
fn should_not_call_a_disabled_channel_listed_twice() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    api.set_preference("u1", "email", "security", false);

    let result = api.send(
        "u1",
        "security",
        "otp",
        &[("code", "1")],
        &["email", "email", "sms"],
    );

    assert_eq!(result, "DELIVERED:sms");
    assert_eq!(gw.channels_called(), vec!["sms"]);
}

#[test]
fn should_treat_channel_names_as_case_sensitive() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));

    assert_eq!(
        api.send("u1", "security", "otp", &[("code", "1")], &["EMAIL", "Sms"]),
        "SUPPRESSED"
    );
    assert_eq!(
        api.send("u1", "security", "otp", &[("code", "1")], &["Email", "email"]),
        "DELIVERED:email"
    );

    assert_eq!(gw.channels_called(), vec!["email"]);
}

#[test]
fn should_deliver_on_any_channel_name_the_user_has_an_address_for() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    api.register_user(
        "u3",
        &[("whatsapp", "+1-555-0100"), ("carrier-pigeon", "roof-7")],
    );

    let result = api.send(
        "u3",
        "security",
        "otp",
        &[("code", "1")],
        &["telegram", "carrier-pigeon", "whatsapp"],
    );

    assert_eq!(result, "DELIVERED:carrier-pigeon");
    assert_eq!(
        gw.calls(),
        vec![call("carrier-pigeon", "roof-7", "Your code is 1")]
    );
}

#[test]
fn should_hand_the_same_address_and_body_to_every_retry() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    gw.refuse_first("email", 2);

    api.send("u1", "security", "otp", &[("code", "9")], &["email"]);

    let expected = call("email", "a@x.com", "Your code is 9");
    assert_eq!(gw.calls(), vec![expected.clone(), expected.clone(), expected]);
}

#[test]
fn should_stop_calling_the_gateway_after_the_first_accepted_attempt() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    gw.refuse_first("email", 1);

    let result = api.send(
        "u1",
        "security",
        "otp",
        &[("code", "1")],
        &["email", "sms", "push"],
    );

    assert_eq!(result, "DELIVERED:email");
    assert_eq!(gw.channels_called(), vec!["email", "email"]);
}

#[test]
fn should_report_failed_not_suppressed_when_the_only_eligible_channel_refuses() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    api.set_preference("u1", "sms", "security", false);
    gw.always_refuse("email");

    let result = api.send(
        "u1",
        "security",
        "otp",
        &[("code", "1")],
        &["email", "sms", "fax"],
    );

    assert_eq!(result, "FAILED");
    assert_eq!(gw.channels_called(), vec!["email", "email", "email"]);
}

#[test]
fn should_forget_old_addresses_when_registering_again() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));

    api.register_user("u1", &[("email", "new@x.com")]);

    assert_eq!(
        api.send("u1", "security", "otp", &[("code", "1")], &["sms", "push"]),
        "SUPPRESSED"
    );
    assert!(gw.calls().is_empty());

    api.register_user("u1", &[]);
    assert_eq!(
        api.send("u1", "security", "otp", &[("code", "1")], &["email"]),
        "SUPPRESSED"
    );
    assert!(gw.calls().is_empty());
}

#[test]
fn should_keep_preferences_independent_per_channel_and_category() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    api.set_preference("u1", "email", "marketing", false);
    api.set_preference("u1", "sms", "security", false);

    assert_eq!(
        api.send("u1", "marketing", "otp", &[("code", "1")], &["email", "sms"]),
        "DELIVERED:sms"
    );
    assert_eq!(
        api.send("u1", "security", "otp", &[("code", "1")], &["sms", "email"]),
        "DELIVERED:email"
    );
    assert_eq!(
        api.send("u1", "updates", "otp", &[("code", "1")], &["sms", "email"]),
        "DELIVERED:sms"
    );
    assert_eq!(gw.channels_called(), vec!["sms", "email", "sms"]);
}

#[test]
fn should_apply_preference_changes_to_the_next_send() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    assert_eq!(
        api.send("u1", "marketing", "otp", &[("code", "1")], &["email"]),
        "DELIVERED:email"
    );

    api.set_preference("u1", "email", "marketing", false);
    assert_eq!(
        api.send("u1", "marketing", "otp", &[("code", "1")], &["email"]),
        "SUPPRESSED"
    );

    api.set_preference("u1", "email", "marketing", true);
    assert_eq!(
        api.send("u1", "marketing", "otp", &[("code", "1")], &["email"]),
        "DELIVERED:email"
    );
    assert_eq!(gw.calls().len(), 2);
}

#[test]
fn should_deliver_to_each_users_own_address_in_send_order() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    api.register_user("u2", &[("email", "b@x.com")]);
    api.register_user("u3", &[("email", "c@x.com")]);

    api.send("u3", "security", "otp", &[("code", "3")], &["email"]);
    api.send("u1", "security", "otp", &[("code", "1")], &["email"]);
    api.send("u2", "security", "otp", &[("code", "2")], &["email"]);
    api.send("u3", "security", "otp", &[("code", "4")], &["email"]);

    assert_eq!(
        gw.calls(),
        vec![
            call("email", "c@x.com", "Your code is 3"),
            call("email", "a@x.com", "Your code is 1"),
            call("email", "b@x.com", "Your code is 2"),
            call("email", "c@x.com", "Your code is 4"),
        ]
    );
}

#[test]
fn should_return_invalid_when_only_some_required_parameters_are_given() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    api.define_template("two", "{a} and {b}");

    assert_eq!(
        api.send("u1", "security", "two", &[("a", "1")], &["email"]),
        "INVALID"
    );

    assert!(gw.calls().is_empty());
}

#[test]
fn should_accept_empty_string_as_a_parameter_value() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    api.define_template("t", "[{a}]");

    let result = api.send("u1", "security", "t", &[("a", "")], &["email"]);

    assert_eq!(result, "DELIVERED:email");
    assert_eq!(gw.bodies(), vec!["[]"]);
}

#[test]
fn should_recognise_placeholder_names_with_digits_and_underscores() {
    let gw = Arc::new(FakeGateway::default());
    let mut api = sut!(system(&gw));
    api.define_template("t", "{user_1}/{A_b2}/{9}");

    api.send(
        "u1",
        "security",
        "t",
        &[("user_1", "x"), ("A_b2", "y"), ("9", "z")],
        &["email"],
    );

    assert_eq!(gw.bodies(), vec!["x/y/z"]);
}
