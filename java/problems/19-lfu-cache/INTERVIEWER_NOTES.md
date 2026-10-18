# Interviewer notes — 19 LFU Cache

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- Frequency beats recency: an old but popular entry survives a newer one-hit entry.
- After an insert the lowest count is 1 again, even if every other entry has a high count.
- A key that was removed or evicted comes back with count 1 (no remembered popularity).
- Tie on count is resolved by least recent use, where gets *and* puts of existing keys refresh recency.
- Updating an existing key at full capacity discards nothing but does raise its count.

## Likely interviewer questions
- What is the O(1) structure? (key -> node, and count -> doubly linked list of nodes, plus a tracked minimum count.)
- How do you maintain the minimum count when a bucket empties, when a key is removed, and after an insert?
- Why is a heap with lazy deletion O(log n) and when is that acceptable?
- Where does the tie-break live in each structure — and what changes if ties were broken by insertion order instead?
- How would you unit test the tie-break without peeking inside?

## Invariants
- `size() <= capacity` after every operation.
- Every entry appears in exactly one count bucket, and the tracked minimum is the smallest non-empty bucket (or irrelevant when empty).
- Within a bucket, entries are ordered by last use.
- A discarded entry minimises `(count, last use)` over all entries present before the insert.

## Common mistakes
- Not counting `put` of an existing key as a use (or counting it but not refreshing the value).
- Forgetting to reset the minimum count to 1 after inserting a new key.
- Not updating the minimum when the bucket of the minimum count becomes empty during a `get`.
- Evicting the freshly inserted key (evicting after insertion instead of before).
- Remembering counts of removed/evicted keys.
- A scan for the minimum on each eviction: passes small tests, fails the performance test.

## Over-engineering traps
- `FrequencyTracker`, `EvictionPolicy`, `CacheEntry` + `Node` + `Bucket` class zoo in V1 when two maps and a few linked lists express it.
- Generic policy plug-in before there is a second policy.

## Concurrency discussion
- (V3) `get` mutates counts, so even reads need exclusion; a single mutex is the baseline, striping gives only approximate LFU.

## Trade-offs worth probing
- Bucketed linked lists (O(1)) vs ordered set keyed by `(count, tick)` (O(log n), much simpler).
- Counter overflow and aging in long-lived caches.

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.
