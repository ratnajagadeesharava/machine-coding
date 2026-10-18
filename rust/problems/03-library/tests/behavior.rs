#![allow(unused_imports, clippy::needless_range_loop)]
use p03_library::{contract::*, create};
use playground_support::{sut, FakeClock};

const HOUR: u64 = 3_600_000;
const DAY: u64 = 24 * HOUR;

fn library(clock: &FakeClock) -> Option<Box<dyn LibraryApi>> {
    let mut lib = create(clock.as_clock())?;
    lib.add_member("alice");
    lib.add_member("bob");
    lib.add_member("carol");
    Some(lib)
}

#[test]
fn should_check_out_available_book() {
    let clock = FakeClock::new(10 * DAY);
    let mut lib = sut!(library(&clock));
    lib.add_book("111", "Clean Code", 2);

    assert!(lib.checkout("alice", "111").is_some());
    assert_eq!(lib.available_copies("111"), 1);
}

#[test]
fn should_refuse_unknown_member_or_book() {
    let clock = FakeClock::new(10 * DAY);
    let mut lib = sut!(library(&clock));
    lib.add_book("111", "Clean Code", 1);

    assert!(lib.checkout("nobody", "111").is_none());
    assert!(lib.checkout("alice", "999").is_none());
    assert_eq!(lib.available_copies("111"), 1);
    assert_eq!(lib.available_copies("999"), 0);
}

#[test]
fn should_refuse_when_no_copies_available() {
    let clock = FakeClock::new(10 * DAY);
    let mut lib = sut!(library(&clock));
    lib.add_book("111", "Clean Code", 1);
    lib.checkout("alice", "111");

    assert!(lib.checkout("bob", "111").is_none());
}

#[test]
fn should_limit_member_to_three_active_loans() {
    let clock = FakeClock::new(10 * DAY);
    let mut lib = sut!(library(&clock));
    for i in 1..=4 {
        lib.add_book(&format!("B{i}"), &format!("Book {i}"), 1);
    }

    assert!(lib.checkout("alice", "B1").is_some());
    assert!(lib.checkout("alice", "B2").is_some());
    assert!(lib.checkout("alice", "B3").is_some());
    assert!(lib.checkout("alice", "B4").is_none());
    assert_eq!(lib.available_copies("B4"), 1);
}

#[test]
fn should_not_allow_same_member_to_borrow_same_book_twice() {
    let clock = FakeClock::new(10 * DAY);
    let mut lib = sut!(library(&clock));
    lib.add_book("111", "Clean Code", 3);
    lib.checkout("alice", "111");

    assert!(lib.checkout("alice", "111").is_none());
    assert_eq!(lib.available_copies("111"), 2);
}

#[test]
fn should_restore_availability_when_book_is_returned() {
    let clock = FakeClock::new(10 * DAY);
    let mut lib = sut!(library(&clock));
    lib.add_book("111", "Clean Code", 1);
    let loan = lib.checkout("alice", "111").unwrap();

    assert!(lib.return_loan(&loan).is_some());

    assert_eq!(lib.available_copies("111"), 1);
    assert!(lib.checkout("bob", "111").is_some());
}

#[test]
fn should_set_due_date_fourteen_days_after_checkout() {
    let clock = FakeClock::new(10 * DAY);
    let mut lib = sut!(library(&clock));
    lib.add_book("111", "Clean Code", 1);
    let start = 10 * DAY;
    let loan = lib.checkout("alice", "111").unwrap();

    assert_eq!(lib.due_at(&loan), Some(start + 14 * DAY));
    assert_eq!(lib.due_at("nope"), None);
}

#[test]
fn should_charge_fine_per_full_overdue_day() {
    let clock = FakeClock::new(10 * DAY);
    let mut lib = sut!(library(&clock));
    lib.add_book("111", "Clean Code", 1);
    let loan = lib.checkout("alice", "111").unwrap();

    clock.advance(16 * DAY);

    assert_eq!(lib.return_loan(&loan), Some(100));
}

