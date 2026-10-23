# 76 · Ordered Printer (Turn Taking) (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 3 — infrastructure provided. Test utilities for threads/clocks exist; business/domain code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

N worker threads each need to do a piece of work over and over, but the work of different workers must
interleave in strict round-robin order: worker 0, worker 1, ..., worker N-1, worker 0, ... (think of N threads that
print "ping", "pong", ... or the three threads of the classic print-in-order / FizzBuzz exercise). Build the turn-taking
primitive they all share. Each worker thread calls it in a loop; the primitive decides who may run next.

### What you are building

A turn-taking gate shared by N worker threads, like the "print-in-order" or FizzBuzz threading exercises. Each worker
thread repeatedly calls `takeTurn` with a piece of work. The gate lets only one worker run at a time and enforces a
strict rotation: worker 0, then 1, then 2, ... then N-1, then back to 0. A worker that shows up too early simply waits
for its turn, however many times and in whatever order the threads were started.

### Glossary

- **Turn**: the right to run one action. Exactly one worker holds it at a time; it starts with worker 0.
- **Hand over**: after a worker's action finishes, the turn moves to worker `(worker + 1) % workers`.
- **Out of turn**: calling `takeTurn` while another worker holds the turn.

## Requirements

1. Workers are numbered `0..workers-1`. Worker 0 goes first.
2. `takeTurn(worker, action)` blocks until it is that worker's turn, runs `action` **on the calling thread**, then hands the turn to worker `(worker + 1) % workers`.
3. Exactly one action runs at any time. What one action wrote must be visible to the action of the next turn (the tests log into a plain, non-thread-safe list on purpose).
4. If `action` throws, the turn is still handed over and the exception reaches the caller.
5. A worker that calls out of turn simply waits; it never skips ahead and never makes another worker skip.
6. An unknown worker number is rejected without affecting the others.
7. `takeTurn` returns only after the action has run (and the turn was handed over). If the action throws, the same exception reaches the caller of `takeTurn` and the next worker still gets its turn.
8. With one worker (`workers == 1`) the turn always passes back to worker 0, so every call runs immediately. An out-of-range worker (`< 0` or `>= workers`) throws `IllegalArgumentException` at once, without waiting and without changing whose turn it is.

## Constraints and thread-safety requirements

- Any number of workers (1 to ~100), one thread per worker, all calling concurrently.
- Waiting must not burn CPU: no spinning, no sleep-polling loops.
- The hand-over must reach the right worker promptly (the hidden tests pass the turn 20,000 times).

## Examples

```
3 workers.  Threads start in any order; the log of actions is always
0 1 2 0 1 2 0 1 2 ...

worker 1 calls takeTurn first        -> waits (worker 0 has not gone yet)
worker 0 calls takeTurn              -> runs, passes turn to 1
worker 1 (waiting)                   -> wakes up, runs, passes turn to 2
```

More worked examples:

```
4 workers, threads started in the order 3, 2, 1, 0 (each ~30 ms apart)
-> all of 3, 2, 1 block; once worker 0 calls takeTurn the actions run 0, 1, 2, 3.

2 workers, each calls takeTurn 3 times in a loop
-> log: 0 1 0 1 0 1

1 worker, 100 calls -> log: 0 repeated 100 times, no blocking.
```

## Edge cases

- A worker calling `takeTurn` for a second time before the others have had their turn waits again; it can never run twice in a row (unless there is only one worker).
- An action that throws (Rust: panics) still passes the turn on, so the ring is never stuck.
- A call with a worker number outside `0..workers` is rejected immediately and the other workers carry on unaffected.
- Actions write to a plain, non-thread-safe list; the next worker's action must see everything the previous action wrote.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- `takeTurn` may be interrupted while waiting (Java: `InterruptedException`). What you do with a thread that is interrupted while waiting is yours to decide; the tests never interrupt.
- The factory is called with `workers >= 1`. With one worker every call runs immediately.
- An out-of-range worker number fails fast: Java `IllegalArgumentException`, Rust `Err(UnknownWorker)`.
- Rust: the action is `&mut dyn FnMut()`. A panic in the action must still pass the turn on and then propagate.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface OrderedPrinter {

    /**
     * Blocks until it is {@code worker}'s turn, runs {@code action} on the calling thread, then hands the turn to
     * worker {@code (worker + 1) % workers}. The turn is handed over even if the action throws (the exception
     * propagates to the caller). IllegalArgumentException if worker is outside [0, workers).
     */
    void takeTurn(int worker, Runnable action) throws InterruptedException;
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

    /** Worker 0 goes first. {@code workers} is at least 1. */
    public static OrderedPrinter create(int workers)
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Concurrency tests run many threads against one object, use deadlock timeouts (a hang becomes a failure) and repeat a workload 100 times to catch rare interleavings.

- 3, 4, 6, 16 workers run 20-100 rounds each: the log must be exactly `0,1,...,n-1` repeated.
- A worker that arrives out of turn really blocks (checked for 150 ms), then runs after its predecessor.
- Workers that arrive in reverse order still run in order.
- A counter inside the actions proves no two actions overlap.
- A throwing / panicking action does not wedge the ring.
- **Repeat 100x** with 2-7 workers to flush out lost wake-ups.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/76-ordered-printer test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`Concurrently`, `Eventually`, `Candidate`): `java/testkit`.
