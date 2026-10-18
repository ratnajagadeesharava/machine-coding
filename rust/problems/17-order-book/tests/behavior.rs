#![allow(unused_imports, clippy::needless_range_loop)]
use p17_order_book::{contract::*, create};
use playground_support::sut;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

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
fn should_report_an_empty_book() {
    let b = sut!(create());

    assert_eq!(b.best_bid(), None);
    assert_eq!(b.best_ask(), None);
    assert!(b.trades().is_empty());
    assert_eq!(b.volume_at("BUY", 100), 0);
    assert_eq!(b.remaining(1), None);
}

#[test]
fn should_rest_non_crossing_orders_and_report_the_best_prices() {
    let mut b = sut!(create());
    let b1 = buy(b.as_mut(), 99, 10);
    buy(b.as_mut(), 98, 5);
    sell(b.as_mut(), 101, 7);
    sell(b.as_mut(), 102, 1);

    assert_eq!(b.best_bid(), Some(99));
    assert_eq!(b.best_ask(), Some(101));
    assert_eq!(b.volume_at("BUY", 99), 10);
    assert_eq!(b.volume_at("SELL", 101), 7);
    assert_eq!(b.volume_at("BUY", 101), 0);
    assert_eq!(b.remaining(b1), Some(10));
    assert!(b.trades().is_empty());
}

#[test]
fn should_trade_at_the_resting_sell_price_when_an_incoming_buy_is_higher() {
    let mut b = sut!(create());
    let s = sell(b.as_mut(), 100, 5);

    let buyer = buy(b.as_mut(), 105, 5);

    assert_eq!(b.trades(), vec![trade(buyer, s, 100, 5)]);
    assert_eq!(b.best_ask(), None);
    assert_eq!(b.best_bid(), None);
    assert_eq!(b.remaining(s), Some(0));
    assert_eq!(b.remaining(buyer), Some(0));
}

#[test]
fn should_trade_at_the_resting_buy_price_when_an_incoming_sell_is_lower() {
    let mut b = sut!(create());
    let buyer = buy(b.as_mut(), 100, 5);

    let seller = sell(b.as_mut(), 90, 5);

    assert_eq!(b.trades(), vec![trade(buyer, seller, 100, 5)]);
}

#[test]
fn should_trade_when_prices_are_equal() {
    let mut b = sut!(create());
    let s = sell(b.as_mut(), 100, 3);

    let buyer = buy(b.as_mut(), 100, 3);

    assert_eq!(b.trades(), vec![trade(buyer, s, 100, 3)]);
}

#[test]
fn should_not_trade_when_prices_do_not_cross() {
    let mut b = sut!(create());
    sell(b.as_mut(), 101, 3);
    buy(b.as_mut(), 100, 3);

    assert!(b.trades().is_empty());
    assert_eq!(b.best_bid(), Some(100));
    assert_eq!(b.best_ask(), Some(101));
}

#[test]
fn should_rest_the_remainder_of_a_partially_filled_incoming_order() {
    let mut b = sut!(create());
    let s = sell(b.as_mut(), 100, 5);

    let buyer = buy(b.as_mut(), 105, 8);

    assert_eq!(b.trades(), vec![trade(buyer, s, 100, 5)]);
    assert_eq!(b.remaining(buyer), Some(3));
    assert_eq!(b.best_bid(), Some(105));
    assert_eq!(b.volume_at("BUY", 105), 3);
    assert_eq!(b.best_ask(), None);
}

#[test]
fn should_keep_the_remainder_of_a_partially_filled_resting_order() {
    let mut b = sut!(create());
    let s = sell(b.as_mut(), 100, 10);

    let buyer = buy(b.as_mut(), 100, 4);

    assert_eq!(b.trades(), vec![trade(buyer, s, 100, 4)]);
    assert_eq!(b.remaining(s), Some(6));
    assert_eq!(b.volume_at("SELL", 100), 6);
    assert_eq!(b.best_bid(), None);
    assert_eq!(b.remaining(buyer), Some(0));
}

