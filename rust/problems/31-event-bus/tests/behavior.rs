#![allow(unused_imports, clippy::needless_range_loop)]
use p31_event_bus::{contract::EventBusApi, create};
use playground_support::{eventually, run_concurrently, sut};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::SeqCst};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

const LONG: u64 = 10_000;

fn bus() -> Option<Arc<dyn EventBusApi>> {
    create()
}

type Handler = Arc<dyn Fn(&str) + Send + Sync>;
type Sink = Arc<Mutex<Vec<String>>>;

fn sink() -> Sink {
    Arc::new(Mutex::new(Vec::new()))
}

fn into(sink: &Sink) -> Handler {
    let sink = sink.clone();
    Arc::new(move |e: &str| sink.lock().unwrap().push(e.to_string()))
}

fn items(sink: &Sink) -> Vec<String> {
    sink.lock().unwrap().clone()
}

fn wait_for(flag: &AtomicBool) {
    assert!(
        eventually(LONG, || flag.load(SeqCst)),
        "gate was never opened"
    );
}

#[test]
fn should_deliver_each_event_to_every_subscriber_of_the_topic_only() {
    let bus = sut!(bus());
    let (s1, s2, other) = (sink(), sink(), sink());
    bus.subscribe("orders", into(&s1));
    bus.subscribe("orders", into(&s2));
    bus.subscribe("billing", into(&other));

    bus.publish("orders", "o-1");

    assert!(bus.await_idle(LONG));
    assert_eq!(items(&s1), vec!["o-1"]);
    assert_eq!(items(&s2), vec!["o-1"]);
    assert!(items(&other).is_empty());
}

#[test]
fn should_treat_the_same_handler_subscribed_twice_as_two_subscriptions() {
    let bus = sut!(bus());
    let s = sink();
    let handler = into(&s);

    let first = bus.subscribe("t", handler.clone());
    let second = bus.subscribe("t", handler);
    bus.publish("t", "e");

    assert!(!first.is_empty());
    assert_ne!(first, second);
    assert!(bus.await_idle(LONG));
    assert_eq!(items(&s), vec!["e", "e"]);
}

#[test]
fn should_be_idle_when_nothing_was_published_and_discard_events_without_subscribers() {
    let bus = sut!(bus());
    assert!(bus.await_idle(0));

    bus.publish("nobody", "lost");
    assert!(bus.await_idle(LONG));

    let s = sink();
    bus.subscribe("nobody", into(&s));
    assert!(bus.await_idle(LONG));
    assert!(items(&s).is_empty());
}

#[test]
fn should_not_deliver_events_published_before_the_subscription() {
    let bus = sut!(bus());
    bus.publish("t", "e1");
    assert!(bus.await_idle(LONG));

    let s = sink();
    bus.subscribe("t", into(&s));
    bus.publish("t", "e2");

    assert!(bus.await_idle(LONG));
    assert_eq!(items(&s), vec!["e2"]);
}

#[test]
fn should_keep_publish_order_per_subscriber_for_one_publisher() {
    let bus = sut!(bus());
    let sinks = [sink(), sink(), sink()];
    for s in &sinks {
        bus.subscribe("t", into(s));
    }
    let expected: Vec<String> = (0..500).map(|i| format!("e{i}")).collect();

    for e in &expected {
        bus.publish("t", e);
    }

    assert!(bus.await_idle(LONG));
    for s in &sinks {
        assert_eq!(items(s), expected);
    }
}

#[test]
fn should_keep_each_publishers_order_when_several_threads_publish() {
    let bus = sut!(bus());
    let sinks = [sink(), sink()];
    for s in &sinks {
        bus.subscribe("t", into(s));
    }

    run_concurrently(4, |t| {
        for i in 0..100 {
            bus.publish("t", &format!("{t}:{i}"));
        }
    });

    assert!(bus.await_idle(LONG));
    for s in &sinks {
        let all = items(s);
        assert_eq!(all.len(), 400);
        let unique: std::collections::HashSet<_> = all.iter().collect();
        assert_eq!(unique.len(), 400);
        for t in 0..4 {
            let prefix = format!("{t}:");
            let seen: Vec<usize> = all
                .iter()
                .filter_map(|e| e.strip_prefix(&prefix))
                .map(|n| n.parse().unwrap())
                .collect();
            assert_eq!(seen, (0..100).collect::<Vec<_>>());
        }
    }
}

