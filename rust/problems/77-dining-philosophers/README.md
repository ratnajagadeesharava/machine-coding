# 77 · Dining Philosophers (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 60 min | Level 3 — infrastructure provided. Test utilities for threads/clocks exist; business/domain code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Five (or N) philosophers sit around a table with one fork between each pair of neighbours. To eat, a philosopher
needs **both** adjacent forks; then puts them down and thinks again. Build the table so that nobody ever deadlocks,
nobody starves, neighbours never eat at the same time, and philosophers who are *not* neighbours can eat in parallel.
It must work for any number of seats, including 2 (where both forks are shared) and 100.

## Requirements

1. Seats are numbered `0..seats-1` in a circle; the fork between seat `p` and seat `(p + 1) % seats` is shared by exactly those two philosophers.
2. `dine(p, eat)` blocks until philosopher `p` holds both its forks, runs `eat` on the calling thread, then releases the forks.
3. Two neighbouring philosophers never eat at the same time.
4. Non-neighbours may eat concurrently - a single global lock is not acceptable.
5. Forks are released even if `eat` throws / panics.
6. Unknown philosopher numbers are rejected.

## Constraints and thread-safety requirements

- One thread per philosopher, all repeatedly calling `dine` (thousands of meals each).
- Deadlock freedom: the classic 'everybody takes the left fork first' must not be able to freeze the table.
- No starvation: every philosopher completes all of its meals even when neighbours are greedy.
- Works for 2..100 seats.

## Examples

```
5 seats. Philosopher 0 eating -> 1 and 4 must wait; 2 and 3 may eat (but not both: they are neighbours).
At most floor(5 / 2) = 2 philosophers eat at once, e.g. {0, 2} or {1, 3}.

everyone calls dine() at the same instant, takes their left fork ...  <- the textbook deadlock
```

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java `dine` throws `InterruptedException`; `IllegalArgumentException` for a bad seat. Rust: `Err(UnknownPhilosopher)`.
- The tests measure 'who is eating' from inside `eat`; the table must not run `eat` before both forks are held or after one is released.
- How you represent forks (locks, semaphores, a state array + condition, a waiter/arbitrator) is entirely yours.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Philosophers sit in a circle, numbered `0..seats`; there is one fork between every two neighbours, so
//! philosopher `p` shares a fork with `(p + 1) % seats` and with `(p + seats - 1) % seats`.

/// The philosopher index was outside `0..seats`.
#[derive(Debug, PartialEq, Eq)]
pub struct UnknownPhilosopher;

pub trait Table: Send + Sync {
    /// Blocks until philosopher `p` holds both of its forks, runs `eat` on the calling thread, then puts the
    /// forks down (also when `eat` panics; the panic propagates). `Err(UnknownPhilosopher)` if `p >= seats`.
    fn dine(&self, philosopher: usize, eat: &mut dyn FnMut()) -> Result<(), UnknownPhilosopher>;
}
```

Your wiring point (`src/lib.rs`):

```rust
/// `seats` is at least 2.
pub fn create(_seats: usize) -> Option<Arc<dyn Table>> { ... }
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Concurrency tests run many threads against one object, use deadlock timeouts (a hang becomes a failure) and repeat a workload 100 times to catch rare interleavings.

- Neighbours never overlap (flags checked inside `eat`) for 3, 4, 5, 7 and 12 seats and for 2 seats.
- 5 philosophers x 2,000 back-to-back meals finish within a deadlock timeout.
- With a 10 ms meal, at least 2 (and at most 2) philosophers are seen eating at once for 5 seats: a global lock fails, a broken exclusion fails.
- A throwing `eat` releases its forks.
- **Repeat 100x** with 2-6 seats.

## Language notes

`Vec<Mutex<()>>` for forks. If `eat` panics while the guards are alive the mutexes get poisoned: decide how you handle `PoisonError` so the table survives.

## Running

```bash
cargo test -p p77_dining_philosophers
STRICT=1 cargo test -p p77_dining_philosophers   # fail instead of skip while unimplemented
cargo test -p p77_dining_philosophers --features hidden   # interviewer scenarios
cargo play run 77              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`run_concurrently`, `eventually`, `stays`, `watchdog`, `sut!`): `rust/support`. `run_concurrently` aborts the test run with a `TIMEOUT` message after 60 s, so a deadlock fails instead of freezing `cargo test`.