#[test]
fn should_prefer_the_best_price_and_sweep_several_levels() {
    let mut b = sut!(create());
    let s102 = sell(b.as_mut(), 102, 1);
    let s101 = sell(b.as_mut(), 101, 1);
    let s103 = sell(b.as_mut(), 103, 1);

    let buyer = buy(b.as_mut(), 102, 3);

    assert_eq!(
        b.trades(),
        vec![trade(buyer, s101, 101, 1), trade(buyer, s102, 102, 1)]
    );
    assert_eq!(b.remaining(buyer), Some(1));
    assert_eq!(b.best_bid(), Some(102));
    assert_eq!(b.best_ask(), Some(103));
    assert_eq!(b.remaining(s103), Some(1));
}

#[test]
fn should_serve_the_earliest_order_first_at_the_same_price() {
    let mut b = sut!(create());
    let s1 = sell(b.as_mut(), 100, 2);
    let s2 = sell(b.as_mut(), 100, 2);

    let buyer = buy(b.as_mut(), 100, 3);

    assert_eq!(
        b.trades(),
        vec![trade(buyer, s1, 100, 2), trade(buyer, s2, 100, 1)]
    );
    assert_eq!(b.remaining(s1), Some(0));
    assert_eq!(b.remaining(s2), Some(1));
    assert_eq!(b.volume_at("SELL", 100), 1);
}

#[test]
fn should_cancel_a_resting_order() {
    let mut b = sut!(create());
    let b1 = buy(b.as_mut(), 99, 10);
    buy(b.as_mut(), 98, 5);

    assert!(b.cancel(b1));

    assert_eq!(b.remaining(b1), Some(0));
    assert_eq!(b.volume_at("BUY", 99), 0);
    assert_eq!(b.best_bid(), Some(98));
}

#[test]
fn should_cancel_only_the_remaining_part_of_a_partially_filled_order() {
    let mut b = sut!(create());
    let s = sell(b.as_mut(), 100, 10);
    let buyer = buy(b.as_mut(), 100, 4);

    assert!(b.cancel(s));

    assert_eq!(b.best_ask(), None);
    assert_eq!(b.trades(), vec![trade(buyer, s, 100, 4)]);
}

#[test]
fn should_refuse_to_cancel_unknown_filled_or_already_cancelled_orders() {
    let mut b = sut!(create());
    let s = sell(b.as_mut(), 100, 1);
    let buyer = buy(b.as_mut(), 100, 1);
    let resting = buy(b.as_mut(), 50, 1);
    b.cancel(resting);

    assert!(!b.cancel(s));
    assert!(!b.cancel(buyer));
    assert!(!b.cancel(resting));
    assert!(!b.cancel(9_999));
}

#[test]
fn should_never_match_a_cancelled_order_and_promote_the_next_in_line() {
    let mut b = sut!(create());
    let s1 = sell(b.as_mut(), 100, 5);
    let s2 = sell(b.as_mut(), 100, 5);
    b.cancel(s1);

    let buyer = buy(b.as_mut(), 100, 5);

    assert_eq!(b.trades(), vec![trade(buyer, s2, 100, 5)]);
}

#[test]
fn should_refuse_invalid_orders_without_changing_the_book() {
    let mut b = sut!(create());

    assert_eq!(b.submit("HOLD", 100, 1), None);
    assert_eq!(b.submit("buy", 100, 1), None);
    assert_eq!(b.submit("BUY", 0, 1), None);
    assert_eq!(b.submit("BUY", -5, 1), None);
    assert_eq!(b.submit("SELL", 100, 0), None);
    assert_eq!(b.submit("SELL", 100, -3), None);

    assert_eq!(b.best_bid(), None);
    assert_eq!(b.best_ask(), None);
    assert_eq!(b.volume_at("BUY", 100), 0);
    assert_eq!(b.volume_at("SELL", 100), 0);
}

#[test]
fn should_aggregate_volume_per_level_and_side_and_ignore_invalid_sides() {
    let mut b = sut!(create());
    buy(b.as_mut(), 90, 4);
    let second = buy(b.as_mut(), 90, 6);
    buy(b.as_mut(), 80, 1);
    sell(b.as_mut(), 110, 2);

    assert_eq!(b.volume_at("BUY", 90), 10);
    b.cancel(second);
    assert_eq!(b.volume_at("BUY", 90), 4);
    assert_eq!(b.volume_at("SELL", 90), 0);
    assert_eq!(b.volume_at("SELL", 110), 2);
    assert_eq!(b.volume_at("HOLD", 90), 0);
}