#[test]
fn should_not_charge_fine_when_returned_on_due_date() {
    let clock = FakeClock::new(10 * DAY);
    let mut lib = sut!(library(&clock));
    lib.add_book("111", "Clean Code", 1);
    let loan = lib.checkout("alice", "111").unwrap();

    clock.advance(14 * DAY);

    assert_eq!(lib.return_loan(&loan), Some(0));
}

#[test]
fn should_reject_returning_the_same_loan_twice() {
    let clock = FakeClock::new(10 * DAY);
    let mut lib = sut!(library(&clock));
    lib.add_book("111", "Clean Code", 2);
    let loan = lib.checkout("alice", "111").unwrap();

    assert!(lib.return_loan(&loan).is_some());
    assert!(lib.return_loan(&loan).is_none());
    assert!(lib.return_loan("unknown").is_none());
    assert_eq!(lib.available_copies("111"), 2);
}

#[test]
fn should_hold_returned_copy_for_reserving_member() {
    let clock = FakeClock::new(10 * DAY);
    let mut lib = sut!(library(&clock));
    lib.add_book("111", "Clean Code", 1);
    let loan = lib.checkout("alice", "111").unwrap();
    assert!(lib.reserve("bob", "111"));

    lib.return_loan(&loan);

    assert_eq!(lib.available_copies("111"), 0);
    assert!(lib.checkout("carol", "111").is_none());
    assert!(lib.checkout("bob", "111").is_some());
}

#[test]
fn should_serve_reservations_in_first_come_first_served_order() {
    let clock = FakeClock::new(10 * DAY);
    let mut lib = sut!(library(&clock));
    lib.add_book("111", "Clean Code", 1);
    let first = lib.checkout("alice", "111").unwrap();
    assert!(lib.reserve("bob", "111"));
    assert!(lib.reserve("carol", "111"));

    lib.return_loan(&first);

    assert!(lib.checkout("carol", "111").is_none());
    let bobs_loan = lib.checkout("bob", "111").unwrap();
    lib.return_loan(&bobs_loan);
    assert!(lib.checkout("carol", "111").is_some());
}

#[test]
fn should_refuse_reservation_when_copy_is_available() {
    let clock = FakeClock::new(10 * DAY);
    let mut lib = sut!(library(&clock));
    lib.add_book("111", "Clean Code", 1);

    assert!(!lib.reserve("bob", "111"));
}

#[test]
fn should_refuse_duplicate_reservation_or_reservation_by_borrower() {
    let clock = FakeClock::new(10 * DAY);
    let mut lib = sut!(library(&clock));
    lib.add_book("111", "Clean Code", 1);
    lib.checkout("alice", "111");

    assert!(!lib.reserve("alice", "111")); // already has it
    assert!(lib.reserve("bob", "111"));
    assert!(!lib.reserve("bob", "111")); // already queued
}

#[test]
fn should_search_titles_case_insensitively_sorted_by_isbn() {
    let clock = FakeClock::new(10 * DAY);
    let mut lib = sut!(library(&clock));
    lib.add_book("300", "The Pragmatic Programmer", 1);
    lib.add_book("100", "Clean Code", 1);
    lib.add_book("200", "Clean Architecture", 1);

    assert_eq!(lib.search("CLEAN"), vec!["100", "200"]);
    assert_eq!(lib.search("pragmatic"), vec!["300"]);
    assert!(lib.search("zzz").is_empty());
    assert_eq!(lib.search(""), vec!["100", "200", "300"]);
}

#[test]
fn should_free_member_limit_when_loan_is_returned() {
    let clock = FakeClock::new(10 * DAY);
    let mut lib = sut!(library(&clock));
    for i in 1..=4 {
        lib.add_book(&format!("B{i}"), &format!("Book {i}"), 1);
    }
    let l1 = lib.checkout("alice", "B1").unwrap();
    lib.checkout("alice", "B2");
    lib.checkout("alice", "B3");

    lib.return_loan(&l1);

    assert!(lib.checkout("alice", "B4").is_some());
}
