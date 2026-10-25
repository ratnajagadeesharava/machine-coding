# 124 · Kafka-Like Message Streaming Service (Rust)

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

1. `create_topic(topic, partitions, max_messages_per_partition)` creates a topic with `partitions >= 1`. `None` means unlimited retention; `Some(n)` with `n >= 1` keeps at most `n` messages in each partition. A blank name, a name that already exists, `partitions == 0` or `Some(0)` is an error.
2. `publish(topic, key, message)` appends the message to one partition and returns the string `"partition:offset"` (for example `"2:17"`). The message may be empty; the key is an `Option` (`None` means key-less).
   - **`Some(key)`** (the empty string is a key): `h = 0; for each UTF-16 code unit c of the key: h = 31*h + c` using 32-bit signed integer arithmetic where overflow wraps around. The partition is `h.rem_euclid(partitions)`, so it is never negative. Equal keys always go to the same partition, in publish order.
   - **No key (`None`)**: round-robin per topic. The topic has a counter starting at 0; a key-less publish goes to partition `counter` and then the counter becomes `(counter + 1) mod partitions`. Keyed publishes do not touch the counter. Counters of different topics are independent.
3. Offsets are per partition, start at 0, and have no gaps: the n-th message ever appended to a partition (0-based) has offset n, even when older messages are later dropped by retention. Order within a partition is publish order.
4. **Consumer groups.** `join_group(group, consumer, topic)` and `leave_group(group, consumer, topic)` change a group's membership for one topic (a group may consume several topics, each with its own membership and assignment). Joining when already a member is a no-op. Leaving when not a member is an error.
5. **Range assignment**, recomputed from scratch on every join and leave. With the consumers sorted by id (ordinary string order, e.g. `"c1" < "c10" < "c2"`), `C` consumers and `P` partitions: every consumer gets `P / C` (integer division) partitions, and the first `P mod C` consumers get one extra. The partitions are handed out as contiguous ascending ranges in consumer order. A consumer may receive nothing when `C > P`. `assignment(group, consumer, topic)` returns the consumer's partitions in ascending order; a consumer that is not a member is an error.
6. `poll(group, consumer, topic, max)` with `max >= 1` returns up to `max` messages from the consumer's assigned partitions, in partition order and then offset order. For each partition it starts at `max(committed offset of the group, earliest offset)` and goes to the end of the log. Each result is the string `"partition:offset:message"` (the message is everything after the second colon and may itself contain colons). Polling never changes any committed offset, so polling twice without committing returns the same messages (plus any newly published ones).
7. `commit(group, consumer, topic, partition, next_offset)` sets the group's committed offset of that partition (the next offset to read). The caller must be a member of the group and the partition must currently be assigned to the caller. `next_offset` must be `>=` the group's current committed offset (committing the same value is fine) and `<= latest_offset` of the partition (committing exactly the log end is fine). Anything else is an error and changes nothing.
8. `committed_offset(group, topic, partition)` returns the group's committed offset, `0` if the group never committed. Committed offsets belong to the group (per topic and partition): they survive rebalances, and survive every member leaving and others joining later. They are not rewritten by retention.
9. **Retention.** When a publish makes a partition hold more than `maxMessagesPerPartition` messages, the oldest message of that partition is dropped. `earliest_offset(topic, partition)` is the offset of the oldest retained message (equal to `latest_offset` when the partition is empty); `latest_offset(topic, partition)` is the offset the next message will get (the log end; 0 for a never-written partition). `message_count(topic)` is the number of retained messages over all partitions. A group whose committed offset is below `earliest_offset` simply starts reading at `earliest_offset`.
10. Errors. Every operation returns `Result<_, StreamError>`: `InvalidArgument` for blank names, `partitions == 0`, `Some(0)` retention, `max == 0` in `poll` and a partition outside `0..partitions-1`; `UnknownTopic`; `TopicExists`; `NotMember` for a consumer that is not a member (assignment, poll, commit, leave); `NotAssigned` for a commit of a partition the caller does not own; `InvalidOffset` for a commit below the committed offset or past the log end. A failed call leaves all state unchanged (a rejected publish does not consume a round-robin slot).
11. The service is used from many threads at once. Concurrent publishers must receive unique, gap-free offsets per partition, round-robin must stay exact, and every read must see a consistent state (never half of a rebalance or half of a retention drop).

## Behavioral contract (entry point)

Anything not stated is yours to decide (and to state as an assumption).

