# Rubric — 120 Thread-Safe Trie (Java)

| Area | Weight |
|---|---:|
| Functional correctness | 30% |
| Design quality (model, API, responsibilities) | 15% |
| Code quality | 10% |
| Testing | 5% |
| Concurrency (stated guarantees, locking granularity, no data races) | 30% |
| Performance and memory | 10% |

Score each area 0–4 (0 = absent, 2 = acceptable, 4 = would hire on this alone), multiply by weight.

## Questions for every problem

**Design**
- Did the candidate identify the correct domain concepts?
- Are responsibilities well separated; does each piece of state have one clear owner?
- Are abstractions justified by a requirement? (Not: "did they use pattern X?")
- Is the API clean, hard to misuse, and free of leaked internals?
- Can the requirements evolve (see `FOLLOWUPS.md`) without a rewrite?
- Is the implementation unnecessarily complicated?

**Correctness & quality**
- Are invariants preserved across every operation, including failed ones (no half-applied changes)?
- Are edge cases (empty, boundary, repeated, invalid operations) handled deliberately?
- Can components be tested independently of the whole system?

**Concurrency & performance**
- Is concurrency safe (or explicitly and correctly declared out of scope)? Is the locking granularity reasoned about?
- Are performance characteristics reasonable for the stated constraints?

**Language-specific**
- Are classes/interfaces justified, or did you reach for an interface by reflex?
- Are locks and concurrent collections chosen on purpose (not `synchronized` everywhere, not `ConcurrentHashMap` as a magic wand)?
- Is mutable state encapsulated; are value types immutable?

## Problem-specific

- Is each operation's cost related to the length of the word or prefix (and to the number of results), not to the number of words stored?
- Can the candidate say, for every operation, at which instant it takes effect, and show why a reader cannot see a half-inserted or half-deleted word?
- Does `wordsWithPrefix` return a real snapshot (no duplicates, sorted, no phantom words) rather than a weakly-consistent traversal, and is that choice explained?
- Is the check-then-act in `insert` / `delete` (is the word already there?) atomic, so that exactly one of many racing callers wins?
- After deletes, are dead branches handled so `startsWith` is correct and memory does not leak, without breaking concurrent inserts through the same path?
- Is `size()` exact after writers finish, and is the cost of keeping it exact (shared counter versus derived) understood?
- Is validation done before any state is touched, so rejected calls leave no trace?
- Can the candidate compare a single lock, a read-write lock, per-node locking and copy-on-write, with their trade-offs, and estimate memory per word?
