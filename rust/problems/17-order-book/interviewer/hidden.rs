#![allow(unused_imports, clippy::needless_range_loop)]
use p17_order_book::{contract::*, create};
use playground_support::sut;

fn buy(b: &mut dyn OrderBookApi, price: i64, qty: i64) -> u64 {
    b.submit("BUY", price, qty).unwrap()
}

fn sell(b: &mut dyn OrderBookApi, price: i64, qty: i64) -> u64 {
    b.submit("SELL", price, qty).unwrap()
}

fn trade(buy_id: u64, sell_id: u64, price: i64, qty: i64) -> String {
    format!("{buy_id}:{sell_id}:{price}:{qty}")
}

#[test]
fn sweep_should_end_with_the_remainder_resting_at_the_incoming_limit() {
    let mut b = sut!(create());
    let s1 = sell(b.as_mut(), 101, 2);
    let s2 = sell(b.as_mut(), 102, 2);
    sell(b.as_mut(), 103, 2);

    let buyer = buy(b.as_mut(), 102, 5);

    assert_eq!(
        b.trades(),
        vec![trade(buyer, s1, 101, 2), trade(buyer, s2, 102, 2)]
    );
    assert_eq!(b.best_bid(), Some(102));
    assert_eq!(b.volume_at("BUY", 102), 1);
    assert_eq!(b.best_ask(), Some(103));
}

#[test]
fn incoming_sell_should_sweep_bids_from_high_to_low_at_the_bid_prices() {
    let mut b = sut!(create());
    let b98 = buy(b.as_mut(), 98, 1);
    let b100 = buy(b.as_mut(), 100, 1);
    let b99 = buy(b.as_mut(), 99, 1);

    let seller = sell(b.as_mut(), 98, 3);

    assert_eq!(
        b.trades(),
        vec![
            trade(b100, seller, 100, 1),
            trade(b99, seller, 99, 1),
            trade(b98, seller, 98, 1)
        ]
    );
    assert_eq!(b.best_bid(), None);
    assert_eq!(b.best_ask(), None);
}

#[test]
fn a_partially_filled_resting_order_should_keep_its_place_in_the_queue() {
    let mut b = sut!(create());
    let s1 = sell(b.as_mut(), 100, 4);
    let s2 = sell(b.as_mut(), 100, 4);

    let first = buy(b.as_mut(), 100, 1);
    let second = buy(b.as_mut(), 100, 4);

    assert_eq!(
        b.trades(),
        vec![
            trade(first, s1, 100, 1),
            trade(second, s1, 100, 3),
            trade(second, s2, 100, 1)
        ]
    );
    assert_eq!(b.remaining(s2), Some(3));
}

#[test]
fn cancelling_the_middle_of_a_queue_should_leave_the_others_in_order() {
    let mut b = sut!(create());
    let s1 = sell(b.as_mut(), 100, 1);
    let s2 = sell(b.as_mut(), 100, 1);
    let s3 = sell(b.as_mut(), 100, 1);
    b.cancel(s2);

    let buyer = buy(b.as_mut(), 100, 2);

    assert_eq!(
        b.trades(),
        vec![trade(buyer, s1, 100, 1), trade(buyer, s3, 100, 1)]
    );
    assert_eq!(b.volume_at("SELL", 100), 0);
}

#[test]
fn remaining_should_distinguish_filled_from_never_issued() {
    let mut b = sut!(create());
    let s = sell(b.as_mut(), 100, 1);
    buy(b.as_mut(), 100, 1);

    assert_eq!(b.remaining(s), Some(0));
    assert_eq!(b.remaining(123_456), None);
}
