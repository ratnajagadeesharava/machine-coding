#![allow(unused_imports, clippy::needless_range_loop)]
use p03_library::{contract::*, create};
use playground_support::{sut, FakeClock};

const HOUR: u64 = 3_600_000;
const DAY: u64 = 24 * HOUR;

fn library(clock: &FakeClock) -> Option<Box<dyn LibraryApi>> {
    let mut lib = create(clock.as_clock())?;
    lib.add_member("alice");
    lib.add_member("bob");
    Some(lib)
}

#[test]
fn should_not_fine_within_the_first_overdue_day() {
    let clock = FakeClock::new(DAY);
    let mut lib = sut!(library(&clock));
    lib.add_book("1", "T", 1);
    let loan = lib.checkout("alice", "1").unwrap();
    clock.advance(14 * DAY + 23 * HOUR + 59 * 60_000);
    assert_eq!(lib.return_loan(&loan), Some(0));
}

#[test]
fn should_fine_exactly_one_day_late() {
    let clock = FakeClock::new(DAY);
    let mut lib = sut!(library(&clock));
    lib.add_book("1", "T", 1);
    let loan = lib.checkout("alice", "1").unwrap();
    clock.advance(15 * DAY);
    assert_eq!(lib.return_loan(&loan), Some(50));
}

#[test]
fn adding_copies_should_make_title_available_again() {
    let clock = FakeClock::new(DAY);
    let mut lib = sut!(library(&clock));
    lib.add_book("1", "T", 1);
    lib.checkout("alice", "1");
    assert!(lib.checkout("bob", "1").is_none());
    lib.add_book("1", "T", 1);
    assert_eq!(lib.available_copies("1"), 1);
    assert!(lib.checkout("bob", "1").is_some());
}

#[test]
fn should_refuse_reservation_by_unknown_member_or_for_unknown_book() {
    let clock = FakeClock::new(DAY);
    let mut lib = sut!(library(&clock));
    lib.add_book("1", "T", 1);
    lib.checkout("alice", "1");
    assert!(!lib.reserve("ghost", "1"));
    assert!(!lib.reserve("bob", "404"));
}

#[test]
fn multiple_held_copies_should_serve_the_first_several_reservers() {
    let clock = FakeClock::new(DAY);
    let mut lib = sut!(library(&clock));
    lib.add_member("carol");
    lib.add_member("dave");
    lib.add_book("1", "T", 2);
    let a = lib.checkout("alice", "1").unwrap();
    let b = lib.checkout("bob", "1").unwrap();
    lib.reserve("carol", "1");
    lib.reserve("dave", "1");
    lib.return_loan(&a);
    lib.return_loan(&b);
    assert!(lib.checkout("dave", "1").is_some());
    assert!(lib.checkout("carol", "1").is_some());
}
