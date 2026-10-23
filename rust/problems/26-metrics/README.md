# 26 · Metrics Collector (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 60 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building the in-process metrics library that a service embeds, the kind of thing Prometheus or Micrometer clients provide. Request handlers record **counters** ("how many requests", split by **tags** such as `route=/pay` and `status=200`) and **observations** such as latencies or payload sizes, which form a distribution you query for percentiles ("what is the 99th-percentile latency?"). Dashboards and alerts read the numbers back, including "how many in the last minute". Many threads record and read at once, and a histogram may hold millions of values.

**Terms**

- **Counter**: a named total that only goes up, in increments of a positive amount.
- **Tags**: key/value pairs attached to an increment. `{route=/pay}` and `{route=/pay, status=200}` are two different tag sets, and so is the empty set `{}`.
- **Histogram**: a named collection of recorded whole-number values (negative and repeated values are allowed).
- **Percentile (nearest-rank)**: sort the `n` recorded values; the p-th percentile is the value at 1-based position `ceil(p * n / 100)`. It is always one of the recorded values (no interpolation).
- **Window**: "the last N milliseconds", i.e. times `t` with `now - N < t <= now`.

## Requirements

1. A **counter** has a name and is incremented by a positive amount, optionally with **tags** (key/value pairs). Counters with the same name but different tag sets are tracked separately.
2. A counter can be read for an exact tag set, or as the total over all tag sets of a name.
3. A **histogram** has a name; values (whole numbers) are recorded one by one. The **percentile** of a histogram can be queried at any time and must be exact (not approximated).
4. A **window query** answers "how much was added to this counter (all tags) during the last N milliseconds".
5. Many threads record and read at the same time; no update may be lost and reads must never fail or block for long.
6. Names and tags are arbitrary non-blank text; nothing needs to be registered in advance.

## Constraints

- Time comes from the clock given to the factory.
- A histogram may receive millions of values; queries are frequent and may be interleaved with new values. Do not re-sort or re-scan everything for each query.
- Window queries over hundreds of thousands of recorded increments must stay cheap.
- Memory should not grow with the number of *queries*.

## Examples

Counters (clock at `10000`):

```
increment("http", 1, {route=/pay, status=200})     -> true
increment("http", 2, {route=/pay, status=500})     -> true
increment("http", 4, {route=/pay, status=200})     -> true
increment("http", 10, {route=/pay})                -> true
increment("http", 100, {})                         -> true
increment("http", 0, {})  -> false     increment("  ", 1, {}) -> false     (nothing recorded)
counter("http", {route=/pay, status=200})          -> 5      (1 + 4)
counter("http", {status=200})                      -> 0      (exact tag set only)
counter("http", {route=/pay})                      -> 10     (not the 1 + 2 + 4 under longer tag sets)
counter("http", {})                                -> 100
counter_total("http")                               -> 117
counter("nope", {})  -> 0     counter_total("nope") -> 0
```

Percentiles. Observe `50, 15, 40, 20, 35` (sorted: 15 20 35 40 50, `n = 5`):

| p | rank = ceil(p * 5 / 100) | result |
|---|---|---|
| 1 | 1 | 15 |
| 20 | 1 | 15 |
| 30 | 2 | 20 |
| 40 | 2 | 20 |
| 50 | 3 | 35 |
| 75 | 4 | 40 |
| 100 | 5 | 50 |

`percentile` with `p = 0`, `p = 101`, a negative `p`, or an unknown/empty histogram gives an empty result. With values `-3, 5, 5, 5, 5, 5, 5, 5, 5, 100` (`n = 10`): p10 = -3, p11 = 5, p90 = 5, p91 = 100.

Window (clock `10000`; `increment("c", 5, {})`; clock advances to `11000`; `increment("c", 3, {k=v})`):

```
count_in_window("c", 1000)  -> 3     window (10000, 11000]: the first increment, at exactly 10000, is excluded
count_in_window("c", 1001)  -> 8     window (9999, 11000]
count_in_window("c", 0)     -> 0     count_in_window("c", -10) -> 0     count_in_window("nope", 1000) -> 0
```

