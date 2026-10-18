#![allow(unused_imports, clippy::needless_range_loop)]
use p22_pub_sub::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

fn drain(api: &dyn PubSubApi, subscriber: &str, topic: &str) -> Vec<String> {
    let mut all = Vec::new();
    loop {
        let batch = api.poll(subscriber, topic, 100);
        if batch.is_empty() {
            return all;
        }
        all.extend(batch);
    }
}

/// Messages look like "publisher:sequence"; each publisher's sequence numbers must be strictly increasing.
fn assert_per_publisher_order(messages: &[String]) {
    let mut last: HashMap<&str, u32> = HashMap::new();
    for m in messages {
        let (publisher, seq) = m.split_once(':').unwrap();
        let seq: u32 = seq.parse().unwrap();
        if let Some(previous) = last.insert(publisher, seq) {
            assert!(previous < seq, "order of {m} after {previous}");
        }
    }
}

fn unique(messages: &[String]) -> usize {
    messages.iter().collect::<HashSet<_>>().len()
}

#[test]
fn should_create_topics_only_once() {
    let api = sut!(create());

    assert!(api.create_topic("news"));
    assert!(!api.create_topic("news"));
    assert!(!api.create_topic(""));
    assert!(!api.create_topic("   "));
    assert!(api.create_topic("sports"));
}

#[test]
fn should_fan_out_to_every_subscriber_and_report_the_count() {
    let api = sut!(create());
    api.create_topic("news");
    api.subscribe("news", "ann");
    assert_eq!(api.publish("news", "m1"), 1);
    api.subscribe("news", "bob");

    assert_eq!(api.publish("news", "m2"), 2);

    assert_eq!(api.poll("ann", "news", 10), vec!["m1", "m2"]);
    assert_eq!(api.poll("bob", "news", 10), vec!["m2"]);
}

#[test]
fn should_report_minus_one_for_unknown_topic_and_zero_without_subscribers() {
    let api = sut!(create());
    api.create_topic("empty");

    assert_eq!(api.publish("nope", "x"), -1);
    assert_eq!(api.publish("empty", "x"), 0);
}

#[test]
fn should_not_deliver_messages_published_before_subscribing() {
    let api = sut!(create());
    api.create_topic("t");
    api.publish("t", "old");

    api.subscribe("t", "late");
    api.publish("t", "new");

    assert_eq!(api.poll("late", "t", 10), vec!["new"]);
}

#[test]
fn should_return_oldest_first_and_consume_what_was_polled() {
    let api = sut!(create());
    api.create_topic("t");
    api.subscribe("t", "s");
    for i in 1..=5 {
        api.publish("t", &format!("m{i}"));
    }

    assert_eq!(api.poll("s", "t", 2), vec!["m1", "m2"]);
    assert_eq!(api.poll("s", "t", 2), vec!["m3", "m4"]);
    assert_eq!(api.poll("s", "t", 2), vec!["m5"]);
    assert!(api.poll("s", "t", 2).is_empty());
}

#[test]
fn should_return_nothing_and_consume_nothing_for_non_positive_max() {
    let api = sut!(create());
    api.create_topic("t");
    api.subscribe("t", "s");
    api.publish("t", "m1");

    assert!(api.poll("s", "t", 0).is_empty());
    assert!(api.poll("s", "t", -3).is_empty());

    assert_eq!(api.poll("s", "t", 5), vec!["m1"]);
}

#[test]
fn should_return_nothing_for_unknown_topic_or_non_subscriber() {
    let api = sut!(create());
    api.create_topic("t");
    api.subscribe("t", "s");
    api.publish("t", "m1");

    assert!(api.poll("s", "nope", 5).is_empty());
    assert!(api.poll("stranger", "t", 5).is_empty());
    assert_eq!(api.poll("s", "t", 5), vec!["m1"]);
}

#[test]
fn should_refuse_invalid_subscriptions() {
    let api = sut!(create());
    api.create_topic("t");

    assert!(!api.subscribe("nope", "s"));
    assert!(!api.subscribe("t", ""));
    assert!(api.subscribe("t", "s"));
    assert!(!api.subscribe("t", "s"));
    assert_eq!(api.publish("t", "m"), 1);
}

#[test]
fn should_stop_delivering_and_discard_pending_messages_on_unsubscribe() {
    let api = sut!(create());
    api.create_topic("t");
    api.subscribe("t", "s");
    api.publish("t", "m1");

    assert!(api.unsubscribe("t", "s"));
    assert!(!api.unsubscribe("t", "s"));
    assert!(!api.unsubscribe("nope", "s"));

    assert_eq!(api.publish("t", "m2"), 0);
    assert!(api.poll("s", "t", 10).is_empty());
}

#[test]
fn should_keep_subscribers_queues_independent() {
    let api = sut!(create());
    api.create_topic("t");
    api.subscribe("t", "a");
    api.subscribe("t", "b");
    api.publish("t", "m1");
    api.publish("t", "m2");

    assert_eq!(api.poll("a", "t", 10), vec!["m1", "m2"]);

    assert_eq!(api.poll("b", "t", 1), vec!["m1"]);
    assert_eq!(api.poll("b", "t", 1), vec!["m2"]);
}

