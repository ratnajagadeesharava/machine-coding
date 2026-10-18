use p68_state::contract::TicketError;
use p68_state::Ticket;
use playground_support::FakeClock;

const SOURCE: &str = include_str!("../src/lib.rs");
const HOUR_MS: u64 = 3_600_000;
const START: u64 = 1_000_000_000;

fn ticket(clock: &FakeClock) -> Ticket {
    Ticket::new("T-1", clock.as_clock()).unwrap()
}

fn in_progress(clock: &FakeClock) -> Ticket {
    let mut t = ticket(clock);
    t.assign("ann").unwrap();
    t
}

fn on_hold(clock: &FakeClock) -> Ticket {
    let mut t = in_progress(clock);
    t.hold("waiting for customer").unwrap();
    t
}

fn resolved(clock: &FakeClock) -> Ticket {
    let mut t = in_progress(clock);
    t.resolve("fixed").unwrap();
    t
}

fn closed(clock: &FakeClock) -> Ticket {
    let mut t = resolved(clock);
    t.close().unwrap();
    t
}

fn illegal(op: &str, state: &str) -> Result<(), TicketError> {
    Err(TicketError::Illegal(format!("cannot {op} a ticket in state {state}")))
}

#[test]
fn should_start_new_and_unassigned() {
    let clock = FakeClock::new(START);
    let t = ticket(&clock);

    assert_eq!(t.id(), "T-1");
    assert_eq!(t.state(), "NEW");
    assert_eq!(t.assigned_to(), None);
    assert!(t.comments().is_empty());
}

#[test]
fn should_move_through_the_happy_path() {
    let clock = FakeClock::new(START);
    let mut t = ticket(&clock);

    t.assign("ann").unwrap();
    assert_eq!(t.state(), "IN_PROGRESS");
    assert_eq!(t.assigned_to(), Some("ann"));
    t.hold("waiting for customer").unwrap();
    assert_eq!(t.state(), "ON_HOLD");
    t.resume().unwrap();
    assert_eq!(t.state(), "IN_PROGRESS");
    t.resolve("restarted the server").unwrap();
    assert_eq!(t.state(), "RESOLVED");
    t.close().unwrap();

    assert_eq!(t.state(), "CLOSED");
    assert_eq!(t.comments(), &["HOLD: waiting for customer".to_string(), "RESOLVED: restarted the server".to_string()]);
}

#[test]
fn should_allow_reassigning_while_work_is_ongoing_without_changing_state() {
    let clock = FakeClock::new(START);
    let mut progress = in_progress(&clock);
    let mut hold = on_hold(&clock);

    progress.assign("bob").unwrap();
    hold.assign("cy").unwrap();

    assert_eq!((progress.state().as_str(), progress.assigned_to()), ("IN_PROGRESS", Some("bob")));
    assert_eq!((hold.state().as_str(), hold.assigned_to()), ("ON_HOLD", Some("cy")));
}

#[test]
fn should_reopen_a_resolved_ticket_at_most_twice() {
    let clock = FakeClock::new(START);
    let mut t = resolved(&clock);

    t.reopen().unwrap();
    assert_eq!(t.state(), "IN_PROGRESS");
    t.resolve("again").unwrap();
    t.reopen().unwrap();
    t.resolve("and again").unwrap();

    assert_eq!(t.reopen(), Err(TicketError::ReopenLimit));
    assert_eq!(t.state(), "RESOLVED");
    assert_eq!(t.comments().len(), 5);
    assert_eq!(t.comments()[1], "REOPENED");
}

#[test]
fn should_reject_operations_that_are_not_allowed_in_the_current_state() {
    let clock = FakeClock::new(START);

    assert_eq!(ticket(&clock).resolve("x"), illegal("resolve", "NEW"));
    assert_eq!(ticket(&clock).hold("x"), illegal("hold", "NEW"));
    assert_eq!(ticket(&clock).close(), illegal("close", "NEW"));
    assert_eq!(in_progress(&clock).resume(), illegal("resume", "IN_PROGRESS"));
    assert_eq!(in_progress(&clock).reopen(), illegal("reopen", "IN_PROGRESS"));
    assert_eq!(on_hold(&clock).resolve("x"), illegal("resolve", "ON_HOLD"));
    assert_eq!(on_hold(&clock).hold("x"), illegal("hold", "ON_HOLD"));
    assert_eq!(resolved(&clock).assign("x"), illegal("assign", "RESOLVED"));
    assert_eq!(resolved(&clock).hold("x"), illegal("hold", "RESOLVED"));
}