#[test]
fn should_fall_back_to_the_next_level_when_the_best_level_is_exhausted() {
    let mut b = sut!(create());
    buy(b.as_mut(), 100, 1);
    buy(b.as_mut(), 99, 1);
    buy(b.as_mut(), 98, 1);

    sell(b.as_mut(), 99, 2);

    assert_eq!(b.best_bid(), Some(98));
    assert_eq!(b.trades().len(), 2);
}

#[test]
fn should_give_each_order_a_unique_id_and_return_trades_as_a_snapshot() {
    let mut b = sut!(create());
    let a = buy(b.as_mut(), 100, 1);
    let c = sell(b.as_mut(), 100, 1);
    let d = buy(b.as_mut(), 10, 1);
    let snapshot = b.trades();

    sell(b.as_mut(), 10, 1);

    assert_eq!([a, c, d].iter().collect::<HashSet<_>>().len(), 3);
    assert_eq!(snapshot.len(), 1);
    assert_eq!(b.trades().len(), 2);
}

#[test]
fn should_stay_consistent_under_a_random_workload() {
    let mut b = sut!(create());
    let mut seed: u64 = 12345;
    let mut quantities: HashMap<u64, i64> = HashMap::new();
    let mut ids: Vec<u64> = Vec::new();
    let mut cancelled: HashSet<u64> = HashSet::new();
    for _ in 0..3000 {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let r = ((seed >> 33) % 1000) as i64;
        if r < 150 && !ids.is_empty() {
            let id = ids[((seed >> 20) as usize) % ids.len()];
            if b.cancel(id) {
                cancelled.insert(id);
            }
        } else {
            let side = if r % 2 == 0 { "BUY" } else { "SELL" };
            let price = 95 + (r / 2) % 11;
            let qty = 1 + (r / 7) % 5;
            let id = b.submit(side, price, qty).unwrap();
            ids.push(id);
            quantities.insert(id, qty);
        }
        if let (Some(bid), Some(ask)) = (b.best_bid(), b.best_ask()) {
            assert!(bid < ask, "crossed book: bid {bid} ask {ask}");
        }
    }

    let mut traded: HashMap<u64, i64> = HashMap::new();
    for t in b.trades() {
        let p: Vec<i64> = t.split(':').map(|x| x.parse().unwrap()).collect();
        *traded.entry(p[0] as u64).or_insert(0) += p[3];
        *traded.entry(p[1] as u64).or_insert(0) += p[3];
    }
    for id in ids {
        let remaining = b.remaining(id).unwrap();
        if cancelled.contains(&id) {
            assert_eq!(remaining, 0);
        } else {
            let filled = traded.get(&id).copied().unwrap_or(0);
            assert_eq!(remaining, quantities[&id] - filled, "order {id}");
        }
    }
}

#[test]
fn should_handle_two_hundred_thousand_operations_without_scanning_the_book() {
    let mut b = sut!(create());
    let start = Instant::now();

    let ids: Vec<u64> = (0..100_000i64)
        .map(|i| b.submit("BUY", 1 + i % 50_000, 1).unwrap())
        .collect();
    for id in &ids[50_000..] {
        assert!(b.cancel(*id), "cancel failed for {id}");
    }
    assert_eq!(b.best_bid(), Some(50_000));
    for i in 0..50_000i64 {
        b.submit("SELL", 50_001 + i, 1).unwrap();
    }
    assert_eq!(b.best_ask(), Some(50_001));
    assert_eq!(b.volume_at("BUY", 25_000), 1);

    let sweeper = b.submit("SELL", 1, 50_000).unwrap();

    assert_eq!(b.best_bid(), None);
    let trades = b.trades();
    assert_eq!(trades.len(), 50_000);
    assert_eq!(trades[0], format!("{}:{sweeper}:50000:1", ids[49_999]));
    assert_eq!(trades[49_999], format!("{}:{sweeper}:1:1", ids[0]));
    assert!(start.elapsed() < Duration::from_secs(10));
}