#[test]
fn should_keep_topics_separate_for_the_same_subscriber() {
    let api = sut!(create());
    api.create_topic("x");
    api.create_topic("y");
    api.subscribe("x", "s");
    api.subscribe("y", "s");

    api.publish("x", "fromX");
    api.publish("y", "fromY");

    assert_eq!(api.poll("s", "y", 10), vec!["fromY"]);
    assert_eq!(api.poll("s", "x", 10), vec!["fromX"]);
}

#[test]
fn should_return_a_copy_that_later_polls_do_not_affect() {
    let api = sut!(create());
    api.create_topic("t");
    api.subscribe("t", "s");
    api.publish("t", "m1");

    let first = api.poll("s", "t", 10);
    api.publish("t", "m2");
    api.poll("s", "t", 10);

    assert_eq!(first, vec!["m1"]);
}

#[test]
fn should_deliver_everything_in_per_publisher_order_to_a_stable_subscriber() {
    let api = sut!(create());
    api.create_topic("t");
    api.subscribe("t", "s");
    let publishers = 8;
    let per_publisher = 500;

    run_concurrently(publishers, |p| {
        for i in 0..per_publisher {
            api.publish("t", &format!("{p}:{i}"));
        }
    });

    let received = drain(api.as_ref(), "s", "t");
    assert_eq!(received.len(), publishers * per_publisher);
    assert_eq!(unique(&received), publishers * per_publisher);
    assert_per_publisher_order(&received);
}

#[test]
fn should_give_every_subscriber_the_same_messages_under_concurrent_publishing() {
    let api = sut!(create());
    api.create_topic("t");
    for s in 0..5 {
        api.subscribe("t", &format!("s{s}"));
    }

    run_concurrently(6, |p| {
        for i in 0..200 {
            assert_eq!(api.publish("t", &format!("{p}:{i}")), 5);
        }
    });

    for s in 0..5 {
        let received = drain(api.as_ref(), &format!("s{s}"), "t");
        assert_eq!(received.len(), 1200);
        assert_per_publisher_order(&received);
    }
}

#[test]
fn should_hand_out_each_message_once_when_several_threads_poll_the_same_subscriber() {
    let api = sut!(create());
    api.create_topic("t");
    api.subscribe("t", "shared");
    let total = 4 * 500;
    let consumed = AtomicUsize::new(0);

    let per_thread = collect_concurrently(8, |i| {
        let mut mine: Vec<String> = Vec::new();
        if i < 4 {
            for n in 0..500 {
                api.publish("t", &format!("{i}:{n}"));
            }
            return mine;
        }
        let deadline = Instant::now() + Duration::from_secs(20);
        while consumed.load(Ordering::SeqCst) < total && Instant::now() < deadline {
            let batch = api.poll("shared", "t", 7);
            consumed.fetch_add(batch.len(), Ordering::SeqCst);
            mine.extend(batch);
        }
        mine
    });

    let mut all: Vec<String> = Vec::new();
    for mine in &per_thread {
        assert_per_publisher_order(mine);
        all.extend(mine.iter().cloned());
    }
    assert_eq!(all.len(), total);
    assert_eq!(unique(&all), total);
}

#[test]
fn should_stay_consistent_while_subscribers_come_and_go_during_publishing() {
    let api = sut!(create());
    api.create_topic("t");
    api.subscribe("t", "stable");
    let counts: Mutex<Vec<i32>> = Mutex::new(Vec::new());

    let received = collect_concurrently(8, |i| {
        let mut mine: Vec<String> = Vec::new();
        if i < 4 {
            for n in 0..300 {
                let c = api.publish("t", &format!("{i}:{n}"));
                counts.lock().unwrap().push(c);
            }
            return mine;
        }
        let me = format!("churn{i}");
        for _ in 0..100 {
            api.subscribe("t", &me);
            mine.extend(api.poll(&me, "t", 50));
            api.unsubscribe("t", &me);
        }
        mine
    });

    let counts = counts.into_inner().unwrap();
    assert_eq!(counts.len(), 1200);
    assert!(counts.iter().all(|c| (1..=5).contains(c)));
    let stable = drain(api.as_ref(), "stable", "t");
    assert_eq!(stable.len(), 1200);
    assert_eq!(unique(&stable), 1200);
    assert_per_publisher_order(&stable);
    for mine in &received {
        assert_eq!(unique(mine), mine.len());
        assert_per_publisher_order(mine);
    }
}

#[test]
fn should_allow_only_one_winner_when_creating_or_subscribing_concurrently() {
    let api = sut!(create());

    let created = collect_concurrently(16, |_| api.create_topic("same"));
    assert_eq!(created.iter().filter(|b| **b).count(), 1);

    let subscribed = collect_concurrently(16, |_| api.subscribe("same", "dup"));
    assert_eq!(subscribed.iter().filter(|b| **b).count(), 1);
    assert_eq!(api.publish("same", "m"), 1);
}

#[test]
fn should_handle_many_messages_and_subscribers_quickly() {
    let api = sut!(create());
    api.create_topic("t");
    for s in 0..10 {
        api.subscribe("t", &format!("s{s}"));
    }
    let start = Instant::now();

    for i in 0..50_000 {
        api.publish("t", &format!("m{i}"));
    }
    for s in 0..10 {
        let mut count = 0;
        loop {
            let batch = api.poll(&format!("s{s}"), "t", 500);
            if batch.is_empty() {
                break;
            }
            count += batch.len();
        }
        assert_eq!(count, 50_000);
    }

    assert!(start.elapsed() < Duration::from_secs(10));
}
