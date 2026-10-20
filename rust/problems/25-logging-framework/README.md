# 25 · Logging Framework (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 90 min | Level 2 — minimal external API. The public interface is part of the problem; everything behind it is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building a small logging framework like the ones every application uses (think SLF4J/Log4j in miniature). Code emits a message under a **logger name** (say `app.db`) at a **severity level** (say `WARN`). The framework decides whether the message is worth keeping, using a per-logger threshold that can be inherited down a dotted-name hierarchy; if it is, it formats it into one line of text and hands that line to every output destination that wants it.

Destinations are not yours to design: an output destination is something the *caller* supplies (it implements the small `LogSink` contract below and receives finished text lines, for example a file writer or an in-memory list in tests). Your job is everything in between: levels, the logger hierarchy, message formatting, and routing to the registered destinations.

**Terms**

- **Logger**: a dotted name such as `app.db.pool`; the thing messages are emitted under. Never needs to be created first.
- **Threshold**: the minimum level a logger accepts. A message below it is dropped.
- **Effective threshold**: the threshold that actually applies to a logger: its own if it has one, else its nearest ancestor's, else the root's.
- **Appender**: a named destination (a sink plus a minimum level), added to and removed from the framework at runtime.
- **Template**: message text with `{}` placeholders, filled from the arguments given with the message.

## Requirements

