# Interviewer notes — 140 Custom HashMap

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. Ask early what they intend to use for storage. A `HashMap` would pass most tests; the point of the problem is to catch that in review. Ask them to point at the line that decides where a key lives.
4. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- 200,000 removals in random order: an insertion-order list that is searched on removal makes this quadratic.
- Add/remove churn of five keys 100,000 times: the table must not grow (growth is checked only after a new key, against the current size).
- All-colliding hasher with 3,000 keys, then removing every other key: order and capacity must be intact.
- `putIfAbsent` from 8 threads on the same keys: exactly one winner per key.
- 8 threads doing mixed put/remove/get/putIfAbsent on 300 overlapping keys, then `keys()`, `entries()`, `size()` and `containsKey` must agree with each other.
- Counting hasher under contention: exactly one call per keyed operation, none for `size`/`keys`.
- `keys()` snapshots taken while other threads insert: no duplicates and each thread's keys appear in its own insertion order.
- `clear()` then reuse: capacity unchanged, order restarts.

## Invariants
- Every entry is reachable from exactly one bucket (determined by its hash and the current capacity) and appears exactly once in the order sequence.
- `size` equals the number of entries in the buckets and in the order sequence.
- `capacity` is a power of two, only ever doubles, and doubles only inside an insertion of a new key.
- Each entry's hash is computed once, at insertion, and reused on resize.
- The order sequence is only ever appended to or unlinked from; a re-put never moves an entry.

## Common mistakes
- Calling the hasher again for every existing key during a resize (breaks the exactly-once rule and doubles resize cost).
- Calling the hasher twice in `put` (once to look up, once to insert), or in `putIfAbsent` after `get`.
- Casting a `u64` hash to `usize`/`u32`/`i32` and then adding or multiplying without wrapping (debug-build overflow panic), or indexing with `hash % capacity` where `capacity` was computed as `usize` and the hash truncated first.
- Rounding the initial capacity wrongly (for 1 or for exact powers of two) or using `size > capacity * lf / 100` with integer division, which differs from the stated rule.
- Using `u32` for `size * 100` or `capacity * lf` (overflow for large tables).
- Growing on every `put`, including replacements, or growing in a loop until the rule holds instead of doubling once.
- Rebuilding the order by iterating the buckets after a resize, so `keys()` changes with the capacity.
- Re-put moves the key to the end, or remove-then-put leaves the key in its old position.
- Removing from a `Vec` of keys: O(n) per removal.
- Shrinking on `remove` or `clear`.
- `keys()` returning a live view, or a list the caller can use to corrupt the map.
- Locking only `put` and `remove` but not `get`/`keys` (torn reads), or holding no lock during resize. In Rust: `Rc<RefCell<..>>` links or `unsafe` raw pointers for the order list where an arena with indices would do.
- Quietly using `HashMap` for storage.

## Design and scaling discussion (patterns and data structures the interviewer hopes to hear)
- **Separate chaining**: array of buckets, each a singly linked chain of nodes. Simple deletion, degrades gracefully with load factor above 100%, cost is one allocation per entry and pointer chasing.
- **Open addressing** (linear probing, Robin Hood): cache-friendly, no per-entry allocation, but needs tombstones or backward-shift deletion and the load factor must stay well below 100%.
- **Linked entries for order**: a doubly linked list through the nodes gives O(1) append and O(1) unlink on removal and makes `keys()` independent of the bucket layout (the idea behind insertion-ordered maps in other languages). An alternative is an append-only array with tombstones and periodic compaction.
- **Power-of-two capacity**: index by masking instead of modulo; spread the high bits into the low bits so that weak hashers still use all buckets.
- **Caching the hash** in the node: cheaper equality checks (compare hashes first) and resize without rehashing.
- **Treeifying** long chains (balanced tree buckets) turns the worst case from O(n) into O(log n), which is what defends against deliberate collisions.
- **Incremental rehash**: keep old and new tables and migrate a few buckets per operation to bound latency.
- **Amortised analysis**: doubling gives amortised O(1) inserts even though one insert costs O(n).
- **Concurrency**: one lock is correct and often enough; striped locks or per-bucket locks need an ordering story for resize and for the order list; lock-free needs safe memory reclamation (epoch or hazard pointers). Compound operations (`putIfAbsent`, `keys()`) are what make naive designs fail.

## Trade-off questions to ask when the candidate is done
- What is the worst-case cost of `get`, and what input produces it? What did the all-colliding test teach you?
- Why is the capacity a power of two? What breaks if the hasher returns only multiples of 16?
- Why double instead of adding a constant? What would adding 16 each time cost over a million inserts?
- Why should resize not call the hasher? What if the hasher is expensive or has side effects?
- Where does the insertion order live and what does removal cost? Alternatives?
- Which operations hold the lock and for how long? Which one hurts most at 50 million entries?
- What changes if values may be null, or if the key set is huge and mostly read?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test -p p140_custom_hashmap --features hidden`. They only use the same public contract.
