# 97 · Idempotent Request Processor (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 60 min | Level 3 — infrastructure provided: a deterministic network and clock simulation (`SimNet`) runs your nodes; the protocol code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Our payments API is called by mobile clients on flaky networks, so clients retry. A retry must never charge twice: each request
carries an idempotency key, and the same key must produce the effect once and the same response every time. Build the component that
wraps a business operation with that guarantee. Retries often arrive while the first attempt is still running, a client
sometimes reuses a key by mistake for a different request, handlers sometimes fail, and the service runs for months,
so the bookkeeping must not grow forever.

## Requirements

1. `process(key, body, handler)` runs `handler(body)` at most once per idempotency key and returns its response. Every other call with the same key and the same body returns the stored response without running the handler.
2. Same key with a **different** body is rejected with `KeyConflictException` (Rust: `Err(KeyConflict)`) - also while the first call is still running - and the handler does not run.
3. Concurrent calls with the same key and body: exactly one runs the handler; the others wait for it and return the same response. Calls with different keys never wait for each other.
4. If the handler throws (Rust: returns `Err`), nothing is stored. The failure reaches the caller that ran it, and the key is free again. Callers that were waiting do not fail: one of them becomes the next to run the handler.
5. A stored response expires 24 hours after it was stored; afterwards the key can be used again, even with a different body.
6. A null or blank key, or a null body, is rejected with `IllegalArgumentException` (Rust: `Err(InvalidInput)` for a blank key).
7. Entries that expired must be reclaimed eventually; memory must not grow with the number of keys ever seen.

## Simulation rules and constraints

- The processor is shared by many request threads; handlers may take long and may sleep. Treat them as black boxes that can block.
- Never hold a processor-wide lock while a handler runs: 16 different keys with 150 ms handlers must finish in about 150 ms, not 16 x 150 ms.
- Waiters of the same key share the one execution: 32 threads with a 100 ms handler finish in about 100 ms and the handler ran once.
- Time comes from the injected clock (`FakeClock`); the tests move it by hand, handler `sleep`s are real but only used to keep calls overlapping.
- No network simulation is involved: this is a thread-safety and exactly-once problem on one process. The distributed angle (what survives a crash) belongs to the follow-ups.

## Examples

```
process("k1", "pay 10", h)   -> runs h, stores "done:pay 10:1"
process("k1", "pay 10", h)   -> "done:pay 10:1"    (handler not run again)
process("k1", "pay 99", h)   -> KeyConflictException
h throws once:  process("k", "x", h) -> throws; process("k", "x", h) -> runs again, stores result
32 threads, same key and body, handler takes 100 ms -> handler runs once, all get the same string
24h - 1ms later: stored; at +24h: expired, the handler runs again
```

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `IdempotentProcessor.process(String, String, Handler) -> String`, `Handler` is `String handle(String body)` and signals failure with any unchecked exception, factory `Solution.create(Clock)`. Rust: `process(&self, key, body, handler: HandlerFn) -> Result<String, IdemError>`, handler is `&(dyn Fn(&str) -> Result<String, String> + Sync)`, `create(Arc<dyn Clock>)`.
- Rust errors: `KeyConflict`, `InvalidInput` (blank key), `HandlerFailed(String)` carrying the handler's own error for the caller that ran it.
- Java: the exact exception thrown by the handler propagates unchanged to the caller that executed it (the test checks type and message).
- A conflict is decided by comparing the body with the one stored or currently in flight for that key; replays compare equal bodies by value.
- Expiry: the response is valid for `now < storedAt + 24h`. One millisecond before 24 h it is replayed; at exactly 24 h the handler runs again. The expiry is measured from when the response was stored, not from the last replay.
- After expiry a different body is accepted for the key.
- When the running handler fails and N callers are waiting, exactly one waiter runs the handler next; the others then share its outcome. In the test 8 threads produce 1 failure, 7 successes and 2 handler invocations.
- The stored response is the string returned by the handler, replayed byte for byte.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Time comes from the clock handed to `create`.

/// The business operation behind an idempotency key (provided by the tests). It fails by returning `Err`.
pub type HandlerFn<'a> = &'a (dyn Fn(&str) -> Result<String, String> + Sync);

#[derive(Debug, PartialEq, Eq)]
pub enum IdemError {
    /// the same key was reused for a different request body
    KeyConflict,
    /// blank key
    InvalidInput,
    /// the handler failed (nothing was stored); carries its error
    HandlerFailed(String),
}

pub trait IdempotentProcessor: Send + Sync {
    /// Runs `handler` for the request at most once per idempotency key and returns its response; every other call
    /// with the same key and the same body returns the stored response without running the handler.
    ///
    /// * Same key, DIFFERENT body (also while the first call is still running): `Err(KeyConflict)`.
    /// * Concurrent calls with the same key and body: one runs the handler, the others wait for it and return the same
    ///   response. Calls with different keys never wait for each other.
    /// * If the handler fails, nothing is stored: the error reaches the caller that ran it (`HandlerFailed`), and the
    ///   key is free again. Callers that were waiting for it do not fail; one of them runs the handler next.
    /// * A stored response expires 24 hours after it was stored (then the key can be used again).
    fn process(
        &self,
        idempotency_key: &str,
        request_body: &str,
        handler: HandlerFn<'_>,
    ) -> Result<String, IdemError>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(_clock: Arc<dyn Clock>) -> Option<Arc<dyn IdempotentProcessor>> { ... }
```

## What the tests check

These tests are black-box and deterministic: every node runs inside a seeded simulation (one tick = 1 ms of simulated time, nothing sleeps), messages can be delayed, dropped, duplicated and reordered, nodes can crash and recover, the network can be partitioned, and safety invariants are checked after EVERY simulated millisecond. A failing seed reproduces exactly.

- Run once and replay; different keys independent; conflict on a different body leaves the stored response replayable.
- Invalid arguments: null key, blank key, null body.
- A failing handler stores nothing and the next call runs it again; expiry at 24 h - 1 ms vs 24 h; a new body allowed after expiry.
- 32 racing threads on one key: handler once, one response, finishes quickly; 16 distinct keys run in parallel.
- Hand-over: with 8 threads and a first handler that fails after 100 ms, exactly one thread sees the failure, seven succeed, two invocations in total.
- A conflicting body is rejected while the first call is still blocked inside its handler.
- **Repeat 100x**: 8 racing threads, exactly one handler call.
- More edge cases and scale scenarios in `interviewer/` (see `INTERVIEWER_NOTES.md`).

## Language notes

A `Mutex<HashMap<String, Arc<Entry>>>` held only for lookup/insert, plus a `Mutex`+`Condvar` inside each entry for waiting, avoids holding the map lock during a handler. Use `Result` to carry `HandlerFailed` out of the executing thread and drop every guard before calling the handler.

## Running

```bash
cargo test -p p97_idempotent_requests
STRICT=1 cargo test -p p97_idempotent_requests   # fail instead of skip while unimplemented
cargo test -p p97_idempotent_requests --features hidden   # interviewer scenarios
cargo play run 97              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`SimNet`, `Rng`, `sut!`): `rust/support`.
