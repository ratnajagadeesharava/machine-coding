use p65_observer::contract::*;
use p65_observer::PriceFeed;
use std::cell::RefCell;
use std::rc::Rc;

type Log = Rc<RefCell<Vec<String>>>;

/// An observer built from a closure, written in the test.
struct Fn1<F: Fn(&str, u64) -> Result<(), String>>(F);

impl<F: Fn(&str, u64) -> Result<(), String>> PriceObserver for Fn1<F> {
    fn on_price(&self, symbol: &str, price_cents: u64) -> Result<(), String> {
        (self.0)(symbol, price_cents)
    }
}

fn observer(f: impl Fn(&str, u64) -> Result<(), String> + 'static) -> Rc<dyn PriceObserver> {
    Rc::new(Fn1(f))
}

fn logger(log: &Log, tag: &'static str) -> Rc<dyn PriceObserver> {
    let log = log.clone();
    observer(move |s, p| {
        log.borrow_mut().push(format!("{tag} {s}={p}"));
        Ok(())
    })
}

fn new_log() -> Log {
    Rc::new(RefCell::new(Vec::new()))
}

#[test]
fn should_notify_observers_in_subscription_order() {
    let feed = PriceFeed::new();
    let log = new_log();
    feed.subscribe(logger(&log, "first"));
    feed.subscribe(logger(&log, "second"));
    feed.subscribe(logger(&log, "third"));

    let failures = feed.publish("ACME", 1250).unwrap();

    assert_eq!(failures, 0);
    assert_eq!(*log.borrow(), vec!["first ACME=1250", "second ACME=1250", "third ACME=1250"]);
}

#[test]
fn should_deliver_only_the_requested_symbol_to_filtered_subscribers() {
    let feed = PriceFeed::new();
    let log = new_log();
    feed.subscribe_to("ACME", logger(&log, "acme-only")).unwrap();
    feed.subscribe(logger(&log, "all"));

    feed.publish("ACME", 100).unwrap();
    feed.publish("INIT", 200).unwrap();

    assert_eq!(*log.borrow(), vec!["acme-only ACME=100", "all ACME=100", "all INIT=200"]);
}

#[test]
fn should_stop_notifying_after_unsubscribe_and_allow_unsubscribing_twice() {
    let feed = PriceFeed::new();
    let log = new_log();
    let a = feed.subscribe(logger(&log, "a"));
    feed.subscribe(logger(&log, "b"));
    feed.publish("X", 1).unwrap();

    assert!(feed.unsubscribe(a));
    assert!(!feed.unsubscribe(a));
    feed.publish("X", 2).unwrap();

    assert_eq!(feed.subscriber_count(), 1);
    assert_eq!(*log.borrow(), vec!["a X=1", "b X=1", "b X=2"]);
}

#[test]
fn should_not_notify_observers_that_subscribe_during_a_notification() {
    let feed = Rc::new(PriceFeed::new());
    let log = new_log();
    let (f, l) = (feed.clone(), log.clone());
    feed.subscribe(observer(move |_, p| {
        l.borrow_mut().push(format!("early {p}"));
        f.subscribe(logger(&l, "late"));
        Ok(())
    }));

    feed.publish("X", 1).unwrap();
    // the observer subscribed again during the second publish too, but that new one is not called yet
    let before = log.borrow().clone();
    feed.publish("X", 2).unwrap();

    assert_eq!(before, vec!["early 1"]);
    assert_eq!(*log.borrow(), vec!["early 1", "early 2", "late X=2"]);
}

#[test]
fn should_skip_an_observer_that_is_unsubscribed_before_its_turn_even_during_the_publish() {
    let feed = Rc::new(PriceFeed::new());
    let log = new_log();
    let second_id = Rc::new(RefCell::new(0u64));
    let (f, l, id) = (feed.clone(), log.clone(), second_id.clone());
    feed.subscribe(observer(move |_, p| {
        l.borrow_mut().push(format!("first {p}"));
        f.unsubscribe(*id.borrow());
        Ok(())
    }));
    *second_id.borrow_mut() = feed.subscribe(logger(&log, "second"));
    feed.subscribe(logger(&log, "third"));

    feed.publish("X", 1).unwrap();

    assert_eq!(*log.borrow(), vec!["first 1", "third X=1"]);
}

#[test]
fn should_keep_notifying_the_others_when_one_observer_fails() {
    let feed = PriceFeed::new();
    let log = new_log();
    feed.subscribe(logger(&log, "ok-1"));
    feed.subscribe(observer(|_, _| Err("boom".to_string())));
    feed.subscribe(observer(|_, _| Err("boom too".to_string())));
    feed.subscribe(logger(&log, "ok-2"));

    let failures = feed.publish("X", 5).unwrap();

    assert_eq!(failures, 2);
    assert_eq!(*log.borrow(), vec!["ok-1 X=5", "ok-2 X=5"]);
    assert_eq!(feed.subscriber_count(), 4);
}

#[test]
fn should_notify_twice_when_the_same_observer_subscribes_twice() {
    let feed = PriceFeed::new();
    let log = new_log();
    let same = logger(&log, "hit");
    feed.subscribe(same.clone());
    let second = feed.subscribe(same);

    feed.publish("X", 1).unwrap();
    feed.unsubscribe(second);
    feed.publish("X", 2).unwrap();

    assert_eq!(*log.borrow(), vec!["hit X=1", "hit X=1", "hit X=2"]);
}

#[test]
fn should_handle_publishing_with_no_subscribers_and_reject_bad_input() {
    let feed = PriceFeed::new();
    let log = new_log();

    assert_eq!(feed.publish("X", 1), Ok(0));
    assert!(matches!(feed.subscribe_to(" ", logger(&log, "x")), Err(FeedError::InvalidSymbol)));
    feed.subscribe(logger(&log, "called"));
    assert_eq!(feed.publish(" ", 1), Err(FeedError::InvalidSymbol));
    assert_eq!(feed.publish("X", 0), Err(FeedError::InvalidPrice));
    assert!(log.borrow().is_empty());
}
