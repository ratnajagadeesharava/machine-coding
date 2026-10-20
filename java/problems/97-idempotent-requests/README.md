# 97 · Idempotent Request Processor (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 60 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

You are building the guard in front of a payments API. Mobile clients on flaky networks retry requests, and a retry must never
charge a customer twice. Every request carries an idempotency key; the guard guarantees the business operation (a handler)
runs at most once per key and that every retry with the same key gets exactly the first response. Retries often arrive while
the first attempt is still running, a client sometimes reuses a key for a different request by mistake, handlers sometimes
fail, and the service runs for months, so stored results expire after 24 hours and memory must not grow forever.

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
8. A call's result is one of: the handler ran and its response is returned; a stored response is replayed; the call waited for another call's execution and shares its response; or an error (`KeyConflictException`, `IllegalArgumentException` or the handler's own exception).
9. Validation happens first and runs no handler: null/blank key and null body are rejected before anything is stored.
10. A conflict (same key, different body) never changes the stored or in-flight call: the original response stays replayable for the original body.
11. The 24 h window starts when the response is stored and is not extended by replays. A new execution after expiry stores its response with a new 24 h window.

## Simulation rules and constraints

- The processor is shared by many request threads; handlers may take long and may sleep. Treat them as black boxes that can block.
- Handlers for different keys must not wait on each other: 16 different keys with 150 ms handlers must finish in about 150 ms, not 16 x 150 ms.
- Waiters of the same key share the one execution: 32 threads with a 100 ms handler finish in about 100 ms and the handler ran once.
- Time comes from the injected clock (`FakeClock`); the tests move it by hand, handler `sleep`s are real but only used to keep calls overlapping.
- No network simulation is involved: this is a thread-safety and exactly-once problem on one process. The distributed angle (what survives a crash) belongs to the follow-ups.

## Glossary

- **Idempotency key**: a client-chosen string identifying one logical request; all retries of it carry the same key.
- **Handler**: the business operation (charge a card, create an order) that takes the request body and returns a response string or fails.
- **Replay**: returning the stored response for a key without running the handler again.
- **In flight**: a key whose handler is running right now and has not finished.
- **Conflict**: a request using a known (stored or in-flight) key with a body different from the original.

## Examples

**Example 1: run once, replay** (handler returns `"done:" + body + ":" + callNumber`)

```
process("k1", "pay 10", h) -> "done:pay 10:1"      handler runs
process("k1", "pay 10", h) -> "done:pay 10:1"      replay, handler not run (calls = 1)
process("k2", "pay 10", h) -> "done:pay 10:2"      a different key is independent
process("k1", "pay 99", h) -> KeyConflictException   same key, different body; calls still 2
process("k1", "pay 10", h) -> "done:pay 10:1"      the stored response is still replayable
```

**Example 2: a failing handler stores nothing**

```
h throws "boom" on its first call and returns "ok:2" on the second
process("k", "x", h) -> throws "boom" (to this caller)      nothing stored
process("k", "x", h) -> "ok:2"                              handler runs again
process("k", "x", h) -> "ok:2"                              replay; handler called 2 times in total
```

**Example 3: expiry after 24 hours** (stored at time 0)

```
at 24h - 1 ms: process("k","x",h) -> replayed response   (valid while now < storedAt + 24h)
at 24h:        process("k","x",h) -> handler runs again (new response, new 24h window)
```
and, for a key whose response was stored at time 0 and is not used again:
```
at 25h:        process("k","new body",h) -> accepted; the key is free again, even with a different body
```

**Example 4: concurrency**

```
32 threads call process("same","body", handler that sleeps 100 ms) at once
  -> the handler runs once; all 32 get the identical response; total time is about 100 ms
16 threads with 16 different keys and 150 ms handlers -> about 150 ms in total, not 16 x 150 ms
first call still inside its handler, another call with the same key and body "B" instead of "A"
  -> KeyConflictException immediately (the handler of the second call does not run)
8 threads, same key and body, the first handler run fails after 100 ms
  -> 1 caller gets the failure, the other 7 get the successful response of the second handler run (2 handler runs in total)
```

## Edge cases

- A null or blank key, or a null body, gives `IllegalArgumentException`; the handler is not run.
- When the running handler fails, only the caller that ran it sees that failure (the same exception object type and message, unchanged); callers waiting for it do not fail, one of them runs the handler next and the rest share that outcome.
- Bodies are compared by value (equal strings are equal bodies).
- The stored response is returned byte for byte.
- Entries that expired must be reclaimed eventually, so the bookkeeping does not grow with the number of keys ever seen.

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

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

/** The business operation behind an idempotency key. Provided by the tests; it fails by throwing an unchecked exception. */
@FunctionalInterface
public interface Handler {
    String handle(String requestBody);
}

package contract;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Time comes from the clock handed to the factory.
 */
public interface IdempotentProcessor {

    /**
     * Runs {@code handler} for the request at most once per idempotency key and returns its response; every other
     * call with the same key and the same body returns the stored response without running the handler.
     * <ul>
     *   <li>Same key, DIFFERENT body (also while the first call is still running): {@link KeyConflictException}.</li>
     *   <li>Concurrent calls with the same key and body: one runs the handler, the others wait for it and return the same
     *       response. Calls with different keys never wait for each other.</li>
     *   <li>If the handler throws, nothing is stored: the exception reaches the caller that ran it, and the key is free
     *       again. Callers that were waiting for it do not fail; one of them runs the handler next.</li>
     *   <li>A stored response expires 24 hours after it was stored (then the key can be used again).</li>
     *   <li>IllegalArgumentException for a null/blank key or a null body.</li>
     * </ul>
     */
    String process(String idempotencyKey, String requestBody, Handler handler);
}

package contract;

/** The same idempotency key was reused for a different request body. Provided. */
public class KeyConflictException extends RuntimeException {
    public KeyConflictException(String message) {
        super(message);
    }
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static IdempotentProcessor create(Clock clock)
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Time-dependent behaviour uses an injectable fake clock, never real sleeping. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Run once and replay; different keys independent; conflict on a different body leaves the stored response replayable.
- Invalid arguments: null key, blank key, null body.
- A failing handler stores nothing and the next call runs it again; expiry at 24 h - 1 ms vs 24 h; a new body allowed after expiry.
- 32 racing threads on one key: handler once, one response, finishes quickly; 16 distinct keys run in parallel.
- Hand-over: with 8 threads and a first handler that fails after 100 ms, exactly one thread sees the failure, seven succeed, two invocations in total.
- A conflicting body is rejected while the first call is still blocked inside its handler.
- **Repeat 100x**: 8 racing threads, exactly one handler call.
- More edge cases and scale scenarios in `interviewer/` (see `INTERVIEWER_NOTES.md`).

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/97-idempotent-requests test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`SimNet`, `FakeClock`, `Candidate`): `java/testkit`.
