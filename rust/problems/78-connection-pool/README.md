# 78 · Bounded Connection Pool (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 60 min | Level 3 — infrastructure provided. Test utilities for threads/clocks exist; business/domain code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

A web server has hundreds of request threads but only a handful of database connections, and opening a connection is expensive. You are building the **connection pool** that sits between them: a fixed set of `size` connections (numbered `0..size-1`) that threads borrow one at a time, use alone, and hand back. When all connections are borrowed, a thread waits, but only up to a timeout it chooses, after which it gives up. When the application shuts down, the pool is closed and every thread still waiting is told to stop waiting. The pool never invents or loses a connection: at any moment each connection is either free or held by exactly one thread.

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

### Worked traces

**Run dry, then recover** (`size 3`)
```
acquire(0) -> Ok(Some(0)), Ok(Some(1)), Ok(Some(2))   (any order of ids; each id exactly once)
acquire(0)            -> Ok(None)   (returns at once, not an error)
available() = 0, in_use() = 3
release(1)            -> Ok(()); available() = 1, in_use() = 2
acquire(0)            -> Ok(Some(1))
```

**Timeout** (`size 1`)
```
a = acquire(0)        -> Ok(Some(0))
acquire(200)          -> waits about 200 ms, then Ok(None)
available() = 0, in_use() = 1       (the timed-out call took nothing and lost nothing)
```

**Bad releases** (`size 2`)
```
a = acquire(0); release(a)          -> Ok(())
release(a)                          -> Err(ReleaseError::NotAcquired)      (double release)
release(1 - a)                      -> Err(ReleaseError::NotAcquired)      (never acquired)
release(5)                          -> Err(ReleaseError::UnknownConnection) (id outside 0..size)
available() = 2 afterwards
```

**Close** (`size 1`)
```
id = acquire(0)
thread T: acquire(10_000)           -> blocks
close(); close()                    -> second close is a no-op
T                                   -> Err(Closed) promptly (does not wait out its 10 s)
acquire(0)                          -> Err(Closed), not Ok(None)
release(id)                         -> Ok(()); in_use() = 0
```

### Edge cases

- `timeoutMs <= 0` (Rust: `0`) on a full pool returns *empty* immediately; on a **closed** pool it fails instead.
- A timeout is a normal outcome (empty / `None`), not an error. Only a closed pool makes `acquire` fail.
- When several connections are free, which id you hand out first is up to you; callers must only get an id that is free right now.
- When one connection is released and several threads wait, exactly one of them receives it.
- A waiter whose timeout expires at the same moment a connection is released either gets that connection or leaves it free; it never makes it disappear or hand it out twice.
- `close()` does not change `available()` / `inUse()`; connections still held are returned later through `release`.
- `release` after `close` succeeds for a held connection; the usual errors (double release, bad id) still apply.

## Glossary

- **Connection**: here just a number `0..size-1`; the id *is* the resource.
- **Free / available**: not currently held by anyone, so `acquire` may hand it out.
- **Acquired / in use**: handed out and not yet released.
- **Holder**: the thread that acquired a connection and has not released it yet.
- **Waiter**: a thread blocked inside `acquire` because nothing is free.
- **Closed**: after `close()`; no further acquires succeed.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `Optional<Integer> acquire(long timeoutMs) throws InterruptedException`; `IllegalStateException` for a closed pool and for a bad release; `IllegalArgumentException` for an id out of range. Rust: `Result<Option<usize>, Closed>` and `Result<(), ReleaseError>`.
- A timeout must wait roughly the whole timeout (tests allow generous tolerance) and not much longer.
- The pool does not create real connections; the id *is* the resource. (A generic `Pool<T>` is a follow-up.)

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! A pool of `size` reusable "connections", identified by the numbers `0..size`.

/// The pool is (or became) closed.
#[derive(Debug, PartialEq, Eq)]
pub struct Closed;

#[derive(Debug, PartialEq, Eq)]
pub enum ReleaseError {
    /// The id is outside `0..size`.
    UnknownConnection,
    /// That connection is not currently acquired (double release).
    NotAcquired,
}

pub trait Pool: Send + Sync {
    /// Waits up to `timeout_ms` for a free connection and returns its id (`Ok(Some(id))`), or `Ok(None)` if none
    /// became free in time. `0` means do not wait. `Err(Closed)` if the pool is (or becomes) closed.
    fn acquire(&self, timeout_ms: u64) -> Result<Option<usize>, Closed>;

    /// Gives a connection back. Allowed after `close`.
    fn release(&self, id: usize) -> Result<(), ReleaseError>;

    /// Connections that could be acquired right now.
    fn available(&self) -> usize;

    /// Connections currently acquired. `available() + in_use() == size` at all times.
    fn in_use(&self) -> usize;

    /// Rejects further acquires and wakes every waiting thread (they get `Err(Closed)`). Idempotent.
    fn close(&self);
}
```

Your wiring point (`src/lib.rs`):

```rust
/// `size` is at least 1.
pub fn create(_size: usize) -> Option<Arc<dyn Pool>> { ... }
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

## Running

```bash
cargo test -p p78_connection_pool
STRICT=1 cargo test -p p78_connection_pool   # fail instead of skip while unimplemented
cargo test -p p78_connection_pool --features hidden   # interviewer scenarios
cargo play run 78              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`run_concurrently`, `eventually`, `stays`, `watchdog`, `sut!`): `rust/support`. `run_concurrently` aborts the test run with a `TIMEOUT` message after 60 s, so a deadlock fails instead of freezing `cargo test`.
