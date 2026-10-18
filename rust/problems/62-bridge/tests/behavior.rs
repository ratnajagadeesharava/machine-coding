use p62_bridge::contract::*;
use p62_bridge::{Alert, DigestAlert, EmailChannel, InfoAlert, SlackChannel, SmsChannel, UrgentAlert};
use std::cell::RefCell;
use std::rc::Rc;

type Wire = Rc<RefCell<Vec<String>>>;

fn wire() -> (Wire, impl Fn() -> Box<dyn Fn(&str)>) {
    let lines: Wire = Rc::new(RefCell::new(Vec::new()));
    let l = lines.clone();
    let make = move || {
        let l = l.clone();
        Box::new(move |s: &str| l.borrow_mut().push(s.to_string())) as Box<dyn Fn(&str)>
    };
    (lines, make)
}

/// A channel written in the test: records exactly what the alert asked it to deliver.
struct Recording(Wire);

impl Channel for Recording {
    fn deliver(&self, recipient: &str, title: &str, body: &str) {
        self.0.borrow_mut().push(format!("{recipient}|{title}|{body}"));
    }
}

fn recording() -> (Wire, Box<dyn Channel>) {
    let w: Wire = Rc::new(RefCell::new(Vec::new()));
    (w.clone(), Box::new(Recording(w)))
}

#[test]
fn should_shape_info_alerts_independently_of_the_channel() {
    let (delivered, channel) = recording();

    InfoAlert::new(channel).send("ann", "Build finished");

    assert_eq!(*delivered.borrow(), vec!["ann|Info|Build finished"]);
}

#[test]
fn should_shape_urgent_alerts_independently_of_the_channel() {
    let (delivered, channel) = recording();

    UrgentAlert::new(channel).send("ann", "disk is full");

    assert_eq!(*delivered.borrow(), vec!["ann|URGENT|DISK IS FULL"]);
}

#[test]
fn should_collect_digest_messages_per_recipient_and_flush_them_together() {
    let (delivered, channel) = recording();
    let mut digest = DigestAlert::new(channel);

    digest.send("ann", "one");
    digest.send("bob", "other");
    digest.send("ann", "two");
    digest.send("ann", "three");

    assert!(delivered.borrow().is_empty());
    assert_eq!(digest.pending("ann"), 3);
    assert!(digest.flush("ann"));
    assert_eq!(*delivered.borrow(), vec!["ann|Digest (3)|one; two; three"]);
    assert_eq!(digest.pending("ann"), 0);
    assert!(!digest.flush("ann"));
    assert_eq!(digest.pending("bob"), 1);
    assert_eq!(delivered.borrow().len(), 1);
}

#[test]
fn should_format_each_channel_independently_of_the_alert() {
    let (lines, make) = wire();

    EmailChannel::new(make()).deliver("ann@x.io", "Report", "All good");
    SlackChannel::new(make()).deliver("ann", "Report", "All good");
    SmsChannel::new(make()).deliver("+1555", "Report", "All good");

    assert_eq!(
        *lines.borrow(),
        vec!["EMAIL to:ann@x.io subject:Report | All good", "SLACK @ann *Report* All good", "SMS +1555: Report: All good"]
    );
}

#[test]
fn should_cut_long_sms_texts_to_40_characters() {
    let (lines, make) = wire();
    let sms = SmsChannel::new(make());

    sms.deliver("1", "T", &"x".repeat(38));
    sms.deliver("1", "T", &"x".repeat(37));

    assert_eq!(lines.borrow()[0], format!("SMS 1: T: {}...", "x".repeat(34)));
    assert_eq!(lines.borrow()[1], format!("SMS 1: T: {}", "x".repeat(37)));
    assert_eq!(lines.borrow()[0]["SMS 1: ".len()..].len(), 40);
}

#[test]
fn should_combine_any_alert_with_any_channel_without_an_extra_type_per_pair() {
    let (lines, make) = wire();
    let mut alerts: Vec<Box<dyn Alert>> = vec![
        Box::new(InfoAlert::new(Box::new(EmailChannel::new(make())))),
        Box::new(UrgentAlert::new(Box::new(SlackChannel::new(make())))),
        Box::new(InfoAlert::new(Box::new(SlackChannel::new(make())))),
        Box::new(UrgentAlert::new(Box::new(EmailChannel::new(make())))),
    ];

    for alert in alerts.iter_mut() {
        alert.send("ann", "ping");
    }

    assert_eq!(
        *lines.borrow(),
        vec![
            "EMAIL to:ann subject:Info | ping",
            "SLACK @ann *URGENT* PING",
            "SLACK @ann *Info* ping",
            "EMAIL to:ann subject:URGENT | PING"
        ]
    );
}

#[test]
fn should_switch_the_channel_at_runtime() {
    let (lines, make) = wire();
    let mut alert = InfoAlert::new(Box::new(EmailChannel::new(make())));
    alert.send("ann", "first");

    alert.set_channel(Box::new(SlackChannel::new(make())));
    alert.send("ann", "second");

    assert_eq!(*lines.borrow(), vec!["EMAIL to:ann subject:Info | first", "SLACK @ann *Info* second"]);
}

#[test]
fn should_work_with_a_channel_type_it_has_never_seen() {
    let pager: Wire = Rc::new(RefCell::new(Vec::new()));
    let mut urgent = UrgentAlert::new(Box::new(Recording(pager.clone())));
    let mut digest = DigestAlert::new(Box::new(Recording(pager.clone())));

    urgent.send("oncall", "db down");
    digest.send("oncall", "a");
    digest.flush("oncall");

    assert_eq!(*pager.borrow(), vec!["oncall|URGENT|DB DOWN", "oncall|Digest (1)|a"]);
}
