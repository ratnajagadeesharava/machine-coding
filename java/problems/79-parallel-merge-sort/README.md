# 79 · Parallel Merge Sort (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 60 min | Level 3 — infrastructure provided. Test utilities for threads/clocks exist; business/domain code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

A batch job must sort an array of millions of 32-bit integers, and the machine has several cores sitting idle. You are building a sorter that sorts the array **in place** (the caller's array ends up sorted, no new array is returned) by dividing the work between at most `parallelism` threads and combining the sorted pieces. The caller picks `parallelism`; the sorter must respect it as an upper bound, still work for tiny or odd inputs, and be safe to call from several threads at once as long as each call has its own array.

## Problem

Sort a large `int` array in place using several threads. Split the work recursively, sort the pieces in
parallel, merge them. The result must be correct for any input and any `parallelism`, and the thread count must stay bounded.

## Requirements

1. `sort(data, parallelism)` sorts `data` ascending, in place.
2. At most `parallelism` worker threads take part (the calling thread may be one of them). `parallelism == 1` must not need any additional thread.
3. Any length (0, 1, 2, ...), any `int` values including `MIN_VALUE` / `MAX_VALUE`, duplicates and already-sorted / reversed input are valid.
4. `parallelism < 1` is rejected.
5. The sorter keeps no per-call state in fields: it may be used from several threads on different arrays at once.

## Constraints and thread-safety requirements

- The pieces handled by different threads must be disjoint.
- The merge of two halves may only start after both halves are completely sorted *and visible* to the merging thread.
- The thread count is bounded by `parallelism`.
- Scale: 1-3 million elements in a few seconds; the hidden tests use millions.

## Examples

```
[5 2 9 1 7 3 8 4], parallelism 2
  left  half [5 2 9 1] -> thread A -> [1 2 5 9]
  right half [7 3 8 4] -> thread B -> [3 4 7 8]
  merge -> [1 2 3 4 5 7 8 9]
```

### Worked traces

```
sort([5, 2, 9, 1, 7, 3, 8, 4], 1)   -> [1, 2, 3, 4, 5, 7, 8, 9]    (no extra thread needed)
sort([], 4)                          -> []            sort([42], 8) -> [42]
sort([3, 1, 2], 8)                   -> [1, 2, 3]     (more threads allowed than elements)
sort([MAX_VALUE, MIN_VALUE, 0, -1], 4) -> [MIN_VALUE, -1, 0, MAX_VALUE]
sort([2, 2, 1, 2], 2)                -> [1, 2, 2, 2]  (duplicates all kept)
sort([2, 1], 0) and sort([2, 1], -3) -> IllegalArgumentException
```

### Edge cases

- Length 0 and 1 are valid and return immediately with the data unchanged.
- `parallelism` larger than the array length is fine; it is only an upper bound.
- Extreme values (`MIN_VALUE`, `MAX_VALUE`) must order correctly; do not let comparing two values overflow.
- The result is the same multiset of values as the input, ascending (non-decreasing). Nothing is added, dropped or duplicated.
- When `sort` returns, all work is finished; no helper keeps touching the array afterwards.
- Different threads may sort different arrays through the same sorter at the same time; the calls must not affect each other.
- The thread limits in the contract count extra threads alive during a big sort (about `parallelism`, plus 1 of slack). A solution that stays on the calling thread fails the "really runs in parallel" check for `parallelism = 4` on a large array.

## Glossary

- **In place**: the sorted result is written back into the array that was passed in.
- **Parallelism**: the maximum number of worker threads that may be doing sorting work at once.
- **Merge**: combining two already-sorted neighbouring ranges into one sorted range.
- **Range / piece**: a contiguous part of the array (from index `lo` up to but not including `hi`) given to one worker.
- **Helper thread**: a thread other than the one that called `sort`.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `ParallelSorter.sort(int[] data, int parallelism)` throws `IllegalArgumentException` for `parallelism < 1`. Rust: `sort(&self, &mut [i32], usize) -> Result<(), InvalidParallelism>`.
- The tests count live threads while a big sort runs: with `parallelism = 4` you may use at most 4 extra threads (+1 slack); with `1` at most 1.
- The tests also expect *some* helper thread to appear for `parallelism = 4` on a large array (a purely sequential sort fails `shouldReallyRunInParallel`).
- No speed-up is asserted (CI machines may have one core); the hidden tests only require 'not much slower than sequential'.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

/** Thin test entry point. Your design lives behind it. May be called from several threads on different arrays. */
public interface ParallelSorter {

    /**
     * Sorts {@code data} in place, ascending, splitting the work over at most {@code parallelism} worker threads
     * (the caller may take part). {@code parallelism == 1} must not need any extra thread.
     * IllegalArgumentException if parallelism < 1. Any length (including 0 and 1) and any int values are valid.
     */
    void sort(int[] data, int parallelism);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static ParallelSorter create()
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Concurrency tests run many threads against one object, use deadlock timeouts (a hang becomes a failure) and repeat a workload 100 times to catch rare interleavings.

- Tiny inputs (0, 1, 2, 3 elements) at parallelism 1, 2 and 8.
- Sorted, reversed, all-equal and few-distinct arrays of 50,000 elements.
- `MIN_VALUE` / `MAX_VALUE` extremes (catches `a - b` comparators).
- Random arrays of up to 1,000,000 elements (400,000 in Rust) at parallelism 1, 2, 4, 8, compared with the library sort.
- More threads than elements.
- Six threads sorting six different arrays through the same sorter at the same time.
- Thread-count sampling during a multi-million-element sort (Rust: reads `/proc/self/status`, Linux only), both an upper bound and a 'something ran in parallel' lower bound.
- **Repeat 100x** with random sizes and parallelism.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/79-parallel-merge-sort test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`Concurrently`, `Eventually`, `Candidate`): `java/testkit`.
