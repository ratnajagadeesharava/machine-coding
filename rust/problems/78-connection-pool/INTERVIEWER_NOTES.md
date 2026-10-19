# Interviewer notes — 78 Bounded Connection Pool

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: race, deadlock, starvation, memory visibility, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Timeout of 0 or less; a very short timeout racing a release.
- Releasing twice / releasing a never-acquired id must not change the counts.
- `close` while waiters exist, and `release` after `close`.
- An interrupted waiter must not lose a connection that was signalled to it.
- One connection, many threads (pure hand-off).

## Likely interviewer questions
- Why do you recompute the remaining time after each wake-up instead of waiting for the full timeout again?
- `signal` vs `signalAll` on release: which is right and what breaks with the wrong one (hint: interrupts, timeouts)?
- Why can a semaphore alone not implement this contract?
- What would a fair pool look like, and what does it cost?
- What if a holder never releases? Who is responsible?

## Invariants
- `free + leased == size`; no id is both free and leased.
- After `close`, nothing new is handed out; counts keep converging as releases arrive.
- A woken waiter only returns an id it removed from the free list.

## Common mistakes
- `if` instead of `while`; re-waiting the full timeout; negative or zero timeouts waiting forever.
- Decrementing 'in use' on release without checking that the id was leased.
- Handing out the same id twice because the free list and the flag array are updated at different times.
- `close` that wakes nobody, or waiters that return a connection after seeing `closed`.
- Counting permits in an `AtomicInteger` and a separate list - two sources of truth.

## Over-engineering traps
- Real connection objects, proxies and a health-checker thread before the pool is correct.
- A fair-queue implementation nobody asked for.

## Concurrency discussion
- Lock + condition + free list is the straightforward answer. Semaphore + lock-free free list is faster under contention but `close` and double-release checks need extra state.

## Trade-off questions to ask when the candidate is done
- Why this primitive? What does a semaphore make awkward?
- What happens under 1,000 threads and a 5-connection pool? Does the wake-up strategy matter?
- How would you test the timeout/release race? (tiny timeouts racing releases, then checking the counts)
- How would you make it lock-free? What do you do with waiting?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.
