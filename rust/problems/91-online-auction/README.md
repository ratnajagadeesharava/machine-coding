# 91 · Online Auction (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 60 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Build the bidding engine of an online auction site. A seller lists an item with a starting price, a hidden reserve
price and an end time. Bidders compete; the highest bid wins if it reaches the reserve. To stop last-second sniping, a bid that
arrives shortly before the end pushes the end back. People who are outbid should be told. Many bidders hit the same popular
auction at once and everyone must see one consistent order of bids.

## Requirements

1. `createAuction(id, seller, startingPrice, reservePrice, endTimeMs)` opens an auction now. Duplicate id, non-positive starting price, negative reserve or an end time that is not in the future are invalid.
2. `placeBid(id, bidder, amount)` is accepted (`true`) only if the auction is open, the bidder is not the seller, and the amount is at least the starting price and strictly higher than the current highest bid. A too-low or equal bid is simply not accepted (`false`), it is not an error.
3. A bid accepted in the last 5 minutes before the end moves the end to *bid time + 5 minutes* (anti-sniping). Rejected bids never extend the auction.
4. A bidder may raise their own leading bid.
5. `highestBid(id)` returns `bidder:amount` if there is a bid; `bidHistory(id)` lists the accepted bids in acceptance order as `bidder:amount`; `endTimeOf(id)` returns the current end.
6. `result(id)` for a finished auction: `SOLD:bidder:amount` if the highest bid reached the reserve, `RESERVE_NOT_MET` if there were bids but not enough, `NO_BIDS` otherwise. While the auction is open it is an error.
7. `cancel(id, seller)`: only the seller, only while there are no bids and the auction has not ended.
8. `notifications(user)` returns the user's messages oldest first: `OUTBID:auctionId:newHighestAmount` each time a *different* bidder takes the lead from them.

## Constraints

- Used from many threads; any number of bidders may bid on the same auction at the same time. 'Accepted' must mean accepted in a single, total order: the history of one auction is strictly increasing in amount.
- Concurrent bidding with unique amounts per thread must leave the globally highest bid as the winner, every time (the tests assert `bidder31:3231` / `b7:17` exactly).
- Notifications are consistent with the history: every accepted bid except the first outbids exactly one previous leader, unless it is the leader raising their own bid.
- Time comes from the injected clock (epoch ms). An auction is open while `now < end`; there is no background thread closing it, so 'ended' is computed from the clock when asked.
- Many auctions can be bid on at once; operations on different auctions should not contend (hidden test: 16 threads over 50 auctions).

## Examples

```
now = T, auction a1: start 100, reserve 500, ends T+60min
placeBid(a1, ann, 99)  -> false      placeBid(a1, ann, 100) -> true
placeBid(a1, bob, 100) -> false      placeBid(a1, bob, 150) -> true   (ann gets "OUTBID:a1:150")
at T+56min: placeBid(a1, ann, 200) -> true; end moves to T+61min
after the end: result(a1) = "RESERVE_NOT_MET" if the top bid is 400, "SOLD:ann:600" if 600
```

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `AuctionApi`; `IllegalArgumentException` (duplicate id on create, unknown auction, amount <= 0, bad create arguments), `IllegalStateException` (bid on an ended or cancelled auction, `result` while open, cancel with bids or after the end), `SecurityException` (seller bids on own auction, someone else cancels). Rust: `Auctions` trait over `u64`, `AuctionError::{InvalidInput, InvalidState, NotAllowed}`, `place_bid` returns `Auc<bool>`, queries return `Auc<..>`.
- A bid of the seller is a permission error regardless of the amount. A bid with amount 0 is invalid input (not `false`). A bid after the end is a state error even if the amount is high.
- Equal to the current highest is `false`; equal to the starting price on an empty auction is `true`; one below the starting price is `false`.
- Anti-sniping: with `endMs - now <= 5 min` an accepted bid sets `end = now + 5 min`; with 6 minutes left nothing changes. Extensions can repeat indefinitely (a bid arriving after the old end but before the extended end is still accepted).
- The history contains only accepted bids; a rejected bid is not recorded and sends no notification. A self-raise is recorded and sends no notification.
- Notifications go to the previous leader only, with the new highest amount; users with no messages get an empty list.
- `result` compares the highest bid with the reserve using `>=` (`reserve 0` sells at any bid). A cancelled auction accepts no more bids.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Time comes from the clock handed to `create` (epoch milliseconds). Amounts are whole currency units.

