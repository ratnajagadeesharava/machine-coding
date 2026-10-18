# Interviewer notes — 20 Rate Limiter

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- Clock starting at 0: computing `t - window` with unsigned arithmetic underflows.
- Window of 1 ms: a request and the next one a millisecond later.
- Staggered expiry: entries leave the window one by one, not all at once.
- Hammering with denied requests must not delay the next allowed request.
- One key's expiry must not affect another key.

## Likely interviewer questions
- Which algorithm did you pick (fixed window, sliding log, sliding counter, token bucket, leaky bucket) and what are its accuracy / memory / burst trade-offs?
- What exactly is stored per key and how is it pruned? What is the cost of one `allow` call?
- Where is the lock: global, per key, striped, lock-free? What does contention look like when one key is hot?
- How does the check-then-record become atomic?
- How do you evict idle keys without a background thread? With one?
- What changes with a clock that can jump?

## Invariants
- For every key and every instant, allowed requests in `(t - window, t]` never exceed `limit`.
- Denied requests leave no trace.
- Each key's stored history never holds entries older than the window (once pruned) and never more than `limit` of them.

## Common mistakes
- Check-then-act race: reading the count, then recording, without holding the key's lock across both.
- Recording denied requests (the client is punished forever by their own retries).
- Off-by-one at the window edge (using `>=`/`<=` the wrong way round).
- Fixed window reset that allows `2 * limit` across a boundary.
- Scanning the whole history per request (O(limit)); or pruning only on writes so reads see stale counts.
- A single global lock around everything, then also holding it while pruning large histories.
- Never evicting keys: memory leak with many one-off clients.

## Over-engineering traps
- `RateLimitStrategy` hierarchy with `FixedWindow`, `SlidingLog`, `TokenBucket` classes, a `RateLimiterFactory`, config objects and listeners before one algorithm works. Pick the algorithm the requirements describe.
- Background cleaner threads before measuring whether lazy pruning is enough.

## Concurrency discussion
- Per-key critical sections: a concurrent map from key to a small guarded structure keeps unrelated keys from contending.
- The map insert (first request of a key) is itself a race; use atomic compute-if-absent semantics.
- Idle-key eviction races with a concurrent request of the same key: how do you avoid resurrecting or losing history?

## Trade-offs worth probing
- Exactness (log of timestamps, O(limit) memory per key) vs approximation (counter, O(1) memory).
- Lazy pruning on access vs periodic sweeping.
- Fail-open vs fail-closed policy for the distributed variant.

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.
