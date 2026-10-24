# Hints — 124 Kafka-Like Message Streaming Service (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

List the things that have a lifetime of their own: what survives when a consumer leaves, and what disappears with it? Which of these facts are about the topic, and which are about a group reading the topic?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Keep the log of a partition separate from any reader state, and treat the assignment as something you can always recompute from the sorted list of members rather than patch incrementally. For retention, track the offset of the first retained message instead of renumbering anything. Decide which operations must be atomic together (allocating an offset and appending) and which can work on a consistent copy.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `Topic { partitions[], roundRobinCounter, maxPerPartition }`, `Partition { deque of messages, nextOffset }` with `earliest = nextOffset - size`; `GroupTopic { sorted members, assignment map, committed offsets per partition }` keyed by (group, topic). `publish` locks one partition (and the counter for key-less messages), `join`/`leave` recompute the assignment from the sorted members, `poll` reads the assignment and then each owned partition from `max(committed, earliest)`.

**Java:** `ConcurrentHashMap` for topics and group states, `ArrayDeque` plus `synchronized`/`ReentrantLock` per partition, `TreeSet<String>` for members, and always take the group lock before a partition lock (never the reverse) so poll/commit cannot deadlock with publish.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