## Edge cases

- A counter and a histogram with the same name are independent of each other.
- Tag order does not matter, and later changes the caller makes to the slice it passed in do not affect what was recorded.
- `increment` with `delta <= 0` or a blank name, and `observe` with a blank name, return `false` and record nothing.
- A window query adds up all tag sets of that counter name; other counters never count.
- Events leave the window as the clock advances: an increment made at time `t` is counted until `now - window` reaches `t`, and is excluded exactly at `t = now - window`.
- Percentile queries interleaved with new observations always reflect every value recorded so far.

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `increment(name, delta, tags)` returns `true` when recorded and `false` (nothing recorded) if `delta ≤ 0` or the name is blank. The recorded event gets the clock's current time. An empty tag set is a valid tag set of its own. The order of tags is irrelevant; two tag sets are the same only if they have exactly the same keys with exactly the same values.
- `counter(name, tags)` is the sum of the deltas recorded with **exactly** that tag set (so `{route=/pay}` and `{route=/pay, status=200}` are different, and so is `{}`); `0` for anything unknown. `counterTotal(name)` is the sum over all tag sets; `0` if unknown.
- Counters and histograms live in separate namespaces: a counter named `x` and a histogram named `x` do not affect each other.
- `observe(name, value)` returns `true` when recorded, `false` if the name is blank. Values may be negative or repeated. `observationCount(name)` is how many values were recorded (`0` if unknown).
- `percentile(name, p)` uses the **nearest-rank** method on the recorded values `v₁ ≤ v₂ ≤ … ≤ vₙ`: for an integer `p` with `1 ≤ p ≤ 100` the answer is `v_k` where `k = ceil(p × n / 100)`. It returns empty for an unknown or empty histogram and for `p` outside `1…100`.
- `countInWindow(name, windowMillis)` is the sum of the deltas of all increments of that counter name (any tags) whose recorded time `t` satisfies `now − windowMillis < t ≤ now`, with `now` read from the clock at the time of the call. It is `0` for a window `≤ 0` or an unknown name. The clock never goes backwards.

## Test entry point (Rust)

The tests drive your system through the thin, primitive-typed contract below (`src/contract.rs`). It is
**not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up
in `src/lib.rs`.

```rust
//! Thin test entry point. Your domain model lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

pub trait MetricsApi: Send + Sync {
    /// False (nothing recorded) if `delta <= 0` or the name is blank.
    fn increment(&self, name: &str, delta: i64, tags: &[(&str, &str)]) -> bool;

    /// Sum over exactly this tag set; 0 if unknown.
    fn counter(&self, name: &str, tags: &[(&str, &str)]) -> i64;

    /// Sum over all tag sets of the counter name.
    fn counter_total(&self, name: &str) -> i64;

    /// False if the name is blank.
    fn observe(&self, name: &str, value: i64) -> bool;

    /// How many values were observed for the histogram.
    fn observation_count(&self, name: &str) -> u64;

    /// Nearest-rank percentile, `p` in 1..=100; `None` if unknown/empty or `p` out of range.
    fn percentile(&self, name: &str, p: u32) -> Option<i64>;

    /// Sum of deltas (any tags) recorded in `(now - window_millis, now]`.
    fn count_in_window(&self, name: &str, window_millis: u64) -> i64;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(_clock: Arc<dyn Clock>) -> Option<Arc<dyn MetricsApi>> {
    None
}
```

## Language notes

The trait is `&self` + `Send + Sync`. Tags arrive as slices of `(key, value)` pairs. `p` is a `u32` and `window_millis` is a `u64`, so negative values cannot occur. Note that the performance tests run in debug mode by default (`cargo test`); use `--release` to see the real numbers.

## Running

```bash
cargo test -p p26_metrics
STRICT=1 cargo test -p p26_metrics   # fail instead of skip while unimplemented
cargo test -p p26_metrics --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p26_metrics --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.