```rust
#[derive(Debug, PartialEq, Eq)]
pub enum StreamError { UnknownTopic, TopicExists, InvalidArgument, NotMember, NotAssigned, InvalidOffset }

pub trait StreamingService: Send + Sync {
    fn create_topic(&self, topic: &str, partitions: usize, max_messages_per_partition: Option<usize>) -> Result<(), StreamError>;
    fn publish(&self, topic: &str, key: Option<&str>, message: &str) -> Result<String, StreamError>; // "partition:offset"
    fn join_group(&self, group: &str, consumer: &str, topic: &str) -> Result<(), StreamError>;
    fn leave_group(&self, group: &str, consumer: &str, topic: &str) -> Result<(), StreamError>;
    fn assignment(&self, group: &str, consumer: &str, topic: &str) -> Result<Vec<usize>, StreamError>; // ascending
    fn poll(&self, group: &str, consumer: &str, topic: &str, max: usize) -> Result<Vec<String>, StreamError>; // "partition:offset:message"
    fn commit(&self, group: &str, consumer: &str, topic: &str, partition: usize, next_offset: u64) -> Result<(), StreamError>;
    fn committed_offset(&self, group: &str, topic: &str, partition: usize) -> Result<u64, StreamError>;
    fn earliest_offset(&self, topic: &str, partition: usize) -> Result<u64, StreamError>;
    fn latest_offset(&self, topic: &str, partition: usize) -> Result<u64, StreamError>;
    fn message_count(&self, topic: &str) -> Result<u64, StreamError>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create() -> Option<Arc<dyn StreamingService>>
```

Order of checks when several things are wrong: blank/null names, then unknown topic, then other argument problems (partition range, `max`), then membership, then assignment, then the offset rule.

## Examples

```
create_topic("t", 3, None)
publish("t", Some("a"), "m1")   -> "1:0"     ("a" = 97, 97 mod 3 = 1)
publish("t", Some("b"), "m2")   -> "2:0"     (98 mod 3 = 2)
publish("t", Some("a"), "m3")   -> "1:1"     (same key, same partition, next offset)
publish("t", Some("c"), "m4")   -> "0:0"     (99 mod 3 = 0)
publish("t", None, "m5")  -> "0:1"     (first key-less publish: counter 0 -> partition 0)
publish("t", None, "m6")  -> "1:2"     (counter 1)
publish("t", Some("ab"), "m7")  -> "0:2"     ("ab" = 97*31 + 98 = 3105, 3105 mod 3 = 0; the counter is untouched)
publish("t", None, "m8")  -> "2:1"     (counter 2)
```

```
create_topic("t", 5, None)
join_group("g", "c1", "t");  assignment c1 -> [0,1,2,3,4]
join_group("g", "c2", "t");  assignment c1 -> [0,1,2]   c2 -> [3,4]
join_group("g", "c3", "t");  assignment c1 -> [0,1]     c2 -> [2,3]   c3 -> [4]
leave_group("g", "c1", "t"); assignment c2 -> [0,1,2]   c3 -> [3,4]
```

```
create_topic("t", 2, None);  join_group("g1","c","t");  join_group("g2","c","t")   (c is alone: it owns partitions 0 and 1)
publish x6 key-less: m0..m5   (partition 0 gets m0 m2 m4, partition 1 gets m1 m3 m5)
poll("g1","c","t",4)          -> ["0:0:m0","0:1:m2","0:2:m4","1:0:m1"]
poll("g1","c","t",4)          -> same four (poll does not commit)
commit("g1","c","t",0,3);  commit("g1","c","t",1,1)
poll("g1","c","t",10)         -> ["1:1:m3","1:2:m5"]
poll("g2","c","t",10)         -> all six, g2 has committed nothing
commit("g1","c","t",0,2)      -> Err(InvalidOffset)   (backwards)
commit("g1","c","t",0,4)      -> Err(InvalidOffset)   (log end of partition 0 is 3)
```

```
create_topic("r", 1, Some(3));  publish 5 key-less messages m0..m4 -> "0:0" ... "0:4"
earliest_offset -> 2, latest_offset -> 5, message_count -> 3
a group with committed offset 0 polls: ["0:2:m2","0:3:m3","0:4:m4"]   (jumped to the earliest retained offset)
committed_offset is still 0 until the group commits again
```

### Edge cases

- Hash overflow: keys such as `"polygenelubricants"` hash to `i32::MIN`; the partition must still be non-negative (`rem_euclid`, not `abs`/`%`), and the arithmetic must wrap rather than panic. Characters outside the BMP count as two UTF-16 code units.
- The empty key `Some("")` is a key (hash 0, partition 0); `None` is key-less.
- Key-less and keyed publishes mix freely; only key-less ones move the round-robin counter.
- Consumer ids are ordered as plain strings, not by join time or numerically.
- A consumer with an empty assignment polls an empty list and may not commit anything (`NotAssigned`).
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
cargo test -p p124_kafka_streaming
STRICT=1 cargo test -p p124_kafka_streaming   # fail instead of skip while unimplemented
cargo test -p p124_kafka_streaming --features hidden   # interviewer scenarios
cargo play run 124              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`FakeClock`, `sut!`, `run_concurrently`): `rust/support`.
