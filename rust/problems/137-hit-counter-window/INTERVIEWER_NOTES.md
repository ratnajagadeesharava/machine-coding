# Interviewer notes — 137 Configurable Time Window Hit Counter

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, concurrency bug, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Decisions fixed by the contract (candidates often ask)
- Window is the half-open span `(now - w, now]`: a hit exactly `w` old has expired. This matches "the last 60 seconds" for a stream with one hit per ms: exactly 60,000 hits are counted, not 60,001.
- The clamp uses the largest `now` the counter has *used*, reads included. A counter that only clamped writes would let `count` jump backwards when the clock is stepped back (NTP adjustment).
- `setWindow` evicts with the old window first. Without it, a lazily-cleaned structure would bring back hits that had expired but were not yet removed when the window grew. This is the hidden trap of the spec.
- No `total(key)` of all hits ever: it needs unbounded state per key or a separate monotonic counter, and is deliberately left out; `totalInWindow` keeps the focus on the window.
- `memoryProbe` is the candidate's own notion of "records"; the bound (100,000) is generous on purpose. A fixed-size array per key counts every slot, which is what the hidden many-keys scenario punishes.

## Hidden edge cases (see `interviewer/`)
- 300,000 hits arriving over 300 s with a 60 s window and NO reads in between: memory must be bounded by eviction on write, not only on read.
- 20,000 keys that each got hits once and then went quiet: `totalInWindow` is 0 and `memoryProbe` is small; per-key preallocated rings (60 slots x 20,000 keys) fail it.
- Shrink 10,000 -> 5,000 -> 20,000 with hits at 0/3000/6000/9000: dropped hits do not return, live ones expire on their own schedule.
- Clock going back and forth around an expiry boundary, including a read at a later time followed by a read at an earlier one.
- 8 writer threads while another thread flips the window every call: no hit lost, no exception, no deadlock.
- `topKeys` and `countInLast` running while 12 threads write.
- Counts of 10^9 per call over 600 minutes: totals above 2^32 (an `int` accumulator overflows).

## Likely interviewer questions
- How much memory does one key use if it receives 5,000,000 hits in one second? In one hour? What is the unit you chose to store (timestamp, bucket) and why?
- When exactly is an expired record removed? Who pays for it?
- What happens to a key that is never touched again? Does the map of keys grow forever?
- Why must `now` be clamped, and where does the clamp live under concurrency?
- What does `countInLast(k, 5000)` cost in your structure? Could it be O(1)?
- `setWindow(shorter)` while 16 threads are hitting: what does a hit that arrives mid-resize see?
- What does `topKeys(10)` cost with a million keys? Could you avoid sorting everything?

## Invariants
- `now` never decreases, for every operation and every thread.
- Stored records are ordered by time and no record is older than the window (after the operation that touched its key).
- Records of the same time/bucket are merged; the number of records per key is bounded by distinct timestamps/buckets in the window, not by hits.
- Counts are non-negative exact sums of what was recorded and not yet expired.
- A rejected call changes nothing.

## Common mistakes
- Storing one entry per hit (a list of timestamps): correct but O(hits) memory; fails the 5,000,000-hit test.
- Using `<=` instead of `<` at the boundary (a hit exactly `w` old still counted), or the same off-by-one in `countInLast`.
- Clamping only writes, or using the raw clock for reads; or clamping with a non-atomic read-modify-write so two threads move `now` backwards.
- Evicting only on read, so a write-only key grows without bound; or evicting only for the touched key so quiet keys keep their records.
- `setWindow` that just assigns the new window: growing resurrects hits that were expired but still stored.
- A circular bucket array that is not cleared when the clock jumps by more than the window (stale buckets are counted again), or that is indexed by `time % size` without remembering which time the slot belongs to.
- Unsigned underflow in `now - window` (Rust) or `int` overflow in sums (Java).
- Holding one global lock while `topKeys` sorts everything (blocks all writers).
- Sorting `topKeys` by count only (ties unstable), or including keys whose hits all expired.
- A background cleaner thread: nothing needs it and it makes the clock-driven tests non-deterministic.

## Over-engineering traps
- A strategy hierarchy of window policies before one window works.
- A scheduled executor for expiry instead of expiring lazily with the injected clock.
- A generic time-series library when pairs of numbers are enough.

## Design and scaling discussion
- Structures an interviewer hopes to hear: a queue/deque of (timestamp, count) per key with merge of equal timestamps and eviction from the front (exact, memory proportional to distinct timestamps); a circular array of fixed-width buckets indexed by time slot with the slot's start time stored next to it (constant memory per key, resolution = bucket width, and a clear-on-wrap rule); a running total per key to make `count` O(1); a concurrent map keyed by key with per-key locking or striped locks; an atomic max for the monotonic clamp.
- Trade-off: exact deque vs bucketed ring. The deque is exact at millisecond resolution but its size depends on traffic shape; the ring is constant-size but approximates the window edge by up to one bucket width. A hybrid: coarse buckets beyond the shortest query duration.
- `topKeys` options: scan and sort (O(n log n)), a bounded heap of size k (O(n log k)), or maintained incrementally; with many keys the real answer is an approximate heavy-hitters structure (count-min sketch plus a small candidate heap).
- Patterns that may come up: Strategy (window/bucket policy), Decorator (rate limiter on top of the counter). Mention only if justified.
- Distribution: merge per-node bucketed counters by summing buckets that share a slot; clock skew moves hits between neighbouring slots; sketches merge by cell-wise addition.

## Trade-off questions to ask when the candidate is done
- Exact or bucketed? What error do you accept at the window edge, and who decides?
- One lock or per-key locks? What does `hit` cost under 64 threads on one hot key (contention), and how would you stripe a single hot key?
- Where would you put the cleanup of quiet keys, and what is the worst-case pause?
- How would you test the memory bound without measuring the heap? (the probe, and what it cannot see)
- How do you test time-dependent code without sleeping? (injected clock, boundary values `w-1`, `w`, `w+1`)

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) or `cargo test --features hidden` (Rust). They only use the same public contract.
