# 59 · Decorator (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — the `MessageSink` interface is provided; the rest is yours to write. |

## Problem

An application sends messages (alerts, audit lines) to a sink. Different call sites need different combinations of
extras: a timestamp, redaction of card numbers, retries on network errors, delivery counting. Build each extra as a
decorator that wraps any `MessageSink`.

## Requirements

All decorators implement `MessageSink`, take the sink to wrap as their **first** constructor argument (a `null` → `IllegalArgumentException`).

1. `TimestampingSink(next, clock)`: sends `"[t=" + clock.millis() + "] " + message`.
2. `RedactingSink(next)`: every run of **4 or more digits** becomes `****`; runs of 1–3 digits stay.
3. `RetryingSink(next, maxAttempts)`: `maxAttempts < 1` → `IllegalArgumentException`. Tries up to `maxAttempts` times when `next` throws a `RuntimeException`; after the last failure rethrows the **last** exception.
4. `CountingSink(next)`: `count()` is the number of messages delivered successfully (a send that throws is not counted).
5. **Order matters.** `Redacting(Timestamping(x))` redacts the message and then adds the timestamp; `Timestamping(Redacting(x))` adds the timestamp first, so its 13 digits are redacted too: `"[t=****] ..."`.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the decorators
```