#[test]
fn should_treat_closed_as_terminal() {
    let clock = FakeClock::new(START);
    let mut t = closed(&clock);

    assert_eq!(t.assign("x"), illegal("assign", "CLOSED"));
    assert!(t.hold("x").is_err());
    assert!(t.resume().is_err());
    assert!(t.resolve("x").is_err());
    assert!(t.reopen().is_err());
    assert!(t.close().is_err());
    assert_eq!(t.comment("late"), illegal("comment", "CLOSED"));
    assert_eq!(t.state(), "CLOSED");
}

#[test]
fn should_allow_comments_in_every_state_except_closed_in_the_order_they_were_added() {
    let clock = FakeClock::new(START);
    let mut t = ticket(&clock);

    t.comment("one").unwrap();
    t.assign("ann").unwrap();
    t.comment("two").unwrap();
    t.hold("why").unwrap();
    t.comment("three").unwrap();
    t.resume().unwrap();
    t.resolve("done").unwrap();
    t.comment("four").unwrap();

    let expected: Vec<String> = ["one", "two", "HOLD: why", "three", "RESOLVED: done", "four"].iter().map(|s| s.to_string()).collect();
    assert_eq!(t.comments(), expected.as_slice());
}

#[test]
fn should_close_resolved_tickets_after_seventy_two_hours_only() {
    let clock = FakeClock::new(START);
    let mut stale = resolved(&clock);
    let mut working = in_progress(&clock);
    let mut fresh = ticket(&clock);

    clock.set(START + 71 * HOUR_MS + 59 * 60_000);
    assert!(!stale.auto_close_if_stale());
    assert_eq!(stale.state(), "RESOLVED");

    clock.set(START + 72 * HOUR_MS);
    assert!(stale.auto_close_if_stale());
    assert_eq!(stale.state(), "CLOSED");
    assert!(!stale.auto_close_if_stale());
    assert!(!working.auto_close_if_stale());
    assert!(!fresh.auto_close_if_stale());
    assert_eq!(working.state(), "IN_PROGRESS");
}

#[test]
fn should_check_arguments_before_state_and_leave_the_ticket_untouched_on_failure() {
    let clock = FakeClock::new(START);
    let mut t = ticket(&clock);

    assert_eq!(t.assign(" "), Err(TicketError::InvalidArgument));
    assert_eq!(t.comment(""), Err(TicketError::InvalidArgument));
    assert_eq!(in_progress(&clock).hold(" "), Err(TicketError::InvalidArgument));
    assert_eq!(in_progress(&clock).resolve(""), Err(TicketError::InvalidArgument));
    assert!(matches!(Ticket::new(" ", clock.as_clock()), Err(TicketError::InvalidArgument)));

    assert_eq!(t.state(), "NEW");
    assert_eq!(t.assigned_to(), None);
    assert!(t.comments().is_empty());
}

#[test]
fn should_be_built_from_state_types_not_from_matching_on_a_state_value() {
    let clock = FakeClock::new(START);
    assert_eq!(ticket(&clock).state(), "NEW");

    let code: Vec<&str> = SOURCE.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with("//")).collect();
    let joined = code.join("\n");
    for smell in ["match self.state", "match &self.state", "match state", "match self.status", "match &self.status"] {
        assert!(!joined.contains(smell), "state-switching smell: {smell}");
    }
    let impls = code.iter().filter(|l| l.starts_with("impl") && l.contains(" for ")).count();
    assert!(impls >= 5, "one `impl <StateTrait> for <State>` per state: found {impls}, need at least 5");
}
