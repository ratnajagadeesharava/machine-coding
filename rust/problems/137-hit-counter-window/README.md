# 137 · Configurable Time Window Hit Counter (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

A service wants to know "how many requests did this API key / page / user make recently?" so it can show a live dashboard, list the busiest keys and (later) throttle abusers. "Recently" is a **time window**, for example the last 60 seconds. You are building the counter: callers record hits per key, and ask how many hits fall inside the window. The same counter must also answer shorter questions ("last 5 seconds") and the window itself can be changed while the system runs. Time is not read from the machine: the counter is created with a clock the tests control, so everything is deterministic. The counter will see millions of hits per second for hot keys, and it must keep running for months.

## Problem

Build a hit counter with a configurable window, measured in milliseconds on an injected clock. Hits are recorded per key. `count(key)` returns the hits inside the window; `countInLast(key, d)` returns the hits in a shorter, recent span; `topKeys(k)` lists the busiest keys. The window can be shrunk and grown at run time. The structure must not grow with the number of hits (5,000,000 hits in two seconds must not mean 5,000,000 stored records), and many threads may record hits at once.

## Requirements

1. `create(windowMs, clock)` makes a counter. `windowMs` must be at least 1, the clock must not be null; otherwise it is an error.
2. **Time.** Every operation (reads and writes alike) takes one clock reading and works at `now = max(reading, largest now this counter has used before)`. A clock that moves backwards is never seen going back: hits are recorded at `now` (the *last seen* time) and queries are answered at `now`.
3. `hit(key)` records one hit at `now`; `hit(key, count)` records `count >= 1` hits at `now`. A `count < 1` is an error and records nothing.
4. **Window rule.** A hit recorded at time `t` is inside a window of length `w` exactly when `now - w < t <= now`. A hit that is exactly `w` ms old has expired.
5. `count(key)` is the number of hits of the key inside the configured window. A valid key that was never hit counts 0.
6. `countInLast(key, durationMs)` is the number of hits of the key with `now - durationMs < t <= now`. `durationMs` must satisfy `0 < durationMs <= current window`, otherwise it is an error (even for an unknown key).
7. `totalInWindow()` is the number of hits across all keys inside the configured window. (A counter of all hits ever recorded is deliberately not part of this contract.)
8. `setWindow(newWindowMs)` (`newWindowMs >= 1`, otherwise an error) changes the window. First, every hit that is already outside the **old** window at the current time is gone for good; then the new window applies. So shrinking permanently drops the hits that fall outside the new window, and growing does **not** resurrect hits that had already expired under the old window. `window()` returns the configured window.
9. `topKeys(k)` returns at most `k` keys that have at least one hit inside the window, ordered by windowed count descending, then by key ascending. `k <= 0` is an error. A key with no hit in the window never appears.
10. `memoryProbe()` returns the approximate number of timestamp/bucket records the counter currently keeps in memory, summed over all keys (see Glossary). It is **yours to define honestly**; the tests only check upper bounds: after 5,000,000 hits at 1,000 distinct timestamps for one key, both while they are live and after the clock moved past the window (when `count` is 0), it must be at most 100,000.
11. A null/blank key is an error everywhere a key is taken.
12. The counter must be thread-safe: 16 threads calling `hit` concurrently lose none of the hits that are inside the window.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Rust: `Result<_, CounterError>` with the single variant `InvalidArgument` (blank key, `hit_n` with `count == 0`, `duration_ms` outside `1..=window`, `set_window(0)`, `top_keys(0)`). `create(window_ms, clock)` returns `None` when `window_ms == 0`. Counts and times are `u64`, `k` is `usize`; the two-argument hit is `hit_n`. The clock is `playground_support::Clock` (`now_millis()`).
- `count`, `countInLast`, `totalInWindow` and `topKeys` never fail for a valid argument; an unknown key is 0 and an empty counter has no top keys.
- Argument validation happens before anything is changed: a rejected call records nothing and changes no window.
- The counts returned are exact (not estimates). A single `hit(key, count)` may carry very large counts (sums stay below 2^63).
- Ties in `topKeys` are broken by the natural ascending order of the key strings.
- `memoryProbe()` of a counter that never received a hit is 0.