#[derive(Debug, PartialEq, Eq)]
pub enum AuctionError {
    /// unknown auction, duplicate id, non-positive amount/starting price, end time not in the future
    InvalidInput,
    /// the auction has ended (or was cancelled), is still open when a result is asked, or cannot be cancelled
    InvalidState,
    /// the seller bids on their own auction, or somebody else cancels it
    NotAllowed,
}

pub type Auc<T> = Result<T, AuctionError>;

pub trait Auctions: Send + Sync {
    /// Opens an auction that runs from now until `end_time_ms`. `starting_price` is the lowest allowed first bid;
    /// `reserve_price` is the price the winner must reach for the item to sell.
    fn create_auction(
        &self,
        id: &str,
        seller: &str,
        starting_price: u64,
        reserve_price: u64,
        end_time_ms: u64,
    ) -> Auc<()>;

    /// Places a bid. Accepted (`Ok(true)`) only if the auction is open, the bidder is not the seller, and the amount is
    /// at least the starting price and strictly higher than the current highest bid. A lower or equal bid is simply
    /// not accepted (`Ok(false)`). A bid accepted in the last 5 minutes before the end moves the end to
    /// "bid time + 5 minutes" (anti-sniping).
    fn place_bid(&self, id: &str, bidder: &str, amount: u64) -> Auc<bool>;

    /// "bidder:amount" of the current highest bid, if any.
    fn highest_bid(&self, id: &str) -> Auc<Option<String>>;

    /// Accepted bids in the order they were accepted, "bidder:amount".
    fn bid_history(&self, id: &str) -> Auc<Vec<String>>;

    /// The (possibly extended) end time.
    fn end_time_of(&self, id: &str) -> Auc<u64>;

    /// Outcome of a finished auction: "SOLD:bidder:amount" if the highest bid reached the reserve,
    /// "RESERVE_NOT_MET", or "NO_BIDS". `InvalidState` while the auction is still open.
    fn result(&self, id: &str) -> Auc<String>;

    /// The seller may cancel an auction that has no bids yet.
    fn cancel(&self, id: &str, seller: &str) -> Auc<()>;

    /// Messages for a user, oldest first: "OUTBID:auctionId:newHighestAmount" each time another bidder takes the
    /// lead from them.
    fn notifications(&self, user: &str) -> Vec<String>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(_clock: Arc<dyn Clock>) -> Option<Arc<dyn Auctions>> { ... }
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Time-dependent behaviour uses an injectable fake clock, never real sleeping. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Bid acceptance rules: below start, equal bid, higher bid; the history order; the highest bid.
- A leader raising their own bid is accepted and produces no notification.
- Invalid bids and auctions: unknown auction, amount 0, seller bidding, duplicate id, zero starting price, end time not in the future.
- No bids after the end (advance exactly to the end time).
- Anti-sniping: no extension with 6 minutes left, extension with 4 minutes left to `now + 5 min`, and a second bid after the original end accepted and extended again; a rejected bid never extends.
- Result only after the end; SOLD, RESERVE_NOT_MET, NO_BIDS.
- Outbid notifications for a three-user sequence (including a user who is outbid twice).
- Cancel only by the seller and only without bids; a cancelled auction rejects bids.
- Concurrency: 32 threads x 100 bids with unique amounts (strictly increasing history, the maximum wins); 8 threads x 200 bids (notification count equals history size minus 1 minus self-raises); 8 threads with different amounts repeated 100x (`b7:17` always wins).
- More edge cases (many auctions at once, repeated extensions) are in `interviewer/`.

## Language notes

`RwLock<HashMap<String, Arc<Mutex<Auction>>>>` or a single `Mutex` first; the notifications map can live in its own `Mutex<HashMap<String, Vec<String>>>`. Read the clock once per call and keep amounts as `u64`; avoid holding two locks at the same time to stay deadlock free.

## Running

```bash
cargo test -p p91_online_auction
STRICT=1 cargo test -p p91_online_auction   # fail instead of skip while unimplemented
cargo test -p p91_online_auction --features hidden   # interviewer scenarios
cargo play run 91              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`FakeClock`, `sut!`): `rust/support`.
