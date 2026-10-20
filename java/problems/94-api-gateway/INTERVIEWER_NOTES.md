# Interviewer notes — 94 API Gateway

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- A 404 or a 401 must not spend a token (client with capacity 1 still succeeds afterwards).
- A 503 (all circuits open) does spend the token, but calls nothing.
- Half a token is not enough (500 ms at 60 rpm); an hour idle yields only `capacity` tokens.
- Exactly 29 999 ms after opening the circuit is still open; at 30 000 ms a trial goes through.
- A failing half-open trial re-opens the circuit for a fresh 30 s; two failures then a success never reach three.
- Concurrent requests against a slow backend must overlap (32 threads x 5 calls of 50 ms finish in well under 3 s).
- Many threads hammering a failing backend call it at most 3 + (requests already in flight) times.

## Likely interviewer questions
- Walk me through exactly what `handle` does for a request on a route whose first backend is down.
- Why refill lazily instead of with a timer? What precision problems come with fractions of a token?
- What is the state machine of your breaker? What happens to a second request that arrives while the trial is running?
- Where does the lock for the token bucket start and end? Is any lock held across a backend call?
- Does a failover attempt cost the client another token? Should it?
- How do two routes that share a backend share its breaker?

## Invariants
- A bucket never holds more than `capacity` tokens and never goes negative; N tokens admit exactly N requests.
- A backend whose circuit is open is not called until 30 s have passed.
- At most one trial request is in flight for a half-open backend.
- `successCount` of all clients sums to the number of 200 responses; `callCount` equals the calls actually made.

## Common mistakes
- Checking tokens and decrementing in two steps (over-admission under concurrency).
- Holding one global lock around the whole `handle` so slow backends serialise everybody.
- Charging a token for 404, or not charging for 502/503.
- Counting 3 failures in total instead of 3 in a row (not resetting on success).
- Letting every request through once the 30 s are over instead of a single trial.
- Advancing the round-robin counter per tried backend, which skews the distribution after a failover.
- Using the wall clock instead of the injected clock.

## Over-engineering traps
- A plugin/filter-chain framework with generic middleware before the five-step pipeline works.
- A background thread for token refill or circuit timeouts; lazy evaluation against the clock is enough.
- A trie for route lookup before a longest-prefix scan over a few routes passes.

## Design and scaling discussion
- Real gateways (Envoy, Kong, NGINX, AWS API Gateway) put auth, rate limiting, routing and outlier detection in separate filters; state such as buckets is per instance unless backed by a shared store like Redis.
- Distributed limits trade accuracy for latency: central counter (exact, extra hop), local quotas with periodic sync (cheap, bursts), or gossip. Token bucket allows bursts; leaky bucket and sliding windows smooth them.
- Circuit breakers protect callers and recovering backends; Envoy's outlier detection ejects hosts and uses a max-ejection percentage so a bad signal cannot eject the fleet.
- Failover without budgets amplifies outages (retry storms). Hedged requests and retry budgets are the usual remedies.
- Config as an immutable snapshot swapped atomically (RCU style) keeps the hot path lock-free for routing.

## Trade-off questions to ask when the candidate is done
- Token bucket vs fixed window vs sliding window log: burst behaviour, memory per client, boundary effects.
- Per-backend breaker vs per-route breaker: what do you lose by sharing it?
- Fail fast with 503 vs queue the request: what happens to latency and memory?
- Where would you put the rate limiter relative to routing, and why does the contract place it after the 404 check?
- How would you test the half-open state deterministically?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
