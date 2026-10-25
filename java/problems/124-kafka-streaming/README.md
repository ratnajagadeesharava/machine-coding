# 124 · Kafka-Like Message Streaming Service (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

An in-memory message streaming service in the style of Kafka. Producers publish messages to named **topics**; each topic is split into numbered **partitions**, and every partition is an append-only log in which a message is identified by its **offset** (0, 1, 2, ...). Consumers belong to **consumer groups**: the partitions of a topic are shared out among the consumers of a group, each consumer reads its own partitions, and the group remembers (**commits**) how far it has read. Different groups read the same topic independently and never affect each other. A topic can optionally keep only the newest N messages per partition.

## Requirements

1. `createTopic(topic, partitions)` creates a topic with `partitions >= 1` and unlimited retention; `createTopic(topic, partitions, maxMessagesPerPartition)` keeps at most `maxMessagesPerPartition >= 1` messages in each partition. A blank/null name, a name that already exists, `partitions < 1` or `maxMessagesPerPartition < 1` is an error.
2. `publish(topic, key, message)` appends the message to one partition and returns the string `"partition:offset"` (for example `"2:17"`). The message may be empty but not null; the key may be null.
   - **Non-null key** (the empty string is a key): `h = 0; for each UTF-16 code unit c of the key: h = 31*h + c` using 32-bit signed integer arithmetic where overflow wraps around. The partition is `floorMod(h, partitions)`, so it is never negative. Equal keys always go to the same partition, in publish order.
   - **Null key**: round-robin per topic. The topic has a counter starting at 0; a key-less publish goes to partition `counter` and then the counter becomes `(counter + 1) mod partitions`. Keyed publishes do not touch the counter. Counters of different topics are independent.
3. Offsets are per partition, start at 0, and have no gaps: the n-th message ever appended to a partition (0-based) has offset n, even when older messages are later dropped by retention. Order within a partition is publish order.
4. **Consumer groups.** `joinGroup(group, consumer, topic)` and `leaveGroup(group, consumer, topic)` change a group's membership for one topic (a group may consume several topics, each with its own membership and assignment). Joining when already a member is a no-op. Leaving when not a member is an error.
5. **Range assignment**, recomputed from scratch on every join and leave. With the consumers sorted by id (ordinary string order, e.g. `"c1" < "c10" < "c2"`), `C` consumers and `P` partitions: every consumer gets `P / C` (integer division) partitions, and the first `P mod C` consumers get one extra. The partitions are handed out as contiguous ascending ranges in consumer order. A consumer may receive nothing when `C > P`. `assignment(group, consumer, topic)` returns the consumer's partitions in ascending order; a consumer that is not a member is an error.
6. `poll(group, consumer, topic, max)` with `max >= 1` returns up to `max` messages from the consumer's assigned partitions, in partition order and then offset order. For each partition it starts at `max(committed offset of the group, earliest offset)` and goes to the end of the log. Each result is the string `"partition:offset:message"` (the message is everything after the second colon and may itself contain colons). Polling never changes any committed offset, so polling twice without committing returns the same messages (plus any newly published ones).
7. `commit(group, consumer, topic, partition, nextOffset)` sets the group's committed offset of that partition (the next offset to read). The caller must be a member of the group and the partition must currently be assigned to the caller. `nextOffset` must be `>=` the group's current committed offset (committing the same value is fine) and `<= latestOffset` of the partition (committing exactly the log end is fine). Anything else is an error and changes nothing.
8. `committedOffset(group, topic, partition)` returns the group's committed offset, `0` if the group never committed. Committed offsets belong to the group (per topic and partition): they survive rebalances, and survive every member leaving and others joining later. They are not rewritten by retention.
9. **Retention.** When a publish makes a partition hold more than `maxMessagesPerPartition` messages, the oldest message of that partition is dropped. `earliestOffset(topic, partition)` is the offset of the oldest retained message (equal to `latestOffset` when the partition is empty); `latestOffset(topic, partition)` is the offset the next message will get (the log end; 0 for a never-written partition). `messageCount(topic)` is the number of retained messages over all partitions. A group whose committed offset is below `earliestOffset` simply starts reading at `earliestOffset`.
10. Errors. Blank/null names, an unknown topic, a duplicate topic, a null message, `max < 1` in `poll`, and a partition outside `0..partitions-1` are `IllegalArgumentException`. A consumer that is not a member (assignment, poll, commit, leave), a partition that is not assigned to the committing consumer, and an invalid commit offset are `IllegalStateException`. A failed call leaves all state unchanged (a rejected publish does not consume a round-robin slot).
11. The service is used from many threads at once. Concurrent publishers must receive unique, gap-free offsets per partition, round-robin must stay exact, and every read must see a consistent state (never half of a rebalance or half of a retention drop).

## Behavioral contract (entry point)

Anything not stated is yours to decide (and to state as an assumption).

