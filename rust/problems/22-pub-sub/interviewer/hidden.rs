#![allow(unused_imports, clippy::needless_range_loop)]
use p22_pub_sub::{contract::*, create};
use playground_support::sut;
use std::sync::Arc;

fn broker() -> Option<Arc<dyn PubSubApi>> {
    let api = create()?;
    api.create_topic("t");
    Some(api)
}

#[test]
fn resubscribing_should_not_bring_back_messages_from_before_unsubscribe() {
    let api = sut!(broker());
    api.subscribe("t", "s");
    api.publish("t", "m1");
    api.unsubscribe("t", "s");
    api.publish("t", "m2");

    assert!(api.subscribe("t", "s"));
    api.publish("t", "m3");

    assert_eq!(api.poll("s", "t", 10), vec!["m3"]);
}

#[test]
fn messages_published_without_subscribers_should_not_be_kept_for_later_subscribers() {
    let api = sut!(broker());
    assert_eq!(api.publish("t", "lost"), 0);

    api.subscribe("t", "s");

    assert!(api.poll("s", "t", 10).is_empty());
}

#[test]
fn partial_poll_should_leave_remainder_ahead_of_new_messages() {
    let api = sut!(broker());
    api.subscribe("t", "s");
    api.publish("t", "m1");
    api.publish("t", "m2");
    api.publish("t", "m3");

    assert_eq!(api.poll("s", "t", 1), vec!["m1"]);
    api.publish("t", "m4");

    assert_eq!(api.poll("s", "t", 10), vec!["m2", "m3", "m4"]);
}

#[test]
fn same_text_published_twice_should_be_delivered_twice() {
    let api = sut!(broker());
    api.subscribe("t", "s");

    api.publish("t", "ping");
    api.publish("t", "ping");

    assert_eq!(api.poll("s", "t", 10), vec!["ping", "ping"]);
}

#[test]
fn unsubscribed_subscriber_should_not_count_in_fan_out_and_others_keep_their_messages() {
    let api = sut!(broker());
    api.subscribe("t", "a");
    api.subscribe("t", "b");
    api.publish("t", "m1");
    api.unsubscribe("t", "a");

    assert_eq!(api.publish("t", "m2"), 1);

    assert_eq!(api.poll("b", "t", 10), vec!["m1", "m2"]);
    assert!(api.poll("a", "t", 10).is_empty());
}
