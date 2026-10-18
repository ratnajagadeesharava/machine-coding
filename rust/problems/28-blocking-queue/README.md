# 28 · Bounded Blocking Buffer (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 3 — infrastructure provided. Test utilities for threads/clocks exist; business/domain code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Producers and consumers run on different threads and run at different speeds. Build the hand-off point between them: a
bounded first-in-first-out buffer. Producers must slow down (wait) when the buffer is full, consumers must wait when it
is empty, and when the producing side is finished there must be a clean way to tell everybody to stop without losing
anything that was already accepted.

## Requirements

1. The buffer has a fixed **capacity** (at least 1) and holds items in arrival order.
2. `put` adds an item and **waits** while the buffer is full. `take` removes the oldest item and **waits** while the buffer is empty (and open).
3. `offer` and `poll` are the same operations with a **timeout** in milliseconds: they give up and report "nothing happened" when the time is up. A timeout of `0` (or less) means do not wait at all.
4. `size` reports how many items are currently stored (never more than the capacity).
5. `close` ends the life of the buffer. From then on nothing can be added: `put` and `offer` fail. Items accepted earlier are **not lost**: consumers keep receiving them in order, and only when the buffer is drained do `take`/`poll` report "no more items".
6. Closing wakes up everybody who is waiting: blocked producers fail (their item is **not** added), blocked consumers receive the remaining items and then "no more items".
7. Every accepted item is delivered to **exactly one** consumer, in FIFO order of acceptance.
8. Closing is idempotent.

## Constraints

- Thread-safe for any number of producers and consumers (multi-producer, multi-consumer).
- Waiting threads must not burn CPU (no busy loops) and must not be starved forever while progress is possible.
- Throughput matters: tens of thousands of items must pass through a small buffer within seconds.

## Examples

```
capacity 2:  put(1) put(2)          → buffer full
             put(3) on another thread → waits
             take() → 1             → the waiting put completes, buffer now holds 2, 3
close() while holding [2, 3]        → take() → 2, take() → 3, take() → "no more items"
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- Java: `put`, `take`, `offer` and `poll` may be interrupted while waiting and then throw `InterruptedException`.
- Java: `put`/`offer` on a closed buffer (also when the close happens while they wait) throw `IllegalStateException`. Rust: they return `Err(Closed)`. The item is not stored.
- `put`/`take`/`offer`/`poll` never fail for any other reason. `offer` returns `true` when the item was stored and `false` when the timeout expired first.
- `take`/`poll` return the oldest item, or empty/`None` when (a) `poll` timed out while the buffer was open and empty, or (b) the buffer is closed and empty. `take` only returns empty/`None` in case (b).
- A closed buffer still hands out its remaining items: `take`/`poll` on a closed non-empty buffer return the oldest item immediately.
- `size()` is the number of stored items; it never exceeds the capacity, also not transiently.
- `close` never blocks and may be called from any thread any number of times.
- Capacity is at least 1 (the factory may reject other values; this is not tested).
- Timeouts are lower bounds: an operation that times out must have waited roughly the whole timeout (tests allow generous tolerance) and must not wait much longer.

## Test entry point (Rust)

The tests drive your system through the thin, primitive-typed contract below (`src/contract.rs`). It is
**not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up
in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

/// Returned by `put`/`offer` when the buffer is closed. The item was not stored.
#[derive(Debug, PartialEq, Eq)]
pub struct Closed;

pub trait BlockingBuffer<T>: Send + Sync {
    /// Waits while full. `Err(Closed)` if the buffer is (or becomes) closed; the item is then not stored.
    fn put(&self, item: T) -> Result<(), Closed>;

    /// Waits while empty and open. `None` only when the buffer is closed and drained.
    fn take(&self) -> Option<T>;

    /// Like `put` but gives up after `timeout_ms` (`Ok(false)`). `0` means do not wait. `Err(Closed)` if closed.
    fn offer(&self, item: T, timeout_ms: u64) -> Result<bool, Closed>;

    /// Like `take` but gives up after `timeout_ms` (`None`). `0` means do not wait.
    fn poll(&self, timeout_ms: u64) -> Option<T>;

    /// Number of stored items.
    fn size(&self) -> usize;

    /// Rejects further puts, wakes all waiters, keeps remaining items for consumers. Idempotent, never blocks.
    fn close(&self);
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(_capacity: usize) -> Option<Arc<dyn BlockingBuffer<i32>>> {
    None
}
```

## Language notes

The trait is `&self` + `Send + Sync`. Items in the tests are `i32`. A `Mutex<VecDeque<T>>` plus two `Condvar`s is the classic starting point; `wait_timeout` returns early on spurious wake-ups, so loop on a deadline.

## Running

```bash
cargo test -p p28_blocking_queue
STRICT=1 cargo test -p p28_blocking_queue   # fail instead of skip while unimplemented
cargo test -p p28_blocking_queue --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p28_blocking_queue --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.
