# 59 · Decorator (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — the `MessageSink` trait and `SinkError` are provided in `src/contract.rs`; four decorators are stubs. |

> **Pattern:** *Decorator* — attach extra behaviour to an object **dynamically** by wrapping it in another object
> that has the **same interface**. Behaviours can be stacked in any order; no type explosion
> (`TimestampedRedactedRetryingSink`...). In Rust a decorator is a struct holding a `Box<dyn MessageSink>` that
> implements `MessageSink` itself.

## Problem

An application sends messages (alerts, audit lines) to a sink. Different call sites need different combinations of
extras: a timestamp, redaction of card numbers, retries on network errors, delivery counting. Build each extra as a
decorator that wraps any `MessageSink`.

## Requirements

All decorators implement `MessageSink` and take the sink to wrap as their **first** constructor argument.

1. `TimestampingSink::new(next, clock)`: sends `"[t=" + clock.now_millis() + "] " + message`.
2. `RedactingSink::new(next)`: every run of **4 or more digits** becomes `****`; runs of 1–3 digits stay.
3. `RetryingSink::new(next, max_attempts)`: tries up to `max_attempts` times (0 behaves like 1) when `next` returns `Err`; after the last failure returns the **last** error.
4. `CountingSink::new(next)`: `count()` is the number of messages delivered successfully (an `Err` is not counted).
5. **Order matters.** `Redacting(Timestamping(x))` redacts the message and then adds the timestamp; `Timestamping(Redacting(x))` adds the timestamp first, so its 13 digits are redacted too: `"[t=****] ..."`.

## Constraints (what the tests catch)

- The innermost sink is a `Recorder` written in the test (it can fail the first N sends), and the tests stack all four decorators around it.

## Running

```bash
cargo test -p p59_decorator     # from rust/ — tests FAIL until you implement the decorators
cargo play run 59
```
