#![allow(unused_imports, clippy::needless_range_loop)]
use p05_vending_machine::{contract::*, create};
use playground_support::sut;

#[test]
fn invalid_slot_should_win_over_every_other_rule() {
    let mut m = sut!(create());
    assert_eq!(m.select("Q1"), "INVALID_SLOT");
}

#[test]
fn sold_out_should_win_over_insufficient_funds() {
    let mut m = sut!(create());
    m.load_product("A1", "Gum", 50, 0);
    assert_eq!(m.select("A1"), "SOLD_OUT");
}

#[test]
fn should_not_be_able_to_make_change_once_dimes_run_out() {
    let mut m = sut!(create());
    m.load_product("A1", "Gum", 90, 3);
    m.load_coins(10, 1);

    m.insert_coin(100);
    assert_eq!(m.select("A1"), "DISPENSED");
    m.take_returned_coins();
    m.insert_coin(100);

    assert_eq!(m.select("A1"), "CANNOT_MAKE_CHANGE");
    assert_eq!(m.stock_of("A1"), 2);
}

#[test]
fn cancel_should_return_inserted_coins_even_when_inventory_could_provide_others() {
    let mut m = sut!(create());
    m.load_coins(5, 10);
    m.insert_coin(25);
    m.cancel();
    assert_eq!(m.take_returned_coins(), vec![25]);
}