## Examples

Examples use the Java method names; in Rust the two-argument `hit(key, n)` is `hit_n(key, n)`, `countInLast` is `count_in_last`, and so on.

```
create(1000, clock)           clock = 10000
hit("a"); hit("a", 2); hit("b")
count("a")  -> 3              count("b") -> 1            count("nobody") -> 0
totalInWindow() -> 4          topKeys(5) -> ["a", "b"]

clock = 10999   count("a") -> 3       (age 999: inside)
clock = 11000   count("a") -> 0       (age exactly 1000: expired)
```

```
create(60000, clock)          clock = 10000
hit("a", 4); clock = 40000; hit("a", 2); clock = 65000; hit("a", 1)
count("a") -> 7                                      (all three are younger than 60000)
countInLast("a", 5000)   -> 1                        (only the hit at 65000)
countInLast("a", 25000)  -> 1                        (the hit at 40000 is exactly 25000 old: outside)
countInLast("a", 25001)  -> 3
countInLast("a", 60001)  -> error                    (longer than the window)
```

```
create(10000, clock)          clock = 10000
hit("a", 1); clock = 13000; hit("a", 10); clock = 16000; hit("a", 100)
count("a") -> 111
setWindow(4000)   count("a") -> 110                  (the hit at 10000 is older than 4000: dropped for good)
setWindow(10000)  count("a") -> 110                  (growing does not bring it back)

create(1000, clock)           clock = 10000
hit("x", 7); clock = 11500; setWindow(5000)
count("x") -> 0                                      (it had already expired under the old window)
```

```
create(1000, clock)           clock = 10000
hit("a")                                              (recorded at 10000)
clock = 9000; hit("a"); hit("a", 2)                   (the clock went back: recorded at 10000 too)
count("a") -> 4
clock = 10999   count("a") -> 4        clock = 11000   count("a") -> 0

clock = 20000; hit("a"); clock = 5000                 (now stays 20000)
count("a") -> 1
clock = 20500; count("a") -> 1                        (now is 20500)
clock = 5000;  hit("a")                               (recorded at 20500)
count("a") -> 2
clock = 21000; count("a") -> 1                        (the hit at 20000 is exactly 1000 old)
```

```
topKeys: hit("pear",3) hit("apple",3) hit("zebra",9) hit("mango",1) hit("fig",3)
topKeys(10) -> [zebra, apple, fig, pear, mango]       topKeys(2) -> [zebra, apple]
```

### Edge cases

- Window boundary: age `w - 1` is inside, age `w` is outside; the same for `countInLast` with its own duration. A window of 1 ms keeps only the hits of the current millisecond.
- `countInLast(key, window())` equals `count(key)`; a duration of 0 or above the window is an error.
- After `setWindow`, `countInLast` limits follow the new window.
- A rejected `hit` (`count < 1`, blank key) records nothing; a rejected `setWindow` leaves the window as it was.
- Reads move the "last seen" time too: when the clock jumps forward and then back, a query made after the jump has already used the later time.
- Keys are case-sensitive and compared exactly; leading/trailing spaces are part of a non-blank key.
- 5,000,000 hits at 1,000 distinct timestamps: `count` is 5,000,000 while they are live, 0 once the clock passes the window, and `memoryProbe()` stays at most 100,000 throughout.

## Glossary

- **Window**: the length in milliseconds of the span that counts as "recent". A hit is inside it for `w - 1` ms after it was recorded.
- **Hit**: one occurrence of an event for a key, recorded at the counter's time.
- **Last seen time**: the largest `now` the counter has used so far; the counter never goes back to an earlier time even if the clock does.
- **Windowed count**: hits of a key (or of all keys) inside the window right now.
- **Record** (for `memoryProbe`): one stored entry that remembers when hits happened and how many: a timestamp with a count, or a time bucket with a count. A fixed-size array of slots kept for a key counts every slot of it (empty or not); a key with nothing stored counts 0.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