```java
package contract;

import java.util.List;

public interface StreamingService {
    void createTopic(String topic, int partitions);
    void createTopic(String topic, int partitions, int maxMessagesPerPartition);
    String publish(String topic, String key, String message);          // "partition:offset"
    void joinGroup(String group, String consumer, String topic);
    void leaveGroup(String group, String consumer, String topic);
    List<Integer> assignment(String group, String consumer, String topic);   // ascending
    List<String> poll(String group, String consumer, String topic, int max); // "partition:offset:message"
    void commit(String group, String consumer, String topic, int partition, long nextOffset);
    long committedOffset(String group, String topic, int partition);
    long earliestOffset(String topic, int partition);
    long latestOffset(String topic, int partition);
    long messageCount(String topic);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static StreamingService create()
}
```

Order of checks when several things are wrong: blank/null names, then unknown topic, then other argument problems (partition range, `max`, null message), then membership, then assignment, then the offset rule.

## Examples

```
createTopic("t", 3)
publish("t", "a", "m1")   -> "1:0"     ("a" = 97, 97 mod 3 = 1)
publish("t", "b", "m2")   -> "2:0"     (98 mod 3 = 2)
publish("t", "a", "m3")   -> "1:1"     (same key, same partition, next offset)
publish("t", "c", "m4")   -> "0:0"     (99 mod 3 = 0)
publish("t", null, "m5")  -> "0:1"     (first key-less publish: counter 0 -> partition 0)
publish("t", null, "m6")  -> "1:2"     (counter 1)
publish("t", "ab", "m7")  -> "0:2"     ("ab" = 97*31 + 98 = 3105, 3105 mod 3 = 0; the counter is untouched)
publish("t", null, "m8")  -> "2:1"     (counter 2)
```

```
createTopic("t", 5)
joinGroup("g", "c1", "t");  assignment c1 -> [0,1,2,3,4]
joinGroup("g", "c2", "t");  assignment c1 -> [0,1,2]   c2 -> [3,4]
joinGroup("g", "c3", "t");  assignment c1 -> [0,1]     c2 -> [2,3]   c3 -> [4]
leaveGroup("g", "c1", "t"); assignment c2 -> [0,1,2]   c3 -> [3,4]
```

```
createTopic("t", 2);  joinGroup("g1","c","t");  joinGroup("g2","c","t")   (c is alone: it owns partitions 0 and 1)
publish x6 key-less: m0..m5   (partition 0 gets m0 m2 m4, partition 1 gets m1 m3 m5)
poll("g1","c","t",4)          -> ["0:0:m0","0:1:m2","0:2:m4","1:0:m1"]
poll("g1","c","t",4)          -> same four (poll does not commit)
commit("g1","c","t",0,3);  commit("g1","c","t",1,1)
poll("g1","c","t",10)         -> ["1:1:m3","1:2:m5"]
poll("g2","c","t",10)         -> all six, g2 has committed nothing
commit("g1","c","t",0,2)      -> IllegalStateException (backwards)
commit("g1","c","t",0,4)      -> IllegalStateException (log end of partition 0 is 3)
```

```
createTopic("r", 1, 3);  publish 5 key-less messages m0..m4 -> "0:0" ... "0:4"
earliestOffset -> 2, latestOffset -> 5, messageCount -> 3
a group with committed offset 0 polls: ["0:2:m2","0:3:m3","0:4:m4"]   (jumped to the earliest retained offset)
committedOffset is still 0 until the group commits again
```

### Edge cases

- Hash overflow: keys such as `"polygenelubricants"` hash to `Integer.MIN_VALUE`; the partition must still be non-negative (`floorMod`, not `abs`/`%`). Characters outside the BMP count as two UTF-16 code units.
- The empty key is a key (hash 0, partition 0); a null key is key-less.
- Key-less and keyed publishes mix freely; only key-less ones move the round-robin counter.
- Consumer ids are ordered as plain strings, not by join time or numerically.
- A consumer with an empty assignment polls an empty list and may not commit anything (`IllegalStateException`).
- After a rebalance a consumer that lost a partition can no longer commit it; the new owner continues from the group's committed offset.
- `poll` with a committed offset equal to the log end returns an empty list; messages published later appear on the next poll.
- A consumer can join the same group for several topics; the memberships do not interact.
- Retention is per partition; one busy partition does not push out messages of another.

## Glossary

- **Topic**: a named stream of messages made of one or more partitions.
- **Partition**: an ordered, append-only log inside a topic.
- **Offset**: the position of a message inside one partition, starting at 0.
- **Log end**: the offset the next message of a partition will get.
- **Consumer group**: consumers that share the partitions of a topic and share one set of committed offsets.
- **Committed offset**: the next offset a group will read from a partition.
- **Rebalance**: recomputing which consumer owns which partition after a join or leave.
- **Retention**: the limit on how many messages a partition keeps.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/124-kafka-streaming test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`Candidate`, `Concurrently`): `java/testkit`.
