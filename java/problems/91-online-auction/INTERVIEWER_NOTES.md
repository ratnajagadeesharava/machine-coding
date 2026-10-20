# Interviewer notes — 91 Online Auction

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Exactly 5 minutes left and exactly 6 minutes left; a bid at the exact end millisecond (ended).
- Bid of exactly the starting price on an empty auction accepted; equal to the starting price when someone already bid is rejected.
- A rejected bid inside the last 5 minutes must not extend the end.
- Extension chain: a bid arriving after the old end but before the extended end is still valid.
- Self-raise: history grows, no notification, and the previous bid of that bidder stays in the history.
- Reserve equal to the top bid (sold) versus one below it; reserve 0.
- Cancel with no bids, cancel with bids, cancel by a stranger, bid on a cancelled auction.
- Notifications counts under concurrency; 16 threads bidding on 50 auctions simultaneously.

## Likely interviewer questions
- What exactly is the critical section of `placeBid`? Which fields does it touch?
- Why can't you use a single `AtomicLong` for the top bid? What else must change atomically with it?
- When is an auction 'ended' if no thread closes it? What are the pros and cons of lazy closing?
- How do you guarantee the outbid message goes to the right user when two bids race?
- Why do equal bids lose? What would change with ties broken by time?

## Invariants
- The accepted-bid history of an auction is strictly increasing in amount and its last entry is the highest bid.
- The end time never decreases and only moves because of an accepted bid in the last 5 minutes.
- Every accepted bid by a bidder other than the current leader produces exactly one OUTBID message for the previous leader.
- No accepted bid is ever below the starting price or from the seller.

## Common mistakes
- Reading the top bid, comparing and then writing outside one critical section (lost update).
- Extending the end on every bid (even far from the end) or on rejected bids.
- Using `>` instead of `>=` for the starting price, or `>=` instead of `>` against the current highest.
- Using the wall clock instead of the injected clock; calling the clock twice in one operation.
- Notifying on a self-raise, or notifying the wrong user because the previous leader was read before the lock.
- Treating a rejected low bid as an exception.

## Over-engineering traps
- A full observer/pub-sub framework for notifications that are just a per-user list.
- A BidStrategy hierarchy (English/Dutch/sealed) before the English auction passes.
- A scheduler thread to close auctions, which makes the tests non-deterministic.

## Design and scaling discussion
- Lazy closing computed from a clock keeps the system deterministic and cheap; a settlement job is still needed to charge and to notify the winner.
- For high contention shard by auction and serialise bids per auction through a single-writer queue; this gives a total order and removes lock contention at the cost of latency.
- Persist bids as an append-only log per auction; the highest bid is derived. Idempotency keys on bids protect against client retries.
- Fairness and auditability: timestamps from a single trusted clock, and bids acknowledged only after being durably written.

## Trade-off questions to ask when the candidate is done
- A lock per auction versus a lock-free compare-and-set loop on an immutable state object.
- Lazy closing versus a timer: determinism, wake-ups, and the interplay with anti-sniping.
- Notifications inside the critical section versus after it (and what a failed delivery means).
- How would you explain the strictness of 'strictly higher' to a product owner? When might ties go to the earlier bidder?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
