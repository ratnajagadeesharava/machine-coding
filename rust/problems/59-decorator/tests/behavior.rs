use p59_decorator::contract::*;
use p59_decorator::{CountingSink, RedactingSink, RetryingSink, TimestampingSink};
use playground_support::FakeClock;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// The innermost component, written in the test: records what finally arrives. Can fail the first N sends.
struct Recorder {
    received: Rc<RefCell<Vec<String>>>,
    attempts: Rc<Cell<u32>>,
    failures_left: Cell<u32>,
}

impl MessageSink for Recorder {
    fn send(&self, message: &str) -> Result<(), SinkError> {
        self.attempts.set(self.attempts.get() + 1);
        if self.failures_left.get() > 0 {
            self.failures_left.set(self.failures_left.get() - 1);
            return Err(SinkError(format!("network down #{}", self.attempts.get())));
        }
        self.received.borrow_mut().push(message.to_string());
        Ok(())
    }
}

struct Fixture {
    received: Rc<RefCell<Vec<String>>>,
    attempts: Rc<Cell<u32>>,
}

fn recorder(failures: u32) -> (Box<dyn MessageSink>, Fixture) {
    let received = Rc::new(RefCell::new(Vec::new()));
    let attempts = Rc::new(Cell::new(0));
    let rec = Recorder { received: received.clone(), attempts: attempts.clone(), failures_left: Cell::new(failures) };
    (Box::new(rec), Fixture { received, attempts })
}

fn clock() -> Arc<dyn playground_support::Clock> {
    FakeClock::new(1_700_000_000_000).as_clock()
}

use std::sync::Arc;

#[test]
fn should_prefix_the_clock_time() {
    let (rec, f) = recorder(0);

    TimestampingSink::new(rec, clock()).send("hello").unwrap();

    assert_eq!(*f.received.borrow(), vec!["[t=1700000000000] hello"]);
}

#[test]
fn should_redact_long_digit_runs_only() {
    let (rec, f) = recorder(0);

    RedactingSink::new(rec).send("card 4111111111111111 exp 1226 room 101 floor 7").unwrap();

    assert_eq!(*f.received.borrow(), vec!["card **** exp **** room 101 floor 7"]);
}

#[test]
fn should_retry_until_the_next_sink_succeeds() {
    let (rec, f) = recorder(2);

    RetryingSink::new(rec, 3).send("important").unwrap();

    assert_eq!(f.attempts.get(), 3);
    assert_eq!(*f.received.borrow(), vec!["important"]);
}

#[test]
fn should_give_up_after_the_maximum_number_of_attempts_and_return_the_last_error() {
    let (rec, f) = recorder(5);
    let sink = RetryingSink::new(rec, 2);

    assert_eq!(sink.send("x"), Err(SinkError("network down #2".to_string())));

    assert_eq!(f.attempts.get(), 2);
    assert!(f.received.borrow().is_empty());
}

#[test]
fn should_count_only_delivered_messages() {
    let (rec, f) = recorder(1);
    let counting = CountingSink::new(rec);

    assert!(counting.send("lost").is_err());
    counting.send("a").unwrap();
    counting.send("b").unwrap();

    assert_eq!(counting.count(), 2);
    assert_eq!(*f.received.borrow(), vec!["a", "b"]);
}

#[test]
fn should_let_the_order_of_wrapping_change_the_result() {
    let (rec_a, redact_first) = recorder(0);
    let (rec_b, timestamp_first) = recorder(0);

    RedactingSink::new(Box::new(TimestampingSink::new(rec_a, clock()))).send("pin 123456").unwrap();
    TimestampingSink::new(Box::new(RedactingSink::new(rec_b)), clock()).send("pin 123456").unwrap();

    assert_eq!(*redact_first.received.borrow(), vec!["[t=1700000000000] pin ****"]);
    assert_eq!(*timestamp_first.received.borrow(), vec!["[t=****] pin ****"]);
}

#[test]
fn should_stack_many_decorators_around_one_component() {
    let (rec, f) = recorder(1);
    let counting = CountingSink::new(Box::new(RetryingSink::new(
        Box::new(RedactingSink::new(Box::new(TimestampingSink::new(rec, clock())))),
        3,
    )));

    counting.send("otp 998877").unwrap();
    counting.send("ok").unwrap();

    assert_eq!(counting.count(), 2);
    assert_eq!(f.attempts.get(), 3);
    assert_eq!(*f.received.borrow(), vec!["[t=1700000000000] otp ****", "[t=1700000000000] ok"]);
}

#[test]
fn should_be_usable_wherever_a_plain_sink_is_expected() {
    let (rec, f) = recorder(0);
    let sinks: Vec<Box<dyn MessageSink>> = vec![
        Box::new(RedactingSink::new(rec)),
        Box::new(CountingSink::new(Box::new(RedactingSink::new(recorder(0).0)))),
    ];

    for sink in &sinks {
        sink.send("code 12345").unwrap();
    }

    assert_eq!(*f.received.borrow(), vec!["code ****"]);
}
