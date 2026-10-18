# Interviewer notes — 22 Pub-Sub Message Broker

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- Subscribing again after unsubscribing must not bring back old messages.
- Publishing to a topic with no subscribers returns 0 and the message is not kept for later subscribers.
- Partial polls leave the remainder in order, and messages published afterwards queue behind them.
- A non-subscriber and a subscriber of a different topic poll nothing.
- Same text published twice is delivered twice.

## Likely interviewer questions
- Do you store one queue per subscriber (copy on publish) or one log per topic with a cursor per subscriber? What does each cost in memory, in `publish`, in `poll`, in `unsubscribe`?
- What makes `publish` atomic with respect to `subscribe`? Which lock (or lock-free structure) decides the set of recipients?
- How do you stop a slow or abandoned subscriber from growing memory forever? (V3)
- Where does ordering come from: a per-topic lock, a per-topic sequence number, the queue itself?
- How would the design change for push delivery?

## Invariants
- For a subscriber that stays subscribed: received messages = all messages published after its subscription took effect, once each, in per-topic order.
- The number returned by `publish` equals the number of pending-queues the message was appended to.
- Refused operations change nothing.
- `poll` removes exactly what it returns.

## Common mistakes
- Iterating the live subscriber collection during `publish` while `subscribe`/`unsubscribe` mutate it (exception or skipped subscriber).
- Appending to subscribers one by one without ordering the whole publish against others, so two subscribers see two publishers' messages in different orders.
- A `poll` that peeks, then removes in a second step (two concurrent pollers get the same message).
- Returning a view of the internal queue instead of a copy.
- Keeping a pending queue after unsubscribe, so a resubscribe resurrects old messages.
- Counting recipients before the append (the number disagrees with the delivery).
- Locking the whole broker for every operation on every topic when topics are independent.

## Over-engineering traps
- `Broker`, `Topic`, `Subscription`, `Message` with id/timestamp/headers, `DeliveryStrategy`, `Dispatcher` thread pool, `EventBus` interfaces. Topic name to subscriber-name to queue is the core.
- Executor-based push in V1 when the contract is pull-based.

## Concurrency discussion
- Per-topic lock around "snapshot recipients and append to each queue" gives a clean total order per topic and is usually fast enough.
- Concurrent maps give atomic membership changes but not an atomic multi-queue append unless something orders the publishes.
- Log-with-cursors needs only one append under the topic lock; polls then contend only on their own cursor.

## Trade-offs worth probing
- Queue per subscriber: fast poll, `O(subscribers)` publish and memory. Shared log: `O(1)` publish, harder cleanup of consumed prefixes.

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.
