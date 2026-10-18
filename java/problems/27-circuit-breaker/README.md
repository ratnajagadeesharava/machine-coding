# 27 · Circuit Breaker (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 60 min | Level 3 — infrastructure provided. Test utilities for threads/clocks exist; business/domain code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Your service calls a flaky downstream dependency. When it keeps failing, hammering it only makes things worse and makes
your own callers wait for doomed calls. Build a guard that wraps an arbitrary operation: while the dependency looks
healthy calls go through; after repeated failures calls are refused instantly for a cool-down period; afterwards the guard
carefully probes the dependency before trusting it again.

## Requirements

1. The guard is created with a **failure threshold** `N`, a **cool-down** in milliseconds and a number of **probe successes required** `K` (all at least 1), plus a clock.
2. The caller hands an operation to the guard. The operation reports success or failure. The guard runs it (or refuses to) and tells the caller what happened.
3. Normally calls run straight through. `N` **consecutive** failures make the guard trip; any success in between resets the failure count.
4. While tripped, every call is refused **without running the operation**.
5. Once the cool-down has elapsed (the elapsed time is *at least* the cool-down), the guard is ready to probe: it lets **one** call at a time through as a trial; every other call arriving while a trial is running is refused.
6. A failed trial trips the guard again and the cool-down starts over from that moment. After `K` **consecutive** successful trials the guard returns to normal operation with a fresh failure count.
7. The current state can be queried at any time and reflects the current time, even if nobody has called anything since the cool-down ended.
8. Many threads call the guard concurrently, including while the operation is running (operations may be slow).

## Constraints

- Thread-safe. The guard must never run the operation while holding a lock that blocks other callers or state queries: a slow trial must not freeze the others (they get refused immediately).
- Time comes from the clock given to the factory; do not read the system clock directly.
- Refused calls have no effect on any counter or timer.

## Examples

```
threshold 3, cool-down 1000 ms, 2 trial successes required
fail, fail, fail            → tripped (3 consecutive failures)
call at +500 ms             → refused, operation not run
call at +1000 ms            → trial runs, succeeds → still probing (1 of 2)
call                        → trial runs, succeeds → back to normal
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `call(op)` runs `op` and returns `"SUCCESS"` if it reported success, `"FAILURE"` if it reported failure, or `"REJECTED"` if the guard refused the call without running it.
- In Java an operation that throws counts as a failure (the exception does not escape; `call` returns `"FAILURE"`). In Rust the operation is a plain closure returning `bool`; panics are out of scope.
- `state()` returns `"CLOSED"` (normal operation), `"OPEN"` (tripped, cooling down) or `"HALF_OPEN"` (cool-down elapsed, probing), computed at the current time. `HALF_OPEN` stays in effect until trials decide the outcome, also while a trial is running.
- A failure that trips the guard happens when the `N`-th consecutive failure *completes*; the cool-down starts at that time. A trial failure restarts the cool-down at the time the failing trial completes.
- A call that was admitted in one state but finishes after the state has changed does not influence the new state.
- `failureThreshold`, `openMillis` and `halfOpenSuccessesRequired` are at least 1 (behaviour for other values is unspecified).

## Test entry point (Java)

The tests drive your system through the thin, primitive-typed contract below. It is **not** your domain
model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in
`solution/Solution.java`.

```java
package contract;

import java.util.function.BooleanSupplier;

/** Thin test entry point. Your domain model lives behind it. Must be thread-safe. */
public interface CircuitBreakerApi {

    /**
     * Runs the operation unless refused. The operation returns true for success; throwing counts as failure.
     * Returns "SUCCESS", "FAILURE" or "REJECTED" (refused, operation not run).
     */
    String call(BooleanSupplier operation);

    /** "CLOSED", "OPEN" or "HALF_OPEN", at the current time. */
    String state();
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    CircuitBreakerApi create(int failureThreshold, long openMillis, int halfOpenSuccessesRequired, Clock clock)
}
```

## Language notes

`Clock` is `java.time.Clock`. `call` and `state` are called from many threads, and `call` may be slow.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/27-circuit-breaker test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.
