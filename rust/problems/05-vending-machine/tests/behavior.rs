#![allow(unused_imports, clippy::needless_range_loop)]
use p05_vending_machine::{contract::*, create};
use playground_support::sut;

fn stocked() -> Option<Box<dyn VendingMachineApi>> {
    let mut m = create()?;
    m.load_product("A1", "Chips", 65, 2);
    m.load_product("B1", "Soda", 100, 1);
    m.load_coins(25, 4);
    m.load_coins(10, 4);
    m.load_coins(5, 4);
    Some(m)
}

#[test]
fn should_accept_supported_coins_and_track_balance() {
    let mut m = sut!(stocked());

    assert!(m.insert_coin(25));
    assert!(m.insert_coin(100));

    assert_eq!(m.balance(), 125);
}

#[test]
fn should_reject_unsupported_coin() {
    let mut m = sut!(stocked());

    assert!(!m.insert_coin(3));
    assert!(!m.insert_coin(50));

    assert_eq!(m.balance(), 0);
}

#[test]
fn should_dispense_product_and_return_change() {
    let mut m = sut!(stocked());
    m.insert_coin(100);

    assert_eq!(m.select("A1"), "DISPENSED");

    assert_eq!(m.take_returned_coins(), vec![25, 10]);
    assert_eq!(m.balance(), 0);
    assert_eq!(m.stock_of("A1"), 1);
}

#[test]
fn should_empty_tray_when_coins_are_taken() {
    let mut m = sut!(stocked());
    m.insert_coin(100);
    m.select("A1");

    m.take_returned_coins();

    assert!(m.take_returned_coins().is_empty());
}

#[test]
fn should_return_no_change_for_exact_payment() {
    let mut m = sut!(stocked());
    m.insert_coin(100);

    assert_eq!(m.select("B1"), "DISPENSED");

    assert!(m.take_returned_coins().is_empty());
}

#[test]
fn should_refuse_when_funds_are_insufficient_and_keep_balance() {
    let mut m = sut!(stocked());
    m.insert_coin(25);

    assert_eq!(m.select("A1"), "INSUFFICIENT_FUNDS");

    assert_eq!(m.balance(), 25);
    assert_eq!(m.stock_of("A1"), 2);
}

#[test]
fn should_complete_purchase_after_adding_more_coins() {
    let mut m = sut!(stocked());
    m.insert_coin(25);
    m.select("A1");

    m.insert_coin(25);
    m.insert_coin(25);

    assert_eq!(m.select("A1"), "DISPENSED");
    assert_eq!(m.take_returned_coins(), vec![10]);
}

#[test]
fn should_refuse_sold_out_product() {
    let mut m = sut!(stocked());
    m.insert_coin(100);
    m.select("B1");
    m.insert_coin(100);

    assert_eq!(m.select("B1"), "SOLD_OUT");
    assert_eq!(m.balance(), 100);
}

#[test]
fn should_refuse_invalid_slot() {
    let mut m = sut!(stocked());
    m.insert_coin(100);

    assert_eq!(m.select("Z9"), "INVALID_SLOT");
    assert_eq!(m.balance(), 100);
}

#[test]
fn should_refuse_when_exact_change_cannot_be_made() {
    let mut m = sut!(create());
    m.load_product("A1", "Chips", 65, 1);
    m.insert_coin(100);

    assert_eq!(m.select("A1"), "CANNOT_MAKE_CHANGE");

    assert_eq!(m.balance(), 100);
    assert_eq!(m.stock_of("A1"), 1);
    assert!(m.take_returned_coins().is_empty());
}

#[test]
fn should_accept_exact_payment_even_with_empty_coin_inventory() {
    let mut m = sut!(create());
    m.load_product("B1", "Soda", 100, 1);
    for _ in 0..4 {
        m.insert_coin(25);
    }

    assert_eq!(m.select("B1"), "DISPENSED");
}

#[test]
fn should_refund_exactly_the_inserted_coins_on_cancel() {
    let mut m = sut!(stocked());
    m.insert_coin(25);
    m.insert_coin(100);
    m.insert_coin(5);

    m.cancel();

    assert_eq!(m.balance(), 0);
    assert_eq!(m.take_returned_coins(), vec![100, 25, 5]);
}

#[test]
fn should_do_nothing_when_cancelling_without_credit() {
    let mut m = sut!(stocked());

    m.cancel();

    assert!(m.take_returned_coins().is_empty());
}

#[test]
fn should_use_fewest_coins_for_change() {
    let mut m = sut!(create());
    m.load_product("A1", "Gum", 10, 1);
    m.load_coins(25, 2);
    m.load_coins(10, 5);
    m.load_coins(5, 5);
    m.insert_coin(25);

    assert_eq!(m.select("A1"), "DISPENSED");

    let change = m.take_returned_coins();
    assert_eq!(change.iter().sum::<u32>(), 15);
    assert_eq!(change.len(), 2); // 10 + 5
}

#[test]
fn should_track_product_quantity_per_slot() {
    let m = sut!(stocked());

    assert_eq!(m.stock_of("A1"), 2);
    assert_eq!(m.stock_of("B1"), 1);
    assert_eq!(m.stock_of("C7"), 0);
}
