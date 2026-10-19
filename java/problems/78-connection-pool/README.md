# 78 · Bounded Connection Pool (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 60 min | Level 3 — infrastructure provided. Test utilities for threads/clocks exist; business/domain code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Build a pool of `size` reusable resources ("connections") shared by many threads. A thread asks for a connection
and may have to wait; it uses it exclusively and gives it back. Threads must be able to give up after a timeout without
losing or duplicating a connection, a connection must never be handed to two holders at once, and shutting the pool down
must wake everybody who is waiting.

## Requirements

1. The pool owns `size` connections identified by `0..size-1`.
2. `acquire(timeoutMs)` returns a free connection's id, waiting up to `timeoutMs` for one. After the timeout it returns *nothing* (empty / `None`). `timeoutMs <= 0` means do not wait.
3. `release(id)` returns a connection to the pool and wakes a waiter. Releasing a connection that is not currently acquired (double release, never acquired) is an error; an id outside `0..size-1` is a different error.
4. `available()` and `inUse()` report the current counts; `available() + inUse() == size` always.
5. `close()` rejects further `acquire`s and wakes every waiting thread, which fails. Releasing after `close` is still allowed. `close` is idempotent.

## Constraints and thread-safety requirements

- Any number of threads acquire/release concurrently; far more threads than connections.
- A connection is held by at most one thread at a time.
- Waiting threads must not burn CPU and must be woken promptly when a connection is released (not only when their timeout fires).
- A waiter that times out at the very moment a connection is released must not swallow it (no lost or duplicated connections).
- Throughput: 32 threads x 1,600 acquire/release cycles on a 5-connection pool in a few seconds.

## Examples

```
size 2:   a = acquire(0) -> 0      b = acquire(0) -> 1      acquire(0) -> empty
          other thread: acquire(10_000) -> waits
          release(a)             -> the waiting thread gets connection 0
close()                          -> everyone still waiting fails; release(b) still works
```

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `Optional<Integer> acquire(long timeoutMs) throws InterruptedException`; `IllegalStateException` for a closed pool and for a bad release; `IllegalArgumentException` for an id out of range. Rust: `Result<Option<usize>, Closed>` and `Result<(), ReleaseError>`.
- A timeout must wait roughly the whole timeout (tests allow generous tolerance) and not much longer.
- The pool does not create real connections; the id *is* the resource. (A generic `Pool<T>` is a follow-up.)

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.Optional;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>A pool of {@code size} reusable "connections", identified by the numbers 0..size-1.
 */
public interface Pool {

    /**
     * Waits up to {@code timeoutMs} for a free connection and returns its id, or empty if none became free in time.
     * {@code timeoutMs <= 0} means do not wait. Throws IllegalStateException if the pool is (or becomes) closed.
     */
    Optional<Integer> acquire(long timeoutMs) throws InterruptedException;

    /**
     * Gives a connection back. IllegalArgumentException if id is outside [0, size);
     * IllegalStateException if that connection is not currently acquired (double release). Allowed after close.
     */
    void release(int id);

    /** Connections that could be acquired right now. */
    int available();

    /** Connections currently acquired. {@code available() + inUse() == size} at all times. */
    int inUse();

    /** Rejects further acquires and wakes every waiting thread (they throw IllegalStateException). Idempotent. */
    void close();
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    /**
 * Wiring point between the test contract and YOUR design.
 * Create your own classes anywhere under src/main/java and return them from here.
 */
public final class Solution {
    private Solution() {}

    /** {@code size} is at least 1. */
    public static Pool create(int size)
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Concurrency tests run many threads against one object, use deadlock timeouts (a hang becomes a failure) and repeat a workload 100 times to catch rare interleavings.

- Each id handed out exactly once until the pool runs dry; reuse after release; counts are consistent.
- A 200 ms timeout waits 180-1500 ms and returns nothing.
- A waiter with a 10 s timeout wakes within 2 s of a release (no polling loop with long sleeps).
- Double and out-of-range releases are rejected.
- 16 threads x 500 cycles on 4 connections: no id is ever held twice, at most 4 holders, nothing leaks.
- 20 waiters time out while the pool is empty, then all connections are still there; 12 threads with 1 ms timeouts racing releases leave exactly `size` available.
- `close` wakes a blocked waiter with a failure; later `acquire`s fail; `release` still works.
- **Repeat 100x** with 1-4 connections, mixed tiny and long timeouts.

## Language notes

`ReentrantLock` + `Condition.awaitNanos(remaining)` returns the remaining time for you. `Semaphore.tryAcquire(timeout)` is fine for the permit but cannot tell you which id to hand out.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/78-connection-pool test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`Concurrently`, `Eventually`, `Candidate`): `java/testkit`.
