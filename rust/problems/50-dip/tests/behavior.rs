use p50_dip::contract::*;
use p50_dip::ReminderService;
use playground_support::FakeClock;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

const NOW: u64 = 1_000_000_000;
const HOUR: u64 = 3_600_000;

#[derive(Default)]
struct SenderState {
    sent: Vec<String>,
    fail_for: HashSet<String>,
}

struct FakeSender(Rc<RefCell<SenderState>>);

impl MessageSender for FakeSender {
    fn send(&self, phone: &str, text: &str) -> bool {
        let mut s = self.0.borrow_mut();
        s.sent.push(format!("{phone} | {text}"));
        !s.fail_for.contains(phone)
    }
}

#[derive(Default)]
struct RepoState {
    all: Vec<Appointment>,
    reminded: HashSet<String>,
    queries: Vec<(u64, u64)>,
}

struct FakeRepository(Rc<RefCell<RepoState>>);

impl AppointmentRepository for FakeRepository {
    fn find_between(&self, from_ms: u64, to_ms: u64) -> Vec<Appointment> {
        let mut s = self.0.borrow_mut();
        s.queries.push((from_ms, to_ms));
        s.all
            .iter()
            .filter(|a| !s.reminded.contains(&a.id) && a.time_ms >= from_ms && a.time_ms <= to_ms)
            .cloned()
            .collect()
    }
    fn mark_reminded(&self, id: &str) {
        self.0.borrow_mut().reminded.insert(id.to_string());
    }
}

struct Fixture {
    sender: Rc<RefCell<SenderState>>,
    repo: Rc<RefCell<RepoState>>,
    clock: FakeClock,
    service: ReminderService,
}

fn fixture() -> Fixture {
    let sender = Rc::new(RefCell::new(SenderState::default()));
    let repo = Rc::new(RefCell::new(RepoState::default()));
    let clock = FakeClock::new(NOW);
    let service = ReminderService::new(
        Box::new(FakeSender(sender.clone())),
        Box::new(FakeRepository(repo.clone())),
        clock.as_clock(),
    );
    Fixture { sender, repo, clock, service }
}

fn appointment(f: &Fixture, id: &str, phone: &str, offset_ms: i64) {
    let time_ms = (NOW as i64 + offset_ms) as u64;
    f.repo.borrow_mut().all.push(Appointment { id: id.into(), phone: phone.into(), time_ms });
}

#[test]
fn should_ask_the_repository_for_the_next_twenty_four_hours_using_the_injected_clock() {
    let f = fixture();

    f.service.send_due_reminders();

    assert_eq!(f.repo.borrow().queries, vec![(NOW, NOW + 24 * HOUR)]);
}

#[test]
fn should_send_the_message_to_the_right_phone_and_mark_the_appointment() {
    let f = fixture();
    appointment(&f, "a1", "+100", 3 * HOUR as i64);

    let sent = f.service.send_due_reminders();

    assert_eq!(sent, 1);
    assert_eq!(f.sender.borrow().sent, vec![format!("+100 | Reminder: appointment at {}", NOW + 3 * HOUR)]);
    assert!(f.repo.borrow().reminded.contains("a1"));
}

#[test]
fn should_not_mark_an_appointment_when_the_sender_refuses_and_should_carry_on() {
    let f = fixture();
    appointment(&f, "a1", "+100", HOUR as i64);
    appointment(&f, "a2", "+200", 2 * HOUR as i64);
    appointment(&f, "a3", "+300", 3 * HOUR as i64);
    f.sender.borrow_mut().fail_for.insert("+200".into());

    let sent = f.service.send_due_reminders();

    assert_eq!(sent, 2);
    let reminded = f.repo.borrow().reminded.clone();
    assert_eq!(reminded, ["a1", "a3"].iter().map(|s| s.to_string()).collect::<HashSet<_>>());
}

#[test]
fn should_do_nothing_when_nothing_is_due() {
    let f = fixture();
    appointment(&f, "late", "+100", 30 * HOUR as i64);
    appointment(&f, "past", "+200", -5 * 60_000);

    assert_eq!(f.service.send_due_reminders(), 0);
    assert!(f.sender.borrow().sent.is_empty());
}

#[test]
fn should_retry_only_what_failed_on_the_next_run() {
    let f = fixture();
    appointment(&f, "a1", "+100", HOUR as i64);
    appointment(&f, "a2", "+200", 2 * HOUR as i64);
    f.sender.borrow_mut().fail_for.insert("+200".into());
    f.service.send_due_reminders();
    f.sender.borrow_mut().sent.clear();
    f.sender.borrow_mut().fail_for.clear();

    let second = f.service.send_due_reminders();

    assert_eq!(second, 1);
    assert_eq!(f.sender.borrow().sent.len(), 1);
    assert!(f.sender.borrow().sent[0].starts_with("+200"));
}

#[test]
fn should_include_both_ends_of_the_window() {
    let f = fixture();
    appointment(&f, "now", "+100", 0);
    appointment(&f, "edge", "+200", 24 * HOUR as i64);
    appointment(&f, "just-after", "+300", 24 * HOUR as i64 + 1);

    assert_eq!(f.service.send_due_reminders(), 2);
}

#[test]
fn should_follow_the_clock_when_time_passes() {
    let f = fixture();
    appointment(&f, "tomorrow", "+100", 30 * HOUR as i64);
    assert_eq!(f.service.send_due_reminders(), 0);

    f.clock.advance(10 * HOUR);

    assert_eq!(f.service.send_due_reminders(), 1);
    assert_eq!(f.repo.borrow().queries[1].0, NOW + 10 * HOUR);
}