#[derive(Debug, PartialEq, Eq)]
pub enum CounterError {
    /// blank key, hit count 0, duration outside `1..=window`, window 0, `k` of 0
    InvalidArgument,
}

/// Time comes from the `Clock` given to `create` (`now_millis()`).
///
/// Every operation, reads included, takes ONE clock reading and uses `now = max(reading, the largest now this
/// counter has used before)`; a clock that moves backwards is therefore never seen going back. A hit is recorded
/// at that `now`. A hit recorded at time `t` is inside a window of length `w` exactly when `now - w < t <= now`
/// (a hit that is exactly `w` ms old has expired).
///
/// A blank key is `InvalidArgument` everywhere. An unknown (but valid) key counts as 0.
pub trait HitCounter: Send + Sync {
    /// Records one hit for the key at the current time.
    fn hit(&self, key: &str) -> Result<(), CounterError>;

    /// Records `count` hits at the current time. `count == 0` is `InvalidArgument`.
    fn hit_n(&self, key: &str, count: u64) -> Result<(), CounterError>;

    /// Hits of the key inside the configured window.
    fn count(&self, key: &str) -> Result<u64, CounterError>;

    /// Hits of the key in `(now - duration_ms, now]`. `InvalidArgument` unless `0 < duration_ms <= current window`;
    /// the duration is validated even for unknown keys.
    fn count_in_last(&self, key: &str, duration_ms: u64) -> Result<u64, CounterError>;

    /// Hits across all keys inside the configured window.
    fn total_in_window(&self) -> u64;

    /// Changes the window (`new_window_ms == 0` is `InvalidArgument`). First every hit that is already outside the
    /// OLD window (at the current time) is gone for good; then the new window applies. Shrinking therefore drops the
    /// hits that are not inside the new window permanently, and growing never brings back a hit that was already
    /// outside the old window.
    fn set_window(&self, new_window_ms: u64) -> Result<(), CounterError>;

    /// The configured window in milliseconds.
    fn window(&self) -> u64;

    /// Keys with at least one hit inside the window, ordered by windowed count descending then key ascending
    /// (byte-wise `str` order), at most `k` of them. `k == 0` is `InvalidArgument`.
    fn top_keys(&self, k: usize) -> Result<Vec<String>, CounterError>;

    /// Approximate memory footprint: the number of timestamp/bucket records this counter currently keeps in memory,
    /// summed over all keys (a fixed-size array of slots kept for a key counts every slot of it; a key with nothing
    /// stored counts 0). Tests only check upper bounds, never an exact value. May drop expired records first.
    fn memory_probe(&self) -> usize;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(window_ms: u64, clock: Arc<dyn Clock>) -> Option<Arc<dyn HitCounter>>
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Time-dependent behaviour uses an injected fake clock, never real sleeping. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Counting inside the window, unknown keys, the exact expiry boundary, batches with `hit(key, count)` (including very large counts), partial expiry.
- Invalid arguments (window, clock, key, count, duration, `k`) and that rejected calls change nothing.
- `countInLast` with its own boundary, the upper limit, a 1 ms window and `totalInWindow` across keys.
- `setWindow`: shrinking drops permanently, growing never resurrects, limits for `countInLast` follow the window.
- `topKeys` ordering, ties, `k` larger than the number of keys, expired keys excluded.
- A clock that moves backwards, for writes and for reads.
- 5,000,000 hits at 1,000 timestamps and the `memoryProbe` bound before and after expiry.
- Concurrency: 16 threads hitting one key, 16 threads hitting ten keys with batches, 50 repeated runs.
- More (an unread stream of hits, many quiet keys, layered shrink/grow, resizing while hitting) are in `interviewer/`.

## Running

```bash
cargo test -p p137_hit_counter_window
STRICT=1 cargo test -p p137_hit_counter_window   # fail instead of skip while unimplemented
cargo test -p p137_hit_counter_window --features hidden   # interviewer scenarios
cargo play run 137              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`FakeClock`, `sut!`, `run_concurrently`): `rust/support`.
