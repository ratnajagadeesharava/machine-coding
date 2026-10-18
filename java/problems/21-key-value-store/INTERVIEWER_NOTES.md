# Interviewer notes — 21 In-Memory Key-Value Store

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- Deleting a key in an inner transaction, committing, then rolling back the outer one: the key is back.
- Plain put inside a transaction removes the TTL once committed.
- A TTL started inside a transaction keeps its original deadline after the commit.
- Rolled-back overwrite restores the old value including its old (still running) TTL.
- Prefix listing and size inside nested transactions with a mix of deletes and puts.

## Likely interviewer questions
- How do you represent a transaction: a stack of overlay maps, an undo log, copy-on-begin? What does each cost for `begin`, `get`, `commit`, `rollback`?
- How do you represent "deleted in this transaction" (tombstone) versus "not touched here"?
- Where do you evaluate expiry: on read (lazy), on write, by a sweeper? What is stored: a TTL or an absolute deadline?
- How is `keysWithPrefix` served without scanning everything? Sorted map, trie, sorted index maintained on write?
- What does `size` cost? How do you keep it right with tombstones and expired keys?

## Invariants
- A value is visible iff the innermost layer that mentions its key holds a live entry.
- Stored deadlines are absolute; they never move when a transaction commits or rolls back.
- `rollback` leaves every observable (get, keys, size) exactly as it was at the matching `begin`, apart from expiry that has happened since.
- Outer layers are never modified by writes in an inner transaction until it commits.

## Common mistakes
- Treating a delete in a transaction as "remove from the overlay" instead of recording a tombstone (the parent's value shows through again).
- Computing the deadline at commit time, or resetting a TTL on rollback.
- Storing the remaining TTL instead of an absolute deadline.
- `expired when now > deadline` instead of `>=`.
- Counting `size` from the base map only, or counting expired/tombstoned keys.
- Commit that copies the child layer over the parent but forgets tombstones (or the reverse).
- Returning keys unsorted, or an expired key in the listing.

## Over-engineering traps
- `Transaction`, `TransactionManager`, `Command` objects with `undo()`, `Entry` hierarchy, `ExpiryPolicy` interface, observers on every write. A stack of maps and one entry shape (value + optional deadline) is the core.
- A background expiry thread in V1.

## Scalability discussion
- Sorted structure (`TreeMap` / `BTreeMap`) gives `O(log n + k)` prefix listing; a hash map plus sort per query is `O(n log n)`.
- Overlay stack: `get` costs `O(depth)`; undo-log: `get` is `O(1)` but rollback costs `O(changes)`.
- Lazy expiry keeps `get` simple; `size` then needs either a scan or a deadline-ordered index.

## Trade-offs worth probing
- Overlay layers make rollback free and commit `O(changes)`; an undo log makes commit free and rollback `O(changes)`. Which does the product need?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.
