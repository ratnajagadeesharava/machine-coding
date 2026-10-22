# Interviewer notes — 113 Multi-Module Logging Framework

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Decisions the README makes on purpose
- A failed write does **not** consume a sequence number. The number is therefore effectively decided at the moment of the successful write, in buffer order. Candidates who assign the number at accept time usually need to "give it back" or leave a gap; ask how they keep the sink order and the gap-free promise together.
- `-1` is the failure result in Java (`Err(WriteFailed)` in Rust).
- The level filter is a threshold, not equality.

## Hidden edge cases (see `interviewer/`)
- 32 threads x 1000 logs through a buffer of capacity 1: no loss, no duplicate, numbers exactly 1..32000.
- Read-your-write: right after `log` returns `n`, `query(.., n, n)` must return that exact line (store before acknowledging).
- Queries run while producers are active: every snapshot is strictly increasing with no duplicates and no holes (a prefix).
- Intermittent failures on one destination under 12 threads: acknowledged numbers are still exactly 1..K and the failure count matches the sink's failed calls.
- A sink blocked behind a latch with capacity 1: producers wait, nothing is dropped, everything completes after release.
- `close()` racing with producers: every call either returns a number (and the sink saw it) or throws `IllegalStateException`; no thread hangs on a log that was accepted but never processed.
- Brute-force model comparison of all 60 filter combinations (with a range).
- A long run of consecutive failures (46) followed by successes: numbering continues at 5.

## Likely interviewer questions
- Who assigns the sequence number and why there? What would break if it were assigned in `log()` before the queue?
- What is acknowledged when, and how does the producer thread learn the result? What does it wait on?
- What happens to a producer blocked on a full buffer when `close()` is called?
- Why is the query result a copy? What does it cost to make it?
- How do you stop the consumer thread deterministically, and what if the sink never returns?

## Invariants
- Sequence numbers of acknowledged logs are exactly 1..K, each once, in sink-delivery order.
- A log is stored/visible if and only if its sink write succeeded.
- A reader sees a prefix-consistent view: if it sees number n it sees every acknowledged number below n that matches.
- After `close()` returns no background thread is running and no accepted log is unresolved.

## Common mistakes
- Assigning the number with an `AtomicLong` at accept time and then failing the write: gaps, or reuse of a number.
- Returning from `log()` as soon as the item is enqueued (acknowledging before the write).
- Using an unbounded queue, or `offer` that silently drops when full.
- Storing the entry after completing the caller's future, so read-your-write fails sporadically.
- Consumer dies on a throwing sink; callers then hang forever.
- `close()` that interrupts or exits without draining, leaving producers waiting forever; or a producer that enqueues just after the consumer stopped.
- Sorting the result of `query` on every call, or exposing the internal list.
- Treating the level filter as equality; treating the range as exclusive.
- A global lock held during the sink write, so queries stall behind a slow sink.

## Over-engineering traps
- A logger hierarchy, appenders, layouts and a plugin system for what is a fixed set of 3 modules and 2 sinks.
- A distributed log broker, write-ahead log or persistence.
- One thread per destination before a requirement asks for it (that is V3).

## Design and scaling discussion
- Single consumer = trivial ordering and trivial numbering; the cost is head-of-line blocking behind the slowest sink (V3).
- The caller waits on a per-request completion (future/latch/reply channel); an alternative is a ticket scheme with a shared monitor.
- Storage in an append-only array in sequence order gives binary search for ranges; secondary indexes per module/destination can be added if queries are hot.
- Durability: acknowledging after the sink write is the contract; what the sink guarantees (fsync) is outside this component.

## Trade-off questions to ask when the candidate is done
- Assign at accept vs assign at write: which guarantees can each give about gaps and about sink order?
- Bounded buffer: block, drop oldest, drop newest or fail fast? What would you choose for an audit log versus debug logs?
- What would you change if `log` must never block the request thread?
- How would you test the shutdown race without sleeping?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
