# 59 · Decorator (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — the `MessageSink` trait and `SinkError` provided in `src/contract.rs`; the rest is yours to write. |

## Problem

An application sends messages (alerts, audit lines) to a sink: `MessageSink::send(&self, message) -> Result<(), SinkError>`,
which returns `Err` when delivery fails. Different call sites need different combinations of extras: a timestamp,
redaction of card numbers, retries on network errors, delivery counting. Build each extra as a small sink of its own that
wraps any other `MessageSink`, adds its one behaviour, and passes the message on, so call sites can stack whichever extras
they need in whichever order.

## Terms

- **Wrapped sink** (`next`): the sink a decorator forwards to after doing its own work.
- **Digit run**: a maximal sequence of consecutive characters `0`-`9` (a letter, space or punctuation ends it).

## Requirements

You write four types: `TimestampingSink`, `RedactingSink`, `RetryingSink`, `CountingSink`. All implement `MessageSink` and take the sink to wrap (a `Box<dyn MessageSink>`) as their **first** constructor argument.

1. `TimestampingSink::new(next, clock)` (`Arc<dyn Clock>`): forwards `"[t=" + clock.now_millis() + "] " + message`.
2. `RedactingSink::new(next)`: every digit run of **4 or more digits** is replaced by `****` (one `****` per run, regardless of length); runs of 1–3 digits stay as they are.
3. `RetryingSink::new(next, max_attempts)`: calls `next.send(message)` (the same message each time); if it returns `Err` it tries again, up to `max_attempts` calls in total (0 behaves like 1), stopping at the first `Ok`. After the last failure it returns the **last** error. No retry happens after a success.
4. `CountingSink::new(next)`: forwards the message; `count()` is the number of messages delivered successfully (an `Err` is not counted, and is still returned to the caller).
5. **Order matters.** `Redacting(Timestamping(x))` redacts the message and then adds the timestamp; `Timestamping(Redacting(x))` adds the timestamp first, so its 13 digits are redacted too: `"[t=****] ..."`. (The outermost decorator sees the message first.)

## Examples

Clock fixed at `1700000000000`; `x` is a recording sink at the bottom.

| Stack (outer → inner) | `send(...)` | `x` receives |
|---|---|---|
| `Timestamping(x)` | `"hello"` | `"[t=1700000000000] hello"` |
| `Redacting(x)` | `"card 4111111111111111 exp 1226 room 101 floor 7"` | `"card **** exp **** room 101 floor 7"` |
| `Redacting(Timestamping(x))` | `"pin 123456"` | `"[t=1700000000000] pin ****"` |
| `Timestamping(Redacting(x))` | `"pin 123456"` | `"[t=****] pin ****"` |

Retry: `x` fails its first 2 calls. `Retrying(x, 3).send("important")` → `x` is called 3 times, the 3rd succeeds, `x` receives `"important"`. If `x` fails 5 times and `max_attempts` is 2, `send` returns `Err(SinkError("network down #2"))` (the error from the 2nd call) and `x` was called twice.

Stack: `Counting(Retrying(Redacting(Timestamping(x, clock)), 3))`, `x` fails its first call. `send("otp 998877")` then `send("ok")` → `count()` is 2, `x` was called 3 times, and received `"[t=1700000000000] otp ****"` then `"[t=1700000000000] ok"`.

## Edge cases

- Exactly 3 digits stays (`101`); exactly 4 is redacted (`1226`).
- `RetryingSink` with `max_attempts = 1` makes one call and never retries.
- A failed send inside a `CountingSink` leaves `count()` unchanged.

## Running

```bash
cargo test -p p59_decorator     # from rust/ — tests FAIL until you implement the decorators
cargo play run 59
```
