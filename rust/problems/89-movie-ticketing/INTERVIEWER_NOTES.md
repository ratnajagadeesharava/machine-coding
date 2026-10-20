# Interviewer notes — 89 Movie Ticket Booking

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- A hold that expires at exactly the 5-minute mark, and one that is one minute short of it.
- A hold request that fails on the last seat of a list: earlier seats must remain free.
- Seat label parsing: lowercase, `A0`, `A6` in a 5-seat row, row beyond the screen, labels like `A10` versus `A1` sorting (numeric, not lexicographic).
- Refund boundaries: exactly 120 minutes (full) and exactly 30 minutes (half), 29 minutes (zero), odd totals rounding down.
- Two shows touching on the same screen; hold on a show that started a minute ago.
- Cancel then re-book the same seats; confirm a hold twice from two threads at the same time.
- Expired holds recycled while other threads keep holding and confirming (hidden: counts must add up to the seat count).

## Likely interviewer questions
- How do holds expire if there is no background thread? What are the downsides of lazy expiry?
- What is the critical section for `hold`? Why can't you check availability and mark seats as two separate steps?
- How is `confirm` racing with the expiry of the same hold resolved?
- How do you sort `A10` after `A9`? Where do you validate labels?
- Where do the prices come from at `totalOf` time: stored at confirmation or recomputed? What if prices change later?

## Invariants
- Each seat of a show is in exactly one state at a time: free, held by one live hold, or sold to one booking.
- Sum of free + held + sold seats equals rows x seatsPerRow.
- A hold is used at most once (confirmed or released), and a confirmed seat is never released by time.
- Two shows on one screen never overlap in time.

## Common mistakes
- Marking seats one by one during `hold` without rolling back, so a failed hold leaves seats stuck.
- Evaluating expiry only in `availableSeats` but not in `hold`/`confirm`, so an expired hold still blocks or still confirms.
- Using the wall clock instead of the injected clock.
- A global lock around every show plus `Thread.sleep`-based expiry thread that races with `confirm`.
- Computing the refund with floating point or with `>` instead of `>=` at the thresholds.
- Sorting seat labels as strings, putting `A10` before `A2`.

## Over-engineering traps
- Seat-type class hierarchy (PremiumSeat, RegularSeat, ...) when a row letter decides the price.
- A scheduled-executor expiry service, observer buses and event sourcing for what is a timestamp comparison.
- Generic payment-gateway abstractions that no test or requirement mentions.

## Design and scaling discussion
- Lazy expiry (check on touch) is simple and deterministic with a fake clock; a background sweeper reclaims memory and makes `availableSeats` cheap but adds a race with `confirm` that must be resolved under the same lock.
- Concurrency granularity: per-show locks make different shows independent; per-seat CAS is possible but multi-seat holds then need ordered acquisition or a rollback.
- In a real system the seat inventory is a database row per seat with a `held_until` column and a conditional update (`UPDATE ... WHERE status='free' OR held_until<now`); hold and confirm are transactions.
- Search is a read-heavy index by (city, movie) ordered by start; it can be served from a cache that is eventually consistent, while booking must be strongly consistent.

## Trade-off questions to ask when the candidate is done
- Lazy expiry versus a sweeper thread: what do you gain and what new races appear?
- One lock per show versus per seat. When does the finer grain pay off?
- How would you prevent one user from holding the whole theatre (limits per user / per hold)?
- How would you design the system so a payment succeeding just after expiry can be reconciled?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.
