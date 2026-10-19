#![allow(unused_imports, clippy::needless_range_loop)]
use p91_online_auction::{contract::*, create};
use playground_support::{run_concurrently, sut, FakeClock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const MIN: u64 = 60_000;

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

fn auctions() -> Option<(Arc<dyn Auctions>, FakeClock)> {
    let clock = FakeClock::new(1_000_000);
    let a = create(clock.as_clock())?;
    a.create_auction("a1", "sam", 100, 500, 1_000_000 + 60 * MIN)
        .unwrap();
    Some((a, clock))
}

#[test]
fn should_accept_bids_that_beat_the_starting_price_and_the_current_leader() {
    let (a, _) = sut!(auctions());

    assert_eq!(a.place_bid("a1", "ann", 99), Ok(false)); // below the starting price
    assert_eq!(a.place_bid("a1", "ann", 100), Ok(true));
    assert_eq!(a.place_bid("a1", "bob", 100), Ok(false)); // not strictly higher
    assert_eq!(a.place_bid("a1", "bob", 150), Ok(true));
    assert_eq!(a.place_bid("a1", "ann", 120), Ok(false));

    assert_eq!(a.highest_bid("a1"), Ok(Some("bob:150".to_string())));
    assert_eq!(a.bid_history("a1"), Ok(strs(&["ann:100", "bob:150"])));
}

#[test]
fn should_let_the_leader_raise_their_own_bid() {
    let (a, _) = sut!(auctions());
    a.place_bid("a1", "ann", 100).unwrap();

    assert_eq!(a.place_bid("a1", "ann", 200), Ok(true));

    assert_eq!(a.highest_bid("a1"), Ok(Some("ann:200".to_string())));
    assert!(a.notifications("ann").is_empty());
}

#[test]
fn should_reject_invalid_bids_and_auctions() {
    let (a, clock) = sut!(auctions());
    let now = clock.as_clock().now_millis();

    assert_eq!(
        a.place_bid("nope", "ann", 100),
        Err(AuctionError::InvalidInput)
    );
    assert_eq!(a.place_bid("a1", "ann", 0), Err(AuctionError::InvalidInput));
    assert_eq!(a.place_bid("a1", "sam", 500), Err(AuctionError::NotAllowed));
    assert_eq!(
        a.create_auction("a1", "x", 1, 0, now + MIN),
        Err(AuctionError::InvalidInput)
    );
    assert_eq!(
        a.create_auction("a2", "x", 0, 0, now + MIN),
        Err(AuctionError::InvalidInput)
    );
    assert_eq!(
        a.create_auction("a3", "x", 1, 0, now),
        Err(AuctionError::InvalidInput)
    );
}

#[test]
fn should_not_accept_bids_after_the_end() {
    let (a, clock) = sut!(auctions());
    a.place_bid("a1", "ann", 100).unwrap();

    clock.advance(60 * MIN);

    assert_eq!(
        a.place_bid("a1", "bob", 500),
        Err(AuctionError::InvalidState)
    );
    assert_eq!(a.highest_bid("a1"), Ok(Some("ann:100".to_string())));
}

#[test]
fn should_extend_the_end_when_a_bid_arrives_in_the_last_five_minutes() {
    let (a, clock) = sut!(auctions());
    let end = a.end_time_of("a1").unwrap();

    clock.advance(54 * MIN); // 6 minutes left: no extension
    a.place_bid("a1", "ann", 100).unwrap();
    assert_eq!(a.end_time_of("a1"), Ok(end));

    clock.advance(2 * MIN); // 4 minutes left
    a.place_bid("a1", "bob", 110).unwrap();
    assert_eq!(
        a.end_time_of("a1"),
        Ok(clock.as_clock().now_millis() + 5 * MIN)
    );

    clock.advance(4 * MIN); // the old end has passed, the new one has not
    assert_eq!(a.place_bid("a1", "ann", 120), Ok(true));
    assert_eq!(
        a.end_time_of("a1"),
        Ok(clock.as_clock().now_millis() + 5 * MIN)
    );
}

#[test]
fn should_not_extend_on_a_rejected_bid() {
    let (a, clock) = sut!(auctions());
    a.place_bid("a1", "ann", 200).unwrap();
    clock.advance(58 * MIN);
    let end = a.end_time_of("a1").unwrap();

    assert_eq!(a.place_bid("a1", "bob", 150), Ok(false));

    assert_eq!(a.end_time_of("a1"), Ok(end));
}

#[test]
fn should_report_the_result_only_after_the_end() {
    let (a, clock) = sut!(auctions());
    a.place_bid("a1", "ann", 600).unwrap();

    assert_eq!(a.result("a1"), Err(AuctionError::InvalidState));
    clock.advance(60 * MIN);

    assert_eq!(a.result("a1"), Ok("SOLD:ann:600".to_string()));
}

#[test]
fn should_not_sell_below_the_reserve_or_without_bids() {
    let (a, clock) = sut!(auctions());
    a.create_auction("a2", "sam", 10, 0, clock.as_clock().now_millis() + 60 * MIN)
        .unwrap();
    a.place_bid("a1", "ann", 400).unwrap();

    clock.advance(60 * MIN);

    assert_eq!(a.result("a1"), Ok("RESERVE_NOT_MET".to_string()));
    assert_eq!(a.result("a2"), Ok("NO_BIDS".to_string()));
}

#[test]
fn should_notify_outbid_users_each_time_they_lose_the_lead() {
    let (a, _) = sut!(auctions());
    a.place_bid("a1", "ann", 100).unwrap();
    a.place_bid("a1", "bob", 150).unwrap();
    a.place_bid("a1", "ann", 200).unwrap();
    a.place_bid("a1", "cy", 250).unwrap();

    assert_eq!(
        a.notifications("ann"),
        strs(&["OUTBID:a1:150", "OUTBID:a1:250"])
    );
    assert_eq!(a.notifications("bob"), strs(&["OUTBID:a1:200"]));
    assert!(a.notifications("cy").is_empty());
}

#[test]
fn should_only_cancel_without_bids_and_only_as_the_seller() {
    let (a, clock) = sut!(auctions());
    a.create_auction("a2", "sam", 10, 0, clock.as_clock().now_millis() + 60 * MIN)
        .unwrap();
    a.place_bid("a1", "ann", 100).unwrap();

    assert_eq!(a.cancel("a1", "sam"), Err(AuctionError::InvalidState));
    assert_eq!(a.cancel("a2", "ann"), Err(AuctionError::NotAllowed));
    a.cancel("a2", "sam").unwrap();
    assert_eq!(
        a.place_bid("a2", "bob", 50),
        Err(AuctionError::InvalidState)
    );
}

#[test]
fn should_keep_a_strictly_increasing_history_under_concurrent_bidding() {
    let clock = FakeClock::new(1_000_000);
    let a = sut!(create(clock.as_clock()));
    a.create_auction("race", "sam", 1, 0, 1_000_000 + 60 * MIN)
        .unwrap();
    let accepted = AtomicUsize::new(0);

    run_concurrently(32, |t| {
        for i in 1..=100u64 {
            if a.place_bid("race", &format!("bidder{t}"), i * 32 + t as u64) == Ok(true) {
                accepted.fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    let history = a.bid_history("race").unwrap();
    assert_eq!(history.len(), accepted.load(Ordering::SeqCst));
    let mut previous = 0;
    for h in &history {
        let amount: u64 = h.split_once(':').unwrap().1.parse().unwrap();
        assert!(amount > previous);
        previous = amount;
    }
    assert_eq!(a.highest_bid("race"), Ok(Some("bidder31:3231".to_string()))); // 100 * 32 + 31
    assert_eq!(history.last().unwrap(), "bidder31:3231");
}

#[test]
fn should_notify_consistently_under_concurrent_bidding() {
    let clock = FakeClock::new(1_000_000);
    let a = sut!(create(clock.as_clock()));
    a.create_auction("race", "sam", 1, 0, 1_000_000 + 60 * MIN)
        .unwrap();

    run_concurrently(8, |t| {
        for i in 1..=200u64 {
            a.place_bid("race", &format!("b{t}"), i * 8 + t as u64)
                .unwrap();
        }
    });

    let outbids: usize = (0..8)
        .map(|t| a.notifications(&format!("b{t}")).len())
        .sum();
    let history = a.bid_history("race").unwrap();
    let self_raises = history
        .windows(2)
        .filter(|w| w[0].split(':').next() == w[1].split(':').next())
        .count();
    assert_eq!(outbids, history.len() - 1 - self_raises);
}

#[test]
fn should_stay_correct_over_100_runs() {
    for run in 0..100 {
        let clock = FakeClock::new(1_000_000);
        let a = sut!(create(clock.as_clock()));
        a.create_auction("r", "sam", 1, 0, 1_000_000 + 60 * MIN)
            .unwrap();

        run_concurrently(8, |t| {
            a.place_bid("r", &format!("b{t}"), 10 + t as u64).unwrap();
        });

        assert_eq!(
            a.highest_bid("r"),
            Ok(Some("b7:17".to_string())),
            "run {run}"
        );
        assert_eq!(
            a.bid_history("r").unwrap().last().unwrap(),
            "b7:17",
            "run {run}"
        );
    }
}
