# 27 · Circuit Breaker (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 60 min | Level 3 — infrastructure provided. Test utilities for threads/clocks exist; business/domain code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Your service calls a flaky downstream dependency (a payment provider, say). When it keeps failing, hammering it only makes things worse and makes your own callers wait for doomed calls. You are building a **circuit breaker**: a guard that wraps an arbitrary operation. While the dependency looks healthy, calls go straight through. After repeated failures the guard "trips" and refuses calls instantly (without running them) for a cool-down period. When the cool-down is over, the guard carefully probes the dependency with single trial calls before trusting it again.

The guard has three states, reported as text: `CLOSED` (normal), `OPEN` (tripped, cooling down), `HALF_OPEN` (cool-down over, probing with trial calls).

**Terms**

- **Failure threshold `N`**: how many *consecutive* failures trip the guard.
- **Cool-down**: how many milliseconds the guard stays `OPEN` after tripping.
- **Trial**: a call let through while `HALF_OPEN`; only one trial may run at a time.
- **Probe successes required `K`**: how many consecutive successful trials close the guard again.
- **Refused / `REJECTED`**: the guard did not run the operation at all.

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

- Thread-safe. A slow trial must not freeze the others: other callers are refused immediately and state queries still answer.
- Time comes from the clock given to the factory; do not read the system clock directly.
- Refused calls have no effect on any counter or timer.

## Examples

All with `N = 3`, cool-down `1000` ms, `K = 2`, clock at `10000`. `fail` / `ok` are operations that report failure / success.

```
state()          -> CLOSED
call(ok)         -> SUCCESS
call(fail)       -> FAILURE     call(fail) -> FAILURE     state() -> CLOSED   (2 failures, below N)
call(ok)         -> SUCCESS     (the success resets the failure count to 0)
call(fail) x3    -> FAILURE x3  state() -> OPEN             (3rd consecutive failure trips it; cool-down starts now, at 10000)
call(ok)         -> REJECTED    (operation not run, and the refusal changes nothing)
clock 10999:  state() -> OPEN        call(ok) -> REJECTED
clock 11000:  state() -> HALF_OPEN   (no call needed to notice; elapsed time 1000 >= cool-down)
call(ok)         -> SUCCESS     state() -> HALF_OPEN   (trial 1 of 2)
call(ok)         -> SUCCESS     state() -> CLOSED      (trial 2 of 2; failure count is fresh: it takes 3 new failures to trip again)
```

A failed trial:

```
tripped at 10000; clock 11000 (HALF_OPEN)
call(ok) -> SUCCESS (1 of 2)      call(fail) -> FAILURE      state() -> OPEN   (cool-down restarts at 11000)
clock 11999: OPEN                 clock 12000: HALF_OPEN
the earlier trial success is forgotten: two new successful trials are needed to close
```

One trial at a time:

```
HALF_OPEN, 20 threads call at once with a slow operation:
exactly 1 operation runs (and returns SUCCESS); the other 19 calls return REJECTED immediately; state() stays HALF_OPEN
while that trial is still running, any further call -> REJECTED; once it finishes, the next call may become the next trial
```

## Edge cases

- While `CLOSED`, many calls may run in parallel; only `HALF_OPEN` limits it to one at a time.
- 100 threads failing at once while `CLOSED`: the guard trips once the 3rd failure completes; calls arriving after that are `REJECTED`, so the number of `FAILURE` results is at least 3 and equals the number of operations that actually ran.
- The cool-down is measured from the moment the tripping failure *completes*, not from when its call started.
- A call admitted before a state change but finishing after it (for example a slow call that began while `CLOSED` and ends after the guard tripped) does not change the new state.
- `HALF_OPEN` also covers the time a trial is running: `state()` reports `HALF_OPEN` throughout.
- Java: an operation that throws counts as a failure; `call` returns `FAILURE` and the exception does not escape. Rust: operations are plain closures returning `bool`; panics are out of scope.

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
