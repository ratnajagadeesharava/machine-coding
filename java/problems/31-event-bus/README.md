# 31 · In-Process Event Bus (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 120 min | Level 3 — infrastructure provided. Test utilities for threads/clocks exist; business/domain code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Components of one application need to talk without knowing each other. Build an in-process, asynchronous
publish/subscribe facility: parties register interest in a named topic, other parties announce events on that
topic, and every interested party gets to react — without a slow or broken party hurting the rest.

## Requirements

1. A party **subscribes** to a topic with a callback and receives a **subscription id**; it can **unsubscribe** with that id.
2. Anyone can **publish** a string event to a topic. Every party subscribed to that topic **at the moment of publishing** receives the event exactly once.
3. Publishing is asynchronous: `publish` hands the event over and returns; it never waits for callbacks to finish and never drops an event because callbacks are slow.
4. A party sees events in the order they were published (for events announced one after another by the same thread).
5. A callback that blocks for a long time must not prevent other subscribers — on the same topic or others — from receiving their events.
6. A callback that fails (throws / panics) must not affect other callbacks, other events, or the bus. It keeps receiving later events.
7. Callbacks may themselves publish (and subscribe/unsubscribe) without deadlocking.
8. A caller can **wait until the bus is idle**, i.e. everything published so far — including what callbacks published in turn — has been dealt with, or give up after a timeout.
9. The bus can be **closed**.

## Constraints

- Many threads publishing, subscribing and unsubscribing at once; thousands of subscriptions and events.
- Tests must be able to observe progress without sleeping: `awaitIdle` is the synchronisation point.
- Bounded resource use: do not assume one thread per subscription is acceptable beyond a toy.

## Examples

```
s1 = subscribe("orders", A)    s2 = subscribe("orders", B)    s3 = subscribe("billing", C)
publish("orders", "o-1")   → A and B each get "o-1" once, C gets nothing
A blocks forever           → B (and C) still receive every later event
A publishes "billing" events from inside its callback → C gets them, no deadlock
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- Topics are exact, case-sensitive strings (no wildcards). Events are arbitrary strings, including empty ones. Subscription ids are non-empty, opaque and unique. Subscribing the same callback twice creates two independent subscriptions.
- **Snapshot rule:** the receivers of an event are exactly the subscriptions that exist when `publish` is called. A subscription created later never receives it, even if the event is still waiting for other callbacks; a subscription removed before the event starts being handled does not receive it. An event with no subscribers is silently discarded.
- **Ordering:** for one subscription, events are handled in the order the `publish` calls happened, for calls that are ordered with respect to each other (same thread, or otherwise happens-before). No order is promised between events from unrelated concurrent publishers. A single subscription's callback is **never run concurrently with itself**; different subscriptions run independently.
- **Unsubscribe:** `unsubscribe(id)` returns `true` the first time for a known id, `false` for unknown or already removed ids. After it returns, the callback is never *started* again — events that were published earlier but not yet started are discarded for that subscription. An invocation that is already running is allowed to finish; `unsubscribe` does not wait for it (so it may be called from inside a callback, including the subscription's own).
- **Failures:** a callback that throws/panics counts as having handled that event; later events are still delivered to it.
- **`awaitIdle(timeoutMillis)`:** returns `true` as soon as there is nothing left to do: every event published so far (by anyone, including callbacks) has been handled or discarded and no callback is running. Returns `false` if that has not happened within the timeout. On a bus with nothing pending it returns `true` immediately (also for timeout 0).
- **`close()`:** idempotent. Events published before `close` are still delivered; `publish` after `close` silently discards the event (no error); `subscribe` after `close` still returns an id but the callback never receives anything. `awaitIdle` keeps working after `close`.

## Test entry point (Java)

The tests drive your system through the thin, primitive-typed contract below. It is **not** your domain
model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in
`solution/Solution.java`.

```java
package contract;

import java.util.function.Consumer;

/** Thin test entry point. Your domain model lives behind it. Must be thread-safe. */
public interface EventBusApi {

    /** Registers the handler for the topic and returns the subscription id. */
    String subscribe(String topic, Consumer<String> handler);

    /** True the first time for a known id; false for unknown or already removed ids. */
    boolean unsubscribe(String subscriptionId);

    /** Hands the event to every current subscriber of the topic; never waits for handlers. */
    void publish(String topic, String event);

    /** True once everything published so far (including by handlers) has been handled or discarded. */
    boolean awaitIdle(long timeoutMillis);

    /** Stops accepting new events; events already published are still delivered. Idempotent. */
    void close();
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    EventBusApi create()
}
```

## Language notes

Handlers are `Consumer<String>` receiving the event. Use daemon threads (or call `close()`), otherwise the test JVM may not exit. Tests only use `awaitIdle` and latches to synchronise.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/31-event-bus test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.
