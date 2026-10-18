#![allow(unused_imports, clippy::needless_range_loop)]
use p04_coffee_machine::{contract::*, create};
use playground_support::sut;

#[test]
fn should_report_alphabetically_first_missing_ingredient() {
    let m = sut!(create());
    m.add_recipe("x", &[("zinc", 1), ("apple", 1), ("mango", 1)]);
    m.refill("mango", 5);

    assert_eq!(m.dispense("x"), "INSUFFICIENT:apple");
}

#[test]
fn should_treat_never_refilled_ingredient_as_empty() {
    let m = sut!(create());
    m.refill("water", 10);
    m.add_recipe("x", &[("water", 1), ("ghost", 1)]);

    assert_eq!(m.dispense("x"), "INSUFFICIENT:ghost");
    assert_eq!(m.stock("water"), 10);
}

#[test]
fn low_stock_should_include_fully_consumed_ingredients() {
    let m = sut!(create());
    m.refill("water", 10);
    m.add_recipe("x", &[("water", 10)]);
    m.dispense("x");

    assert_eq!(m.low_stock(1), vec!["water"]);
}

#[test]
fn replacing_a_recipe_should_affect_next_dispense_only() {
    let m = sut!(create());
    m.refill("a", 10);
    m.refill("b", 10);
    m.add_recipe("d", &[("a", 1)]);
    m.dispense("d");
    m.add_recipe("d", &[("b", 2)]);
    m.dispense("d");

    assert_eq!(m.stock("a"), 9);
    assert_eq!(m.stock("b"), 8);
}
