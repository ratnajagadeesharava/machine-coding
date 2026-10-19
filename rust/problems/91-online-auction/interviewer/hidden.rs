#![allow(unused_imports, clippy::needless_range_loop)]
use p91_online_auction::{contract::*, create};
use playground_support::{run_concurrently, sut, FakeClock};

#[test]
fn should_handle_many_auctions_being_bid_on_at_once() {
    let clock = FakeClock::new(0);
    let a = sut!(create(clock.as_clock()));
    for i in 0..50 {
        a.create_auction(&format!("a{i}"), "seller", 1, 0, 3_600_000).unwrap();
    }

    run_concurrently(16, |t| {
        for round in 1..=500u64 {
            a.place_bid(&format!("a{}", round % 50), &format!("b{t}"), round * 16 + t as u64).unwrap();
        }
    });

    for i in 0..50 {
        assert!(a.highest_bid(&format!("a{i}")).unwrap().is_some());
    }
}

#[test]
fn should_keep_extending_while_bids_keep_arriving_in_the_last_minutes() {
    let clock = FakeClock::new(0);
    let a = sut!(create(clock.as_clock()));
    a.create_auction("x", "s", 1, 0, 600_000).unwrap();

    for i in 1..=100u64 {
        clock.advance(4 * 60_000); // always inside the last 5 minutes
        assert_eq!(a.place_bid("x", &format!("b{}", i % 2), i), Ok(true));
    }

    assert_eq!(a.end_time_of("x"), Ok(clock.as_clock().now_millis() + 5 * 60_000));
}
