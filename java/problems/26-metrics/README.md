# 26 · Metrics Collector (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 60 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Build the in-process metrics library that a service embeds: request handlers record **counters** (how many requests,
split by tags such as `route=/pay`), and **timings/sizes** (a distribution of observed values, queried for
percentiles). Dashboards and alerts read the numbers back, including "how many in the last minute".

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

```
increment("http.requests", 1, {route=/pay, status=200})
increment("http.requests", 2, {route=/pay, status=500})
counter("http.requests", {route=/pay, status=200})   → 1
counter("http.requests", {route=/pay})                → 0     (exact tag match only)
counterTotal("http.requests")                         → 3

observe("latency", 15) observe("latency", 20) observe("latency", 35) observe("latency", 40) observe("latency", 50)
percentile("latency", 40)                             → 20
percentile("latency", 50)                             → 35
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `increment(name, delta, tags)` returns `true` when recorded and `false` (nothing recorded) if `delta ≤ 0` or the name is blank. The recorded event gets the clock's current time. An empty tag set is a valid tag set of its own. The order of tags is irrelevant; two tag sets are the same only if they have exactly the same keys with exactly the same values.
- `counter(name, tags)` is the sum of the deltas recorded with **exactly** that tag set (so `{route=/pay}` and `{route=/pay, status=200}` are different, and so is `{}`); `0` for anything unknown. `counterTotal(name)` is the sum over all tag sets; `0` if unknown.
- Counters and histograms live in separate namespaces: a counter named `x` and a histogram named `x` do not affect each other.
- `observe(name, value)` returns `true` when recorded, `false` if the name is blank. Values may be negative or repeated. `observationCount(name)` is how many values were recorded (`0` if unknown).
- `percentile(name, p)` uses the **nearest-rank** method on the recorded values `v₁ ≤ v₂ ≤ … ≤ vₙ`: for an integer `p` with `1 ≤ p ≤ 100` the answer is `v_k` where `k = ceil(p × n / 100)`. It returns empty for an unknown or empty histogram and for `p` outside `1…100`.
- `countInWindow(name, windowMillis)` is the sum of the deltas of all increments of that counter name (any tags) whose recorded time `t` satisfies `now − windowMillis < t ≤ now`, with `now` read from the clock at the time of the call. It is `0` for a window `≤ 0` or an unknown name. The clock never goes backwards.

## Test entry point (Java)

The tests drive your system through the thin, primitive-typed contract below. It is **not** your domain
model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in
`solution/Solution.java`.

```java
package contract;

import java.util.Map;
import java.util.OptionalLong;

/** Thin test entry point. Your domain model lives behind it. Must be thread-safe. */
public interface MetricsApi {

    /** False (nothing recorded) if delta <= 0 or the name is blank. */
    boolean increment(String name, long delta, Map<String, String> tags);

    /** Sum over exactly this tag set; 0 if unknown. */
    long counter(String name, Map<String, String> tags);

    /** Sum over all tag sets of the counter name. */
    long counterTotal(String name);

    /** False if the name is blank. */
    boolean observe(String name, long value);

    /** How many values were observed for the histogram. */
    long observationCount(String name);

    /** Nearest-rank percentile, p in 1..100; empty if unknown/empty or p out of range. */
    OptionalLong percentile(String name, int p);

    /** Sum of deltas (any tags) recorded in (now - windowMillis, now]. */
    long countInWindow(String name, long windowMillis);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    MetricsApi create(Clock clock)
}
```

## Language notes

`Clock` is `java.time.Clock`. Tags arrive as a `Map<String, String>` that the caller may reuse or mutate afterwards - copy what you keep. All methods are called from many threads in the concurrency tests.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/26-metrics test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.
