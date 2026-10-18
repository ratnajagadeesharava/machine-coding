#![allow(unused_imports, clippy::needless_range_loop)]
use p31_event_bus::{contract::EventBusApi, create};
use playground_support::{eventually, sut};
use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
use std::sync::{Arc, Mutex};

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
fn a_subscription_created_while_an_event_is_still_pending_must_not_receive_it() {
    let bus = sut!(bus());
    let started = Arc::new(AtomicBool::new(false));
    let gate = Arc::new(AtomicBool::new(false));
    let (a, b) = (sink(), sink());
    let (st, g, out) = (started.clone(), gate.clone(), a.clone());
    bus.subscribe(
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

    bus.subscribe("t", into(&b));
    bus.publish("t", "e3");
    gate.store(true, SeqCst);

    assert!(bus.await_idle(LONG));
    assert_eq!(items(&a), vec!["e1", "e2", "e3"]);
    assert_eq!(items(&b), vec!["e3"]);
}

#[test]
fn an_already_running_handler_finishes_after_unsubscribe() {
    let bus = sut!(bus());
    let started = Arc::new(AtomicBool::new(false));
    let gate = Arc::new(AtomicBool::new(false));
    let finished = sink();
    let (st, g, out) = (started.clone(), gate.clone(), finished.clone());
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

    assert!(bus.unsubscribe(&id));
    assert!(!bus.await_idle(100)); // the running invocation still counts as work
    gate.store(true, SeqCst);

    assert!(bus.await_idle(LONG));
    assert_eq!(items(&finished), vec!["e1"]);
}

#[test]
fn a_handler_that_publishes_and_then_fails_still_gets_its_published_event_delivered() {
    let bus = sut!(bus());
    let out = sink();
    let b = bus.clone();
    bus.subscribe(
        "in",
        Arc::new(move |e: &str| {
            b.publish("out", &format!("{e}-out"));
            panic!("after publishing");
        }),
    );
    bus.subscribe("out", into(&out));

    bus.publish("in", "x");

    assert!(bus.await_idle(LONG));
    assert_eq!(items(&out), vec!["x-out"]);
}

#[test]
fn close_is_idempotent_and_later_subscriptions_never_receive() {
    let bus = sut!(bus());
    let late = sink();

    bus.close();
    bus.close();
    let id = bus.subscribe("t", into(&late));
    bus.publish("t", "e");

    assert!(!id.is_empty());
    assert!(bus.await_idle(LONG));
    assert!(items(&late).is_empty());
}

#[test]
fn topics_are_case_sensitive_and_empty_events_are_events() {
    let bus = sut!(bus());
    let (lower, upper) = (sink(), sink());
    bus.subscribe("orders", into(&lower));
    bus.subscribe("Orders", into(&upper));

    bus.publish("orders", "");
    bus.publish("Orders", "  spaced  ");

    assert!(bus.await_idle(LONG));
    assert_eq!(items(&lower), vec![""]);
    assert_eq!(items(&upper), vec!["  spaced  "]);
}
