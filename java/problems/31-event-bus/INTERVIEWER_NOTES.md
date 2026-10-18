# Interviewer notes — 31 In-Process Event Bus

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- A subscription created after publish but before delivery must not get the event.
- An already-running callback finishes after unsubscribe; its pending events do not run.
- A callback that publishes and then throws: the published event still goes out.
- Subscribe after close never receives; close twice is fine.
- Topics are case-sensitive; empty events are events.

## Likely interviewer questions
- Where do events wait: one global queue, one queue per subscription, per topic? What does each choice do to head-of-line blocking by a slow subscriber?
- How is "idle" defined and tracked when callbacks publish more events (counter of outstanding events incremented *before* publish returns)?
- How do you take the subscriber snapshot without holding a lock while delivering?
- How many threads? A pool plus per-subscription serial mailboxes (a "serial executor" per subscription) scales better than thread-per-subscription.
- What does `close` do to in-flight and pending work?

## Invariants
- Outstanding-work counter equals the number of (event, subscription) deliveries not yet finished or discarded.
- A subscription's callback never runs concurrently with itself.
- No lock is held while a callback runs.
- Removing a subscription removes it from every index (no leaks, no zombie deliveries).

## Common mistakes
- One shared single-thread queue: one blocked callback stalls everybody.
- Running callbacks while holding the registry lock (re-entrant publish/unsubscribe deadlocks or `ConcurrentModificationException`).
- Computing the receiver set at delivery time instead of at publish time.
- Idle detection that misses events published by callbacks, or that counts an event as done before its callback ran.
- Counting a thrown callback as "never finished", so `awaitIdle` never returns.
- Unsubscribe that waits for the running callback (self-unsubscribe deadlock).
- Unbounded thread creation per event.

## Over-engineering traps
- `Event`, `Topic`, `Subscriber`, `Dispatcher`, `DeliveryStrategy`, `EventBusFactory` hierarchies for something a map of topic → subscriptions plus per-subscription mailboxes covers.
- A priority system or persistence nobody asked for.

## Concurrency discussion
- Per-subscription serial mailbox drained by a shared pool: preserves per-subscriber order, isolates slow handlers, bounded threads. A mailbox is "scheduled" iff non-empty and not running.
- Idle tracking: atomic counter + condition/monitor; increment on enqueue, decrement after handling or discard, notify at zero.
- Copy-on-write subscriber lists make the publish snapshot cheap.

## Trade-offs worth probing
- Copy-on-write snapshots vs locking the registry per publish.
- Unbounded mailboxes (memory) vs backpressure (publisher can block).
- Thread per subscription (simple, fragile) vs pool + serial executors.

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.
