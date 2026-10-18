use p57_adapter::contract::*;
use p57_adapter::{alert_all, LegacySmsAdapter};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

struct FakeLegacy {
    received: Rc<RefCell<Vec<String>>>,
    code: Rc<Cell<i32>>,
}

impl LegacySmsGateway for FakeLegacy {
    fn submit_xml(&self, xml: &str) -> i32 {
        self.received.borrow_mut().push(xml.to_string());
        if xml.contains("<to>BAD</to>") {
            1
        } else {
            self.code.get()
        }
    }
}

fn fake() -> (LegacySmsAdapter, Rc<RefCell<Vec<String>>>, Rc<Cell<i32>>) {
    let received = Rc::new(RefCell::new(Vec::new()));
    let code = Rc::new(Cell::new(0));
    let adapter = LegacySmsAdapter::new(Box::new(FakeLegacy { received: received.clone(), code: code.clone() }));
    (adapter, received, code)
}

#[test]
fn should_translate_a_send_into_the_legacy_xml_format() {
    let (adapter, received, _) = fake();

    assert_eq!(adapter.send("+15551234", "Hello there"), Ok(()));

    assert_eq!(*received.borrow(), vec!["<sms><to>+15551234</to><body>Hello there</body></sms>"]);
}

#[test]
fn should_escape_xml_special_characters() {
    let (adapter, received, _) = fake();

    adapter.send("1", "A&B <c> \"d\" 'e'").unwrap();

    assert_eq!(
        *received.borrow(),
        vec!["<sms><to>1</to><body>A&amp;B &lt;c&gt; &quot;d&quot; &apos;e&apos;</body></sms>"]
    );
}

#[test]
fn should_translate_legacy_return_codes_into_results() {
    let (adapter, _, code) = fake();

    code.set(0);
    assert_eq!(adapter.send("1", "m"), Ok(()));
    code.set(1);
    assert_eq!(adapter.send("1", "m"), Err("invalid number".to_string()));
    code.set(2);
    assert_eq!(adapter.send("1", "m"), Err("message too long".to_string()));
    code.set(3);
    assert_eq!(adapter.send("1", "m"), Err("unauthorized".to_string()));
    code.set(42);
    assert_eq!(adapter.send("1", "m"), Err("unknown error (code 42)".to_string()));
}

#[test]
fn should_reject_blank_input_without_bothering_the_legacy_system() {
    let (adapter, received, _) = fake();

    assert_eq!(adapter.send(" ", "m"), Err("invalid number".to_string()));
    assert_eq!(adapter.send("1", ""), Err("empty message".to_string()));
    assert_eq!(adapter.send("", ""), Err("invalid number".to_string()));

    assert!(received.borrow().is_empty());
}

#[test]
fn should_call_the_legacy_system_exactly_once_per_send() {
    let (adapter, received, _) = fake();

    adapter.send("1", "a").unwrap();
    adapter.send("2", "b").unwrap();

    assert_eq!(received.borrow().len(), 2);
}

#[test]
fn should_let_application_code_use_the_adapter_without_knowing_about_xml() {
    let (adapter, received, _) = fake();

    let failures = alert_all(&adapter, &["111", "BAD", "222"], "Server down");

    assert_eq!(failures, vec!["BAD: invalid number"]);
    assert_eq!(received.borrow().len(), 3);
}

#[test]
fn should_work_with_any_notifier_not_just_the_adapter() {
    struct Flaky;
    impl Notifier for Flaky {
        fn send(&self, phone: &str, _message: &str) -> Result<(), String> {
            if phone.starts_with('0') {
                Err("blocked".to_string())
            } else {
                Ok(())
            }
        }
    }

    assert_eq!(alert_all(&Flaky, &["0100", "200", "0300"], "hi"), vec!["0100: blocked", "0300: blocked"]);
    assert!(alert_all(&Flaky, &[], "hi").is_empty());
}
