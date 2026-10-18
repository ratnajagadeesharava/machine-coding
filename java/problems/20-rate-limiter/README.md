# 20 · Rate Limiter (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 60 min | Level 3 — infrastructure provided. Test utilities for threads/clocks exist; business/domain code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

An API gateway must protect itself from clients that send too many requests. Build the component that decides, for
every incoming request, whether it may proceed. The limit is a number of requests per time window, tracked separately
for each client.

## Requirements

1. The limiter is created with a **limit** (maximum allowed requests), a **window length** in milliseconds and a clock. All time comes from that clock.
2. Every incoming request carries a **key** (a client id, API token, IP address, ...). Each key has its own independent budget; any string, including the empty string, is a valid key. Keys are case-sensitive.
3. The window **slides**: a request arriving at time `t` is allowed if fewer than `limit` requests were *allowed* for the same key at times in the half-open interval `(t − window, t]`. A request allowed at time `s` therefore stops counting at exactly `t = s + window`.
4. An allowed request is recorded and uses up budget. A **denied request is not recorded** and does not use up anything: hammering a limiter that says "no" never makes the client wait longer.
5. Many threads call the limiter at the same time. For one key, the number of requests allowed in any window must never exceed the limit, no matter how the calls interleave.
6. The limiter may run for a long time and see many one-off keys. Memory must not grow forever just because time passes.

## Constraints

- Millions of requests, hundreds of thousands of distinct keys, and limits that can be as large as hundreds of thousands per window.
- Every decision must be cheap: do not rescan all of a key's history on each request.
- The limit and window are positive. The clock never goes backwards.

## Examples

```
limit 2, window 1000 ms, one key
t=0     allow -> yes        t=400   allow -> yes
t=500   allow -> no         t=999   allow -> no      (denied calls are not recorded)
t=1000  allow -> yes        (the request at t=0 left the window (0, 1000]; only t=400 counts)
t=1000  allow -> no
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `allow(key)` returns `true` if the request may proceed (and records it) and `false` if it is denied (and records nothing). It reads the current time from the clock given at creation.
- The window rule is exactly the one in the requirements: allowed iff the number of allowed requests of that key at times `s` with `t − window < s <= t` is smaller than `limit`.
- Keys never influence each other.
- Creating a limiter with a limit below 1 or a window below 1 ms is invalid.

## Test entry point (Java)

The tests drive your system through the thin, primitive-typed contract below. It is **not** your domain
model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in
`solution/Solution.java`.

```java
package contract;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface RateLimiterApi {

    /**
     * Decides whether a request with this key may proceed right now (time comes from the clock given at creation).
     * Allowed requests are recorded; denied requests are not.
     */
    boolean allow(String key);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    RateLimiterApi create(int limit, long windowMillis, Clock clock)
}
```

## Language notes

`Clock` is `java.time.Clock`; read it with `clock.millis()`. `allow` is called from many threads. `create` must throw `IllegalArgumentException` for a limit or window below 1.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/20-rate-limiter test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.
