# 22 · Pub-Sub Message Broker (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 90 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building the heart of a small in-process messaging service, like a miniature Kafka/Google Pub/Sub that lives inside
one program. Publishers send text messages to named topics. Every subscriber of a topic gets its **own copy** of each message
and fetches (pulls) its messages whenever it likes; nothing is pushed to it. Many threads publish, subscribe, unsubscribe and
poll at the same time, and the broker must stay correct and fast.

**Terms**

- **Topic**: a named channel that messages are published to.
- **Subscriber**: a string id that has subscribed to one or more topics. Subscriptions are per topic: "ann on news" and "ann on sports" are separate.
- **Pending messages**: the messages delivered to one subscriber on one topic that it has not polled yet.
- **Poll**: pull-style fetch; it removes what it returns from the subscriber's pending messages.

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

Single-threaded traces on a `PubSubApi` value `api` (`api.` prefix omitted) (all of these follow from the rules and the tests):

```
createTopic("news")  -> true            createTopic("news") -> false     createTopic("  ") -> false
subscribe("news","ann") -> true         publish("news","m1") -> 1        // only ann is subscribed
subscribe("news","bob") -> true         publish("news","m2") -> 2
poll("ann","news",10) -> [m1, m2]       poll("bob","news",10) -> [m2]    // bob never sees m1
poll("ann","news",10) -> []             // already consumed
publish("nope","x")   -> -1             // unknown topic
```

Batching and consumption:

```
subscriber "s" on "t", then publish m1..m5
poll("s","t",2) -> [m1, m2]   poll("s","t",2) -> [m3, m4]   poll("s","t",2) -> [m5]   poll("s","t",2) -> []
poll("s","t",0) -> []  and  poll("s","t",-3) -> []          // nothing consumed; m1.. would still be pending
```

Independence and unsubscribing:

```
a and b subscribed to "t"; publish m1, m2
poll("a","t",10) -> [m1, m2]            poll("b","t",1) -> [m1]     poll("b","t",1) -> [m2]
unsubscribe("t","a") -> true            unsubscribe("t","a") -> false
publish("t","m3") -> 1                  poll("a","t",10) -> []      // a gets nothing, not even old pending messages
```

## Edge cases

- Publishing to an existing topic with no subscribers returns `0` and the message is gone for good; subscribing afterwards does not recover it.
- Publishing the same text twice produces two separate messages.
- `poll` for an unknown topic, or by someone not subscribed to it, returns an empty list (no error is raised).
- A list returned by `poll` is yours: later publishes or polls never change it.
- Several threads polling the *same* subscriber together must between them receive each message exactly once.
- 16 threads racing to create the same topic: exactly one gets `true`. Likewise for the same (topic, subscriber) subscription.

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `createTopic` returns `true` when the topic was created; `false` for a blank name or an existing topic.
- `subscribe` returns `true` when the subscription was added; `false` for an unknown topic, a blank subscriber or when the subscriber is already subscribed to that topic.
- `unsubscribe` returns `true` when a subscription was removed; `false` otherwise.
- `publish` returns the number of subscribers the message was delivered to (`0` is valid), or `-1` for an unknown topic.
- `poll(subscriber, topic, max)` returns a fresh list of at most `max` messages; it is empty for an unknown topic, a subscriber that is not subscribed, or `max <= 0` (and in the last case nothing is consumed).
- Messages are opaque strings; the same text published twice is two messages.

## Test entry point (Java)

The tests drive your system through the thin, primitive-typed contract below. It is **not** your domain
model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in
`solution/Solution.java`.

```java
package contract;

import java.util.List;

/** Thin test entry point. Your domain model lives behind it. Must be thread-safe. */
public interface PubSubApi {

    boolean createTopic(String topic);

    boolean subscribe(String topic, String subscriber);

    boolean unsubscribe(String topic, String subscriber);

    /** Number of subscribers the message was delivered to; -1 for an unknown topic. */
    int publish(String topic, String message);

    /** Up to max pending messages, oldest first, consumed by the call. */
    List<String> poll(String subscriber, String topic, int max);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    PubSubApi create()
}
```

## Language notes

All methods are called from multiple threads. `poll` must return a list the caller owns (see Edge cases).

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/22-pub-sub test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.
