# Interviewer notes — 26 Metrics Collector

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- Window boundary: an event at exactly `now − window` is excluded, one at `now` is included.
- Nearest rank with duplicates and with `p` on a rank boundary (`p × n / 100` integral vs. not).
- Empty tag set vs. any tag set; tag order.
- Counters and histograms with the same name.
- Rejected increments (`delta ≤ 0`) leave no trace in the totals nor the window.

## Likely interviewer questions
- How are tag sets keyed: sorted list of pairs, a canonical string, a map with value semantics? Cost of building the key on every increment?
- What is the data structure behind the percentile: sorted array with lazy re-sort, a sorted buffer plus merge, an order-statistic tree, a Fenwick tree over value ranges, fixed buckets? Complexity of observe / percentile?
- What do concurrent writers contend on? Striped counters, per-name locks, atomics?
- What does `countInWindow` scan? How would you keep it O(log n) (prefix sums + binary search) or O(buckets)?
- How is a read concurrent with writes consistent (snapshot or best-effort)?
- What changes when memory must be bounded (V3/V4)?

## Invariants
- `counterTotal(name)` = Σ over tag sets of `counter(name, tags)` at any quiescent moment.
- `countInWindow(name, w)` is non-decreasing in `w`, and for a huge `w` equals `counterTotal(name)`.
- `percentile` is monotone in `p`; `percentile(100)` is the maximum, `percentile(1)` for `n ≤ 100` the minimum.
- No lost updates under concurrency.

## Common mistakes
- Re-sorting the whole sample on every `percentile` call (quadratic under interleaved observe/query).
- Interpolating percentiles (not nearest-rank) or using `floor` instead of `ceil`; floating-point rank errors at boundaries (`0.07 * 100`).
- A single global lock held while sorting.
- `counter(name, tags)` doing a prefix match, or a map keyed by an unordered `HashMap<String,String>` that is later mutated.
- Window query scanning every event, or using `>=`/`<` instead of `(now−w, now]`.
- Using the wall clock instead of the injected one.
- `ConcurrentHashMap` with get-then-put (lost updates) instead of `merge`/atomic ops.

## Over-engineering traps
- `Metric`, `Counter`, `Gauge`, `Timer`, `Histogram`, `MetricRegistry`, `MetricFactory`, `Reporter` hierarchies and a plugin system for exporters in V1.
- A full time-series database for a window query that a sorted event log with prefix sums answers.

## Scalability / concurrency discussion
- Striping / `LongAdder` for hot counters; separate locks per histogram; copy-on-read snapshots.
- Histograms: batch writes into a per-thread buffer merged under the lock; or a lock-protected sorted structure with `O(log n)` operations.
- Event log for windows: append-only arrays of `(timestamp, cumulative sum)`, binary search for the left edge.

## Trade-offs worth probing
- Exact vs. approximate percentiles (memory, speed, error bounds).
- Lazily sorted buffer (fast writes, slow first read) vs. always-sorted (slower writes).
- Per-event log vs. fixed time buckets for windows.

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.
