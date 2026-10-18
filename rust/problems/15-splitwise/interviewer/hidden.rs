#![allow(unused_imports, clippy::needless_range_loop)]
use p15_splitwise::{contract::*, create};
use playground_support::sut;

#[test]
fn should_allocate_equal_remainder_starting_from_the_alphabetically_first_even_when_listed_last() {
    let mut s = sut!(create());
    s.add_equal_expense("Z", 101, &["C", "B", "A"]);
    // 101 / 3 = 33 r 2 -> A and B get 34, C gets 33
    assert_eq!(s.owed("A", "Z"), 34);
    assert_eq!(s.owed("B", "Z"), 34);
    assert_eq!(s.owed("C", "Z"), 33);
}

#[test]
fn settling_more_than_owed_should_flip_the_direction() {
    let mut s = sut!(create());
    s.add_equal_expense("A", 100, &["A", "B"]);
    s.settle("B", "A", 80);
    assert_eq!(s.owed("B", "A"), 0);
    assert_eq!(s.owed("A", "B"), 30);
}

#[test]
fn simplify_of_already_settled_state_should_be_empty() {
    let mut s = sut!(create());
    s.add_equal_expense("A", 100, &["A", "B"]);
    s.settle("B", "A", 50);
    assert!(s.simplify().is_empty());
}

#[test]
fn simplify_between_two_people_should_be_a_single_payment() {
    let mut s = sut!(create());
    s.add_exact_expense("A", 70, &[("B", 70)]);
    assert_eq!(s.simplify(), vec!["B>A:70"]);
}

#[test]
fn refused_expenses_should_leave_no_trace() {
    let mut s = sut!(create());
    s.add_exact_expense("A", 10, &[("B", 3), ("C", 3)]);
    s.add_percent_expense("A", 10, &[("B", 99)]);
    s.add_equal_expense("A", 10, &["B", "B"]);
    assert!(s.balances().is_empty());
    assert_eq!(s.owed("B", "A"), 0);
}
