# 113 · Multi-Module Logging Framework (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

Three application modules (`AUTH`, `PAYMENTS`, `SEARCH`) share one logging framework. Each module logs activity at a level (`DEBUG`, `INFO`, `WARN`, `ERROR`) to one of two destinations: a log file (`FILE`) or a monitoring tool (`TOOL`). Many threads log at the same time, so every accepted log gets a sequence number that is unique, increasing and gap-free. Accepted logs pass through one shared, bounded buffer and are streamed to their destination. A caller is told its sequence number only after the destination has really written the line, and engineers can later retrieve logs by destination, module, minimum level and sequence range.

## Problem

Build the logging core. A caller logs from any thread and gets back the number of its log once the destination confirmed the write; if the destination could not write it, the caller is told so and no number is wasted. Logs flow through a bounded buffer to a consumer so that a slow destination slows producers down instead of exhausting memory. Retrieval returns matching logs in sequence order, deterministically, and must be consistent with what the destinations received.

## Requirements

1. The modules are the fixed set `AUTH`, `PAYMENTS`, `SEARCH`; destinations are `FILE` and `TOOL`; levels are ordered `DEBUG < INFO < WARN < ERROR`. All are case-sensitive. Anything else, or a null/blank activity, is `InvalidArgument` and has no effect at all.
2. `log(module, destination, level, activity)` returns the log's sequence number. Numbers start at 1, increase by one and are unique and gap-free across all modules and destinations.
3. A log is acknowledged (the call returns its number) only after the destination sink's `write` returned `true`. The sink receives the line `<seq>|<MODULE>|<DESTINATION>|<LEVEL>|<activity>`.
4. If the sink write fails (`write` returns `false`), `log` returns `Err(WriteFailed)`. A failed log is not given a sequence number, is not stored and is not retrievable; the next successful log receives the next number, so acknowledged numbers stay gap-free.
5. Each sink receives its lines in increasing sequence order, one at a time per line, and every acknowledged log is written exactly once.
6. All accepted logs go through one shared buffer of the capacity given at construction. A producer waits while the buffer is full; nothing is dropped.
7. `query(destination, module, level, fromSeq, toSeq)` returns the acknowledged logs that match every non-null filter, as lines, in increasing sequence order. `level` means "this level or above". `fromSeq` and `toSeq` are inclusive. Unknown filter values, or `fromSeq > toSeq`, are `InvalidArgument`.
8. A log acknowledged to its caller is immediately visible to `query`.
9. `close()` stops accepting logs, lets everything already accepted finish, stops any background thread and returns. It is idempotent. `log` after `close` fails with `Closed`; `query` keeps working.

## Constraints

- Many producer threads call `log` (and other threads call `query`) concurrently: sequence numbers are unique and gap-free, and no log is lost or written twice.
- The sinks are slow and may fail. A slow sink must not make the framework lose logs; a failing sink must not poison later logs or the other destination.
- Tests never sleep or read the clock; every background thread must have stopped when `close()` returns.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- `log` returns `Result<u64, LogError>`: `Ok(n)` with `n >= 1` on success, `Err(WriteFailed)` when the sink write failed, `Err(InvalidArgument)` for bad input, `Err(Closed)` after `close`.
- Line format everywhere (sink and query result): `<seq>|<MODULE>|<DESTINATION>|<LEVEL>|<activity>`; the activity is verbatim (leading/trailing spaces and `|` are kept).
- A sequence number is never consumed by an invalid, failed or rejected log.
- `query` returns an owned `Vec<String>` (empty when nothing matches); filters are `Option<&str>` / `Option<u64>`.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Create whatever modules and types you like and return them from `create` in `src/lib.rs`.

```rust
pub enum LogError { InvalidArgument, WriteFailed, Closed }

/// A simulated destination, implemented by the tests.
pub trait Sink: Send + Sync {
    /// true = written, false = failed.
    fn write(&self, line: &str) -> bool;
}

pub trait LogService: Send + Sync {
    fn log(&self, module: &str, destination: &str, level: &str, activity: &str) -> Result<u64, LogError>;
    fn query(&self, destination: Option<&str>, module: Option<&str>, level: Option<&str>,
             from_seq: Option<u64>, to_seq: Option<u64>) -> Result<Vec<String>, LogError>;
    fn close(&self);
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(file: Arc<dyn Sink>, tool: Arc<dyn Sink>, buffer_capacity: usize) -> Option<Arc<dyn LogService>>
```

## Examples

```
svc = create(fileSink, toolSink, 8)
log("AUTH", "FILE", "WARN", "bad password")      -> 1     fileSink received "1|AUTH|FILE|WARN|bad password"
log("PAYMENTS", "TOOL", "INFO", "charged")       -> 2     toolSink received "2|PAYMENTS|TOOL|INFO|charged"
log("BILLING", "FILE", "INFO", "x")              -> Err(InvalidArgument)   (no number consumed)
```

### Worked traces

```
toolSink starts failing
log("SEARCH", "TOOL", "ERROR", "index down")     -> Err(WriteFailed)   (not stored, number not consumed)
log("SEARCH", "FILE", "ERROR", "index down")     -> 3     (fileSink still works; 3 is the next free number)
toolSink recovers
log("SEARCH", "TOOL", "INFO", "index up")        -> 4

query(null, null, null, null, null)              -> lines 1, 2, 3, 4
query("FILE", null, null, null, null)            -> lines 1, 3
query(null, null, "WARN", null, null)            -> lines 1 (WARN), 3 (ERROR)      // WARN or above
query(null, "SEARCH", null, 3, 4)                -> lines 3, 4
query(null, null, null, 4, 3)                    -> Err(InvalidArgument)
query("DISK", null, null, null, null)            -> Err(InvalidArgument)
close(); log("AUTH","FILE","INFO","late")        -> Err(Closed)
```

### Edge cases

- Invalid module, destination, level or a blank activity: nothing is written, nothing is consumed; the next valid log still gets the next number.
- A failure on one destination does not affect the other, and the first success after failures continues the numbering without a gap.
- The filters combine with AND; `null` means "no filter". A range that matches nothing returns an empty list.
- With 16 threads logging 500 logs each through a small buffer, the returned numbers are exactly 1..8000, each once.
- Calling `close()` twice is fine; `query` after `close()` still returns everything acknowledged before it.

## Glossary

- **Module**: one of the three applications that produce logs.
- **Destination**: where a log is delivered (`FILE` or `TOOL`); each has a sink.
- **Sink**: the simulated target the tests provide; `write(line)` reports success or failure.
- **Sequence number**: the position of an acknowledged log in the global order, starting at 1.
- **Acknowledged**: the sink wrote the line and the caller received its number.
- **Buffer**: the bounded place accepted logs wait in before they are written.

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Sinks are fakes controlled by the test; nothing sleeps or reads the clock. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Numbering, routing, line format, every module and level, verbatim activity text.
- Invalid input (module, destination, level, blank activity) and that it consumes nothing.
- Sink failure (`write` returns false): `Err(WriteFailed)`, no number consumed, not retrievable, other destination unaffected.
- Every filter alone and combined, the inclusive range, the minimum-level rule, invalid filters, snapshot semantics, close.
- Concurrency: 16 threads x 500 logs unique and gap-free; per-sink ordering; a flaky sink under 8 threads stays gap-free over acknowledged logs.

## Running

```bash
cargo test -p p113_multi_module_logging                    # from rust/
STRICT=1 cargo test -p p113_multi_module_logging            # fail instead of skip while unimplemented
cargo test -p p113_multi_module_logging --features hidden   # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` returns `Some`, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`run_concurrently`, `sut!`): `rust/support`.
