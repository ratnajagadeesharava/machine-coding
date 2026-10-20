# 91 · Online Auction (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 60 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building the bidding engine of an online auction site. A seller lists an item with a starting price, a hidden reserve price (the minimum the seller will accept) and an end time. Bidders compete; when the auction ends, the highest bid wins if it reaches the reserve. To stop last-second "sniping", a bid that arrives in the final 5 minutes pushes the end back. People who are outbid are notified. Many bidders hit the same popular auction at once and everyone must see one consistent order of bids.

## Glossary

- **Starting price**: the lowest acceptable first bid.
- **Reserve price**: the seller's hidden minimum; if the top bid is below it the item is not sold.
- **Leader**: the bidder who currently holds the highest accepted bid.
- **Outbid**: a different bidder takes the lead from you. Raising your own leading bid is not being outbid.
- **Accepted bid**: a bid that satisfies all rules and is recorded; a bid that is merely too low is *not accepted* (returns `false`), which is different from an invalid request (an error).
- **Anti-sniping extension**: an accepted bid within 5 minutes of the end moves the end to *bid time + 5 minutes*.
- **Time**: epoch milliseconds from an injected clock; an auction is open while `now < end`.

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
now = T, a1: seller sam, start 100, reserve 500, ends T + 60 min

placeBid(a1, ann, 99)   -> false   (below starting price)
placeBid(a1, ann, 100)  -> true    (equal to the starting price is fine on an empty auction)
placeBid(a1, bob, 100)  -> false   (not strictly higher)
placeBid(a1, bob, 150)  -> true    ann is notified "OUTBID:a1:150"
placeBid(a1, ann, 120)  -> false
highestBid(a1) -> "bob:150"        bidHistory(a1) -> ["ann:100", "bob:150"]

Errors (not "false")
placeBid(a1, sam, 500) -> permission error (seller)     placeBid(a1, ann, 0) -> invalid input
placeBid("nope", ann, 100) -> invalid input             placeBid(a1, bob, 500) at T + 60 min -> state error (ended)

Notifications
ann 100, bob 150, ann 200, cy 250  ->  ann: ["OUTBID:a1:150", "OUTBID:a1:250"]   bob: ["OUTBID:a1:200"]   cy: []
ann bids 100 then 200 with nobody in between -> ann gets no notification

Anti-sniping
at T + 54 min (6 min left): accepted bid, end stays T + 60 min
at T + 56 min (4 min left): accepted bid, end becomes T + 61 min
at T + 60 min (old end passed, new one not): a higher bid is still accepted, end becomes T + 65 min
a rejected bid (e.g. too low) never extends the end

Result (after the end)
top bid ann:600, reserve 500 -> "SOLD:ann:600"        top bid ann:400 -> "RESERVE_NOT_MET"
no bids at all -> "NO_BIDS"                              result(a1) while still open -> state error

Cancel
cancel(a1, sam) with a bid on it -> state error      cancel(a2, ann) (not the seller) -> permission error
cancel(a2, sam) with no bids -> ok; afterwards placeBid(a2, bob, 50) -> state error
```

## Edge cases

- `createAuction` with a duplicate id, starting price `0`, or an end time `<= now` is invalid input (reserve `0` is allowed and means any bid sells).
- `result` uses `highest >= reserve` to decide SOLD.
- Exactly 5 minutes left counts as "in the last 5 minutes" (end becomes `now + 5 min`); with 6 minutes left nothing changes.
- A rejected bid is not recorded in the history and sends no notification.
- Concurrent bidders: the history of one auction is strictly increasing in amount, and the bidder with the globally highest amount wins every time (for example 32 threads bidding `i*32 + t` end with `bidder31:3231`).
- Notifications are oldest first; a user who was never outbid has an empty list.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `AuctionApi`; `IllegalArgumentException` (duplicate id on create, unknown auction, amount <= 0, bad create arguments), `IllegalStateException` (bid on an ended or cancelled auction, `result` while open, cancel with bids or after the end), `SecurityException` (seller bids on own auction, someone else cancels). Rust: `Auctions` trait over `u64`, `AuctionError::{InvalidInput, InvalidState, NotAllowed}`, `place_bid` returns `Auc<bool>`, queries return `Auc<..>`.
- A bid of the seller is a permission error regardless of the amount. A bid with amount 0 is invalid input (not `false`). A bid after the end is a state error even if the amount is high.
- Equal to the current highest is `false`; equal to the starting price on an empty auction is `true`; one below the starting price is `false`.
- Anti-sniping: with `endMs - now <= 5 min` an accepted bid sets `end = now + 5 min`; with 6 minutes left nothing changes. Extensions can repeat indefinitely (a bid arriving after the old end but before the extended end is still accepted).
- The history contains only accepted bids; a rejected bid is not recorded and sends no notification. A self-raise is recorded and sends no notification.
- Notifications go to the previous leader only, with the new highest amount; users with no messages get an empty list.
- `result` compares the highest bid with the reserve using `>=` (`reserve 0` sells at any bid). A cancelled auction accepts no more bids.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.List;
import java.util.Optional;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Time comes from the clock handed to the factory (epoch milliseconds). Amounts are whole currency units.
 */
public interface AuctionApi {

    /**
     * Opens an auction that runs from now until {@code endTimeMs}. {@code startingPrice} is the lowest allowed first
     * bid; {@code reservePrice} (>= 0) is the price the winner must reach for the item to sell. IllegalArgumentException
     * for a duplicate id, non-positive starting price, or an end time that is not in the future.
     */
    void createAuction(String auctionId, String seller, long startingPrice, long reservePrice, long endTimeMs);

    /**
     * Places a bid. A bid is accepted (returns true) only if the auction is open, the bidder is not the seller,
     * and the amount is at least the starting price and strictly higher than the current highest bid. A lower or equal
     * bid is simply not accepted (returns false). A bid accepted in the last 5 minutes before the end moves the end to
     * "bid time + 5 minutes" (anti-sniping).
     * IllegalArgumentException for an unknown auction or amount <= 0; IllegalStateException if the auction has ended;
     * SecurityException if the bidder is the seller.
     */
    boolean placeBid(String auctionId, String bidder, long amount);

    /** "bidder:amount" of the current highest bid, if any. */
    Optional<String> highestBid(String auctionId);

    /** Accepted bids in the order they were accepted, "bidder:amount". */
    List<String> bidHistory(String auctionId);

    /** The (possibly extended) end time. */
    long endTimeOf(String auctionId);

    /**
     * Outcome of a finished auction: "SOLD:bidder:amount" if the highest bid reached the reserve, "RESERVE_NOT_MET",
     * or "NO_BIDS". IllegalStateException while the auction is still open.
     */
    String result(String auctionId);

    /** The seller may cancel an auction that has no bids yet. SecurityException for others; IllegalStateException with bids or after the end. */
    void cancel(String auctionId, String seller);

    /** Messages for a user, oldest first: "OUTBID:auctionId:newHighestAmount" each time another bidder takes the lead from them. */
    List<String> notifications(String user);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static AuctionApi create(Clock clock)
}
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

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/91-online-auction test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`FakeClock`, `Candidate`): `java/testkit`.
