# Interviewer notes — 18 LRU Cache

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- Capacity 1: every new key evicts the previous one; updating the same key evicts nothing.
- Updating an existing key at full capacity must not evict anyone.
- A lookup of an absent key must not disturb the recency order.
- Removing an entry frees a slot: the next insert evicts nobody.
- A removed key that is re-inserted is the most recently used entry.

## Likely interviewer questions
- Which two structures give O(1) `get`/`put`/`remove`, and what does each store?
- Doubly linked list: how do you avoid null checks at the ends (sentinels)? Who owns the nodes in Rust?
- Why is `get` a mutation? What does that imply for locking (V2)?
- What does "use" mean — is `put` of an existing key a use? (the spec says yes; would a different product decision change the design?)
- How would you test eviction order without exposing internals?

## Invariants
- `size() <= capacity` after every operation.
- The map and the recency order always contain exactly the same keys.
- The discarded entry is always the one whose most recent use is oldest.

## Common mistakes
- `get` that does not refresh recency, or `put` of an existing key that does not.
- Evicting before checking whether the key already exists (loses an entry on update at full capacity).
- Linear scan (list `remove(Object)`, `Vec::remove`) on every access: passes small tests, fails the performance test.
- Forgetting to unlink on `remove` (dangling order entry leads to a later wrong eviction or a size mismatch).
- Off-by-one: evicting when `size > capacity` after inserting, or when `size == capacity` on an update.

## Over-engineering traps
- `EvictionPolicy` interface, `CacheEntry`, `CacheStats`, listeners and builders in V1. A hash map plus a recency list is the whole design.
- Generic eviction framework before a second policy exists.

## Concurrency discussion
- (V2) A single mutex around everything is correct; lock striping breaks global recency, so approximate LRU (sampling, per-segment) is the real trade-off.

## Trade-offs worth probing
- Hand-written linked list vs `LinkedHashMap` (access order) in Java; index-based arena vs `Rc<RefCell>` vs `unsafe` in Rust.
- Exact LRU vs approximations (clock algorithm).

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.