1. Severity levels, lowest to highest: `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`.
2. Loggers are identified by dotted names (`app`, `app.db`, `app.db.pool`). They form a hierarchy: `app.db` is a child of `app`, which is a child of the **root** logger. A logger need not be created or registered before it is used.
3. Each logger may have its own threshold level. A logger without one **inherits** the threshold of its nearest ancestor that has one; the root logger's threshold is `INFO` until changed. Thresholds can be changed at any time and the change is effective immediately for all loggers that inherit it.
4. A message is produced only if its level is at least the logger's effective threshold.
5. Message text is a template with `{}` placeholders that are filled, in order, from the arguments given with the message.
6. A produced message becomes one line of text with a fixed layout and is handed to the **appenders**. An appender is a named destination with its own minimum level: it only receives messages at or above that level (in addition to the logger's threshold). Appenders can be added and removed at any time.
7. A destination that fails (throws) must not prevent the others from receiving the message, nor break the caller.

## Constraints

- Time comes from the clock given to the factory.
- Expect hundreds of thousands of messages; filtering a message that is below the threshold must be cheap (do not format first and filter later).
- The framework is used from a single thread in the tests.

## Examples

Clock reads `1700000000123` and one appender (minimum level `TRACE`) collects lines. Root threshold is the default `INFO`.

```
log("app", "INFO", "started")
    -> line "1700000000123 [INFO] app - started"
log("app.db", "WARN", "slow query {}ms on {}", "850", "orders")
    -> line "1700000000123 [WARN] app.db - slow query 850ms on orders"
log("app", "DEBUG", "hidden")                         -> nothing (DEBUG < INFO, the root default)
```

Threshold inheritance:

```
set_level("app", "ERROR") -> true
log("app.db.pool", "WARN", "x")          -> nothing (inherits ERROR from app)
log("app.db.pool", "ERROR", "x")         -> line
log("application", "INFO", "x")          -> line  (application is NOT a child of app)
set_level("app.db", "DEBUG")
log("app.db.pool", "DEBUG", "x")         -> line  (nearest configured ancestor is now app.db)
log("app", "DEBUG", "x")                 -> nothing (still ERROR)
set_level("app", "LOUD") -> false         set_level("app", "info") -> false   (names are upper case only)
```

Appenders (root threshold `DEBUG`; appender `w` has minimum `WARN`, appender `e` has minimum `TRACE`):

```
log("app", "TRACE", "t")  -> nobody (below the logger threshold)
log("app", "DEBUG", "d")  -> e only
log("app", "WARN",  "w")  -> w, then e (in the order they were added)
```

Template filling:

```
"a={} b={} c={}"  with ["1"]            -> "a=1 b={} c={}"      (missing arguments leave {} as is)
"only {}"         with ["x","y","z"]    -> "only x"             (extra arguments ignored)
"x={} y={}"       with ["{}","2"]       -> "x={} y=2"           (an inserted "{}" is not filled again)
"{ } { } } {{}} $1 \ {}"  with ["A"]   -> "{ } { } } {A} $1 \ {}"   (only an exact "{}" is a placeholder)
```

## Edge cases

- The root logger is named `root`; `set_level("root", ...)` changes the default for every logger without an ancestor that has its own level.
- A threshold change takes effect immediately, even for loggers that were already used.
- An unknown level name or blank logger name in `log` is silently ignored; no line is produced and nothing is thrown.
- Adding an appender with a taken or blank name, or an unknown minimum level, returns `false` and leaves everything unchanged; after `remove_appender` the name can be reused.
- A new appender sees only messages logged after it was added; no appenders means the message simply goes nowhere.
- A sink that panics must not stop later appenders from receiving the line, and `log` still returns normally.
- The timestamp is the clock's epoch milliseconds at the moment of each `log` call.

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- Level names are exactly `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR` (upper case). The root logger is named `root`.
- `setLevel(logger, level)` returns `true` when applied, `false` (and changes nothing) for an unknown level name or a blank logger name. Setting the level of `root` changes the default for every logger that has no ancestor with its own level. Setting a level twice: the last one wins.
- Only whole dot-separated segments define the hierarchy: `application` is **not** a child of `app`; `app.db` is. A logger's ancestors are its dot-prefixes, nearest first, then `root`.
- `log(logger, level, template, args…)` does nothing for an unknown level name or blank logger name. Otherwise, if `level ≥` the logger's effective threshold, it builds one line and delivers it to every appender whose minimum level is `≤ level`, in the order the appenders were added.
- Line layout, exactly: `<epochMillis> [<LEVEL>] <logger> - <message>` — epoch milliseconds from the clock at the moment of the call, the level name upper case without padding, the logger name as given, then a single space-hyphen-space and the message. No trailing newline.
- Message building: scan the template left to right; each `{}` is replaced by the next argument, as text. If there are fewer arguments than `{}`, the remaining `{}` stay in the output literally. Extra arguments are ignored. Text that comes from an argument is never scanned for `{}` again. There is no escaping; anything that is not exactly `{}` (such as `{`, `}` or `{ }`) is copied as is.
- `addAppender(name, sink, minLevel)` returns `true` if registered, `false` if the name is already in use, blank, or the level is unknown (nothing changes). A new appender only sees messages logged after it was added. `removeAppender(name)` returns `true` if it existed (it receives nothing afterwards, and the name can be reused) and `false` otherwise.
- If a sink throws, the remaining appenders still receive the message, and `log` returns normally.
- With no appenders registered a message goes nowhere (and nothing fails).

## Test entry point (Rust)

The tests drive your system through the thin, primitive-typed contract below (`src/contract.rs`). It is
**not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up
in `src/lib.rs`.

```rust
//! Thin test entry point. Your domain model lives behind it.

use std::sync::Arc;

/// An output destination supplied by the caller. Receives complete lines (no trailing newline).
pub trait LogSink: Send + Sync {
    fn write(&self, line: &str);
}

pub trait LoggingApi {
    /// Levels: TRACE, DEBUG, INFO, WARN, ERROR. The root logger is named "root". False if invalid.
    fn set_level(&mut self, logger_name: &str, level: &str) -> bool;

    /// Template placeholders are "{}". Ignored for an unknown level or blank logger name.
    fn log(&mut self, logger_name: &str, level: &str, template: &str, args: &[&str]);

    /// False if the name is blank or taken, or the level is unknown.
    fn add_appender(&mut self, name: &str, sink: Arc<dyn LogSink>, min_level: &str) -> bool;

    fn remove_appender(&mut self, name: &str) -> bool;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(_clock: Arc<dyn Clock>) -> Option<Box<dyn LoggingApi>> {
    None
}
```

## Language notes

`LogSink` is a trait (`Send + Sync`, `write(&self, &str)`); sinks are passed as `Arc<dyn LogSink>`. A sink may panic. The trait is `&mut self`: the tests use it from a single thread. Template arguments arrive as `&[&str]`.

## Running

```bash
cargo test -p p25_logging_framework
STRICT=1 cargo test -p p25_logging_framework   # fail instead of skip while unimplemented
cargo test -p p25_logging_framework --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p25_logging_framework --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.