#[test]
fn should_never_run_one_subscriptions_handler_concurrently_with_itself() {
    let bus = sut!(bus());
    let active = Arc::new(AtomicUsize::new(0));
    let overlapped = Arc::new(AtomicBool::new(false));
    let handled = Arc::new(AtomicUsize::new(0));
    let (a, o, h) = (active.clone(), overlapped.clone(), handled.clone());
    bus.subscribe(
        "t",
        Arc::new(move |_| {
            if a.fetch_add(1, SeqCst) > 0 {
                o.store(true, SeqCst);
            }
            std::thread::yield_now();
            a.fetch_sub(1, SeqCst);
            h.fetch_add(1, SeqCst);
        }),
    );

    run_concurrently(4, |_| {
        for _ in 0..50 {
            bus.publish("t", "e");
        }
    });

    assert!(bus.await_idle(LONG));
    assert_eq!(handled.load(SeqCst), 200);
    assert!(!overlapped.load(SeqCst));
}

#[test]
fn should_return_from_publish_and_report_busy_while_a_handler_is_blocked() {
    let bus = sut!(bus());
    let gate = Arc::new(AtomicBool::new(false));
    let s = sink();
    let (g, out) = (gate.clone(), s.clone());
    bus.subscribe(
        "t",
        Arc::new(move |e: &str| {
            wait_for(&g);
            out.lock().unwrap().push(e.to_string());
        }),
    );

    let (tx, rx) = mpsc::channel();
    let publisher_bus = bus.clone();
    std::thread::spawn(move || {
        for n in 0..50 {
            publisher_bus.publish("t", &format!("e{n}"));
        }
        let _ = tx.send(());
    });
    assert!(
        rx.recv_timeout(Duration::from_millis(LONG)).is_ok(),
        "publish must not wait for handlers"
    );

    assert!(!bus.await_idle(100));
    gate.store(true, SeqCst);
    assert!(bus.await_idle(LONG));
    let got = items(&s);
    assert_eq!(got.len(), 50);
    assert_eq!(got[0], "e0");
    assert_eq!(got[49], "e49");
}

#[test]
fn should_not_let_a_slow_handler_block_other_subscribers_or_topics() {
    let bus = sut!(bus());
    let gate = Arc::new(AtomicBool::new(false));
    let (slow, same_topic, other_topic) = (sink(), sink(), sink());
    let (g, out) = (gate.clone(), slow.clone());
    bus.subscribe(
        "a",
        Arc::new(move |e: &str| {
            wait_for(&g);
            out.lock().unwrap().push(e.to_string());
        }),
    );
    bus.subscribe("a", into(&same_topic));
    bus.subscribe("b", into(&other_topic));

    for i in 0..20 {
        bus.publish("a", &format!("a{i}"));
        bus.publish("b", &format!("b{i}"));
    }

    assert!(
        eventually(LONG, || items(&same_topic).len() == 20
            && items(&other_topic).len() == 20),
        "other subscribers keep receiving while one handler is blocked"
    );
    assert!(items(&slow).is_empty());
    gate.store(true, SeqCst);
    assert!(bus.await_idle(LONG));
    assert_eq!(items(&slow).len(), 20);
}

#[test]
fn should_isolate_handler_failures() {
    let bus = sut!(bus());
    let failures = Arc::new(AtomicUsize::new(0));
    let good = sink();
    let f = failures.clone();
    bus.subscribe(
        "t",
        Arc::new(move |e: &str| {
            f.fetch_add(1, SeqCst);
            panic!("handler blew up on {e}");
        }),
    );
    bus.subscribe("t", into(&good));

    bus.publish("t", "e1");
    bus.publish("t", "e2");
    bus.publish("t", "e3");

    assert!(bus.await_idle(LONG));
    assert_eq!(items(&good), vec!["e1", "e2", "e3"]);
    assert_eq!(failures.load(SeqCst), 3);
}

