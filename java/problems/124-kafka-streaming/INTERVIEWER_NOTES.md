# Interviewer notes — 124 Kafka-Like Message Streaming Service

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- `Math.abs(h) % n` instead of `floorMod`: the key `"polygenelubricants"` hashes to `Integer.MIN_VALUE`, whose `abs` is still negative.
- Hashing UTF-8 bytes or code points instead of UTF-16 code units (a surrogate pair must count as two units).
- Consumer ids sorted numerically or by join time instead of plain string order (`c1 < c10 < c2`).
- Range assignment with `P mod C` extras given to the last consumers, or non-contiguous (round-robin) assignment.
- A commit from a consumer that lost the partition in a rebalance must fail; the new owner must start from the group's offset.
- Committed offsets stored on the consumer object (lost on leave/rebalance) instead of in the group.
- A failed publish (null message) must not consume a round-robin slot; keyed publishes must not move the counter.
- Retention: offsets must not be renumbered after a drop, `earliest = latest - size`, and a stale committed offset must make poll jump forward without rewriting the stored value.
- Poll that mutates the committed offset ("auto-commit") or that returns messages the consumer does not own.
- Round-robin counter incremented before validation, or read-then-write without atomicity (uneven distribution under load).
- Rebalance interleaving with poll/commit: a poll must see one assignment, not half of an update.

## Likely interviewer questions
- Why does key-based routing give per-key ordering, and what breaks it when partitions are added later?
- Where do committed offsets live and why is that not the consumer? What happens to them when everybody leaves?
- Why is "polling does not commit" useful? What delivery semantics (at-least-once, at-most-once) does commit-after-processing give?
- What exactly needs to be atomic in `publish`? Can two partitions be published to in parallel?
- How would you do poll without blocking publishers, and what does a consistent snapshot of a partition mean here?
- How is a rebalance triggered and observed by a consumer that is in the middle of processing?
- What would you change for millions of messages per partition (ring buffer / segments, memory bound)?

## Patterns and data structures an interviewer hopes to hear
- A per-topic partition array of append-only logs with a base offset (ring buffer or deque with `earliest = next - size`) rather than index shifting.
- Per-partition (or per-topic) locking for offset allocation, with `AtomicInteger`/lock-protected round-robin counter; reads under the same lock or against an immutable snapshot.
- Group state keyed by (group, topic): sorted membership (`TreeSet`), assignment as a pure function of the membership, committed offsets in a map owned by the group.
- Strategy for the assignment (range now, sticky/round-robin later); value objects for a record `(partition, offset, message)` formatted only at the contract boundary.
- Lock ordering (group before partition) or a snapshot copy to avoid deadlock between poll/commit and publish.

## Invariants
- For every partition `earliest <= latest`, `latest` only grows, and `latest - earliest <= max` when a limit is set.
- Offsets handed out per partition are exactly `0..latest-1`, each once.
- The assignment of a (group, topic) is a partition of `0..P-1` among the current members: disjoint and complete.
- A committed offset never decreases and never exceeds the log end at the time of the commit.
- Failed calls leave no trace.
