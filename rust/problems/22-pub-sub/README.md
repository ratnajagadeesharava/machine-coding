# 22 · Pub-Sub Message Broker (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 90 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Build an in-memory message broker. Publishers send text messages to named topics; every subscriber of a topic gets its own
copy of each message and fetches (pulls) its messages whenever it likes. Many threads publish, subscribe and poll at the
same time.

## Requirements

1. Topics are created explicitly with a name. A topic name must not be blank and must not exist yet.
2. A subscriber (identified by a string) subscribes to an existing topic. A subscriber may be subscribed to many topics, and many subscribers may share a topic.
3. A message is delivered **only to those subscribed at the moment it is published**. Nobody receives messages that were published before they subscribed.
4. `publish` reports how many subscribers the message was delivered to. A topic with no subscribers accepts the message and drops it.
5. `poll` returns up to `max` of the subscriber's pending messages for a topic, **oldest first**, and **removes them from the subscriber's pending messages** (a message is returned by `poll` at most once per subscriber).
6. Each subscriber's pending messages are independent: consuming a message does not affect what other subscribers can still read.
7. Within a topic, messages are delivered in the order in which the publishes took effect; one publisher's messages therefore always arrive in the order it published them.
8. `unsubscribe` ends the subscription and discards that subscriber's pending messages for the topic. Subscribing again later starts from scratch (rule 3 applies again).
9. Invalid requests (unknown topic, not subscribed, duplicate subscription, blank names) are refused and change nothing.
10. Publishing, subscribing, unsubscribing and polling are called concurrently. Each publish must take effect atomically: its recipients are exactly the subscribers present at one single instant, none skipped and none receiving it twice.

## Constraints

- Thousands of topics, up to hundreds of subscribers per topic, tens of thousands of pending messages.
- Thread-safe. No message may be lost, duplicated or reordered for a subscriber that stays subscribed.
- A slow subscriber must not block publishers.

## Examples

```
createTopic("news") -> true
subscribe("news", "ann"); publish("news", "m1") -> 1       subscribe("news", "bob"); publish("news", "m2") -> 2
poll("ann", "news", 10) -> [m1, m2]                       poll("bob", "news", 10) -> [m2]
poll("ann", "news", 10) -> []                              publish("nope", "x") -> -1
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `createTopic` returns `true` when the topic was created; `false` for a blank name or an existing topic.
- `subscribe` returns `true` when the subscription was added; `false` for an unknown topic, a blank subscriber or when the subscriber is already subscribed to that topic.
- `unsubscribe` returns `true` when a subscription was removed; `false` otherwise.
- `publish` returns the number of subscribers the message was delivered to (`0` is valid), or `-1` for an unknown topic.
- `poll(subscriber, topic, max)` returns a fresh list of at most `max` messages; it is empty for an unknown topic, a subscriber that is not subscribed, or `max <= 0` (and in the last case nothing is consumed).
- Messages are opaque strings; the same text published twice is two messages.

## Test entry point (Rust)

The tests drive your system through the thin, primitive-typed contract below (`src/contract.rs`). It is
**not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up
in `src/lib.rs`.

```rust
//! Thin test entry point. Your domain model lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

pub trait PubSubApi: Send + Sync {
    fn create_topic(&self, topic: &str) -> bool;

    fn subscribe(&self, topic: &str, subscriber: &str) -> bool;

    fn unsubscribe(&self, topic: &str, subscriber: &str) -> bool;

    /// Number of subscribers the message was delivered to; -1 for an unknown topic.
    fn publish(&self, topic: &str, message: &str) -> i32;

    /// Up to `max` pending messages, oldest first, consumed by the call.
    fn poll(&self, subscriber: &str, topic: &str, max: i32) -> Vec<String>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create() -> Option<Arc<dyn PubSubApi>> {
    None
}
```

## Language notes

The trait is `&self` + `Send + Sync`. A `RwLock`/`Mutex`-protected map from topic to per-topic state is a reasonable start; think about whether the topic lock is held during the whole publish.

## Running

```bash
cargo test -p p22_pub_sub
STRICT=1 cargo test -p p22_pub_sub   # fail instead of skip while unimplemented
cargo test -p p22_pub_sub --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p22_pub_sub --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.