#[test]
fn should_let_handlers_publish_without_deadlock() {
    let bus = sut!(bus());
    let chained = sink();
    let b = bus.clone();
    bus.subscribe(
        "a",
        Arc::new(move |e: &str| b.publish("b", &format!("{e}!"))),
    );
    let b = bus.clone();
    bus.subscribe(
        "b",
        Arc::new(move |e: &str| b.publish("c", &format!("{e}!"))),
    );
    bus.subscribe("c", into(&chained));
    let countdown = sink();
    let (b, out) = (bus.clone(), countdown.clone());
    bus.subscribe(
        "count",
        Arc::new(move |e: &str| {
            let n: u32 = e.parse().unwrap();
            out.lock().unwrap().push(e.to_string());
            if n > 0 {
                b.publish("count", &(n - 1).to_string());
            }
        }),
    );

    bus.publish("a", "x");
    bus.publish("count", "200");

    assert!(bus.await_idle(LONG));
    assert_eq!(items(&chained), vec!["x!!"]);
    let expected: Vec<String> = (0..=200).rev().map(|n| n.to_string()).collect();
    assert_eq!(items(&countdown), expected);
}

#[test]
fn should_stop_deliveries_after_unsubscribe() {
    let bus = sut!(bus());
    let s = sink();
    let id = bus.subscribe("t", into(&s));
    bus.publish("t", "e1");
    assert!(bus.await_idle(LONG));

    assert!(bus.unsubscribe(&id));
    bus.publish("t", "e2");

    assert!(bus.await_idle(LONG));
    assert_eq!(items(&s), vec!["e1"]);
    assert!(!bus.unsubscribe(&id));
    assert!(!bus.unsubscribe("no-such-id"));
}

#[test]
fn should_discard_pending_events_of_an_unsubscribed_subscription() {
    let bus = sut!(bus());
    let started = Arc::new(AtomicBool::new(false));
    let gate = Arc::new(AtomicBool::new(false));
    let s = sink();
    let (st, g, out) = (started.clone(), gate.clone(), s.clone());
    let id = bus.subscribe(
        "t",
        Arc::new(move |e: &str| {
            st.store(true, SeqCst);
            wait_for(&g);
            out.lock().unwrap().push(e.to_string());
        }),
    );
    bus.publish("t", "e1");
    wait_for(&started);
    bus.publish("t", "e2");
    bus.publish("t", "e3");

    assert!(bus.unsubscribe(&id));
    gate.store(true, SeqCst);

    assert!(bus.await_idle(LONG));
    assert_eq!(items(&s), vec!["e1"]);
}

#[test]
fn should_allow_a_handler_to_unsubscribe_itself() {
    let bus = sut!(bus());
    let id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let calls = Arc::new(AtomicUsize::new(0));
    let (b, i, c) = (bus.clone(), id.clone(), calls.clone());
    let sub = bus.subscribe(
        "t",
        Arc::new(move |_| {
            c.fetch_add(1, SeqCst);
            let own = i.lock().unwrap().clone().unwrap();
            b.unsubscribe(&own);
        }),
    );
    *id.lock().unwrap() = Some(sub);

    bus.publish("t", "e1");
    bus.publish("t", "e2");
    bus.publish("t", "e3");

    assert!(bus.await_idle(LONG));
    assert_eq!(calls.load(SeqCst), 1);
}

#[test]
fn should_discard_events_published_after_close_but_deliver_earlier_ones() {
    let bus = sut!(bus());
    let s = sink();
    bus.subscribe("t", into(&s));
    bus.publish("t", "before");

    bus.close();
    bus.publish("t", "after");

    assert!(bus.await_idle(LONG));
    assert_eq!(items(&s), vec!["before"]);
}

#[test]
fn should_stay_consistent_while_subscriptions_come_and_go_during_publishing() {
    let bus = sut!(bus());
    let stable = sink();
    bus.subscribe("t", into(&stable));

    run_concurrently(8, |t| {
        if t < 4 {
            for i in 0..200 {
                bus.publish("t", &format!("{t}:{i}"));
            }
        } else {
            for _ in 0..100 {
                let id = bus.subscribe("t", Arc::new(|_| {}));
                bus.unsubscribe(&id);
            }
        }
    });

    assert!(bus.await_idle(LONG));
    let all = items(&stable);
    assert_eq!(all.len(), 800);
    let unique: std::collections::HashSet<_> = all.iter().collect();
    assert_eq!(unique.len(), 800);
}
