# Interviewer notes — 97 Idempotent Request Processor

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: safety violation (split brain, lost or divergent committed data), liveness problem (no progress), missed failure case (crash, loss, duplicate, reorder, partition), non-determinism, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Same key with a different body while the first call is blocked inside its handler: conflict, not a wait.
- Handler failure: nothing stored; the next caller (or one waiter) runs again; the failure reaches only the executor.
- Exactly 24 h - 1 ms still replays, 24 h expires; expired key accepts a different body.
- Blank or null key and null body rejected before anything is stored.
- 100,000 distinct keys, each retried three times by eight threads, run each handler exactly once.
- 50 simulated days of 2,000 keys each: expired entries must be reclaimed or memory runs out.

## Likely interviewer questions
- What states does a key go through? Draw the transitions including failure.
- How do waiters get the result, and what do they do when the executor fails?
- Why can't the handler run inside a map-wide lock? What do you lock instead?
- How do you compare request bodies? What if bodies are large?
- How and when do you reclaim expired entries without a background thread, and with one?
- What exactly does 'exactly once' mean here, and what does it not cover?

## Invariants
- For a key, at most one handler execution is in progress at any time.
- A stored response is only ever the result of a successful handler call.
- After a successful execution, every caller with the same unexpired key and body gets that same response.
- No record outlives its expiry indefinitely.

## Common mistakes
- Check-then-act on a plain map: two threads both see 'absent' and both run the handler.
- A single lock around everything, including the handler.
- Storing a failed attempt (or an exception) as the response.
- Waiters failing with the executor's exception instead of retrying.
- Ignoring body mismatch for an in-flight key.
- Never removing expired entries; or removing them but resurrecting stale responses in the same call.
- Measuring expiry from the last access instead of the store time.

## Over-engineering traps
- A distributed database and write-ahead log in a one-process exercise.
- A generic `Cache<K,V>` with eviction policies when only TTL matters.
- Event buses and listeners to tell waiters; a future or condition variable is enough.

## Distributed-systems discussion
- Stripe's `Idempotency-Key` header, AWS client tokens and Kafka's idempotent producer are the same idea: a client-chosen key plus server-side dedup for a bounded window.
- The durable version needs an atomic insert-if-absent in a shared store (a unique index, DynamoDB conditional put, Redis `SET NX`) with an in-progress marker and a lease for crashes.
- Exactly-once effect = at-least-once delivery + idempotent handling; true exactly-once across a crash needs the effect and the record to commit together (transactional outbox, or pass the key downstream).
- The retention window is a product decision: longer windows catch slow retries but cost storage; keys should be scoped per client/tenant.
- Request fingerprints (hash of the body) are cheaper than storing bodies but admit collisions in theory; store a strong hash.

## Trade-off questions to ask when the candidate is done
- Per-key locks vs striped locks vs a concurrent map with futures.
- Reject on in-flight conflict vs make the retry wait.
- Lazy expiry on access vs a sweeper: latency vs memory.
- Storing the full response vs a pointer to it.
- How would you test the hand-over deterministically?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
