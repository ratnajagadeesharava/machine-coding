# 99 · Raft Leader Election (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 3 — infrastructure provided: a deterministic network and clock simulation (`SimNet`) runs your nodes; the protocol code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

We are building a replicated service and, first, need the part that decides which node is in charge. Implement one node of a cluster that elects
a leader the way Raft does: nodes watch for a leader, start elections when it goes quiet, vote, and the winner keeps authority by
sending heartbeats. The cluster must never have two leaders in the same term, no matter how nodes crash and recover, how the network
is partitioned or how many messages are lost or delayed, and it must get a new leader quickly once things calm down. Your node
never sees real time or real sockets: a test harness drives it and the network.

## Requirements

1. A node is created with its id, the ids of its peers, a `send(to, payload)` function, and a random seed. It starts as a FOLLOWER in term 0.
2. A follower that has heard nothing from a leader (and granted no vote) for its election timeout becomes a CANDIDATE: term + 1, votes for itself, sends a vote request to every peer.
3. The election timeout is drawn uniformly from [150, 300] ms, again after every reset, from a random generator seeded with the given seed so runs are reproducible.
4. A node grants at most one vote per term (to the first asker, or again to the same candidate) and resets its election timer when it grants one.
5. A candidate holding votes from a majority of the whole cluster (itself included) becomes LEADER and sends a heartbeat to all peers immediately; a leader sends heartbeats every 50 ms.
6. A message with a higher term makes the receiver adopt that term and become FOLLOWER, forgetting its vote. A message with a lower term is answered with the receiver's term and otherwise ignored. A candidate that receives a heartbeat of its own term becomes FOLLOWER.
7. A heartbeat of the current term (or higher) resets the election timer and records the sender as the known leader.
8. Check-quorum: a leader that has not received heartbeat replies from a majority (itself included) within the last 300 ms steps down to FOLLOWER (same term) and starts a fresh election timer.
9. A cluster of one node elects itself as soon as its election timer fires.
10. `role()`, `term()` and `leader()` expose the state: `leader()` is the leader known in the current term (itself if LEADER, empty if unknown).

## Simulation rules and constraints

- Single-threaded and deterministic: one node object is driven only by `onMessage(from, payload)` and `onTick(nowMs)`. No locks, no threads, no real clock, no sleeping.
- Time is simulated: the harness (`SimNet`) advances 1 ms per step; in each step it first delivers the messages that are due, then ticks every running node. `onTick` receives the simulated time in ms since start.
- Network model: messages travel with a random latency of 1-5 ms by default (the hidden tests widen it to 1-40 ms), can be delayed, dropped (the tests use 20% loss), duplicated and therefore reordered, and are lost when sender or receiver is crashed or in different partitions.
- Faults the tests inject: node crash and recovery (a recovered node keeps its in-memory state object, the cluster simply goes on), partitions of the cluster into groups, healing, message loss, and 100 seeded random schedules of crash/recover/partition/heal.
- Safety is checked after every simulated millisecond: no two nodes may ever be LEADER in the same term. Liveness is checked with bounds (a leader within 1.5 s, a new one after a crash within 1.5 s, convergence after faults stop).
- Randomness may come only from your own generator seeded with the given seed; the same seed must replay the same run. The payload string format is yours; nodes only talk through `send`.

## Examples

```
5 nodes n0..n4, every node a FOLLOWER in term 0.
t~200: n3's timer fires -> CANDIDATE term 1, asks n0 n1 n2 n4 for votes
n0, n1, n2 grant (first asker in term 1) -> n3 has 4 of 5 votes -> LEADER, heartbeat to all
every 50 ms: heartbeat -> followers reset their timers, leader() == "n3", term() == 1 everywhere

crash n3 -> ~150-300 ms later n1 becomes CANDIDATE term 2, wins -> LEADER term 2
recover n3 -> it is still LEADER term 1 until it hears term 2 -> FOLLOWER of n1
```

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `ElectionNode` with `onMessage(String from, String payload)`, `onTick(long nowMs)`, `role()`, `term()`, `leader()` (`Optional<String>`), factory `Solution.create(id, peers, BiConsumer<String,String> send, long seed)`. Rust: `on_message(&self, &str, &str)`, `on_tick(&self, u64)`, `role()`, `term()`, `leader() -> Option<String>`, `Role::{Follower, Candidate, Leader}`, `create(id, &[&str], SendFn, seed: u64)`; state behind interior mutability (the trait takes `&self`).
- Majority is computed over the whole cluster (peers + self); a lone survivor of five can never win, and a partitioned minority leader must stop being leader (via check-quorum), though it keeps its term.
- A vote is granted at most once per term; granting resets the election timer. A node must forget its vote when it adopts a higher term.
- Lower-term messages never change the receiver's role or timer; the sender learns the receiver's higher term from the reply and steps down.
- A leader's check-quorum window is 300 ms; with replies arriving, a healthy leader keeps its term indefinitely (no unnecessary elections: the same leader and term after 10,000 ms of quiet).
- A recovered old leader must end up FOLLOWER of the new leader, with `leader()` equal to it, within 600 ms of recovery.
- After a heal, the cluster converges to exactly one leader and every node reports that leader and the same term.
- A one-node cluster elects itself without sending anything and reports `leader() == itself`.
- The harness derives seeds from node ids; do not depend on identical seeds across nodes. If all nodes had the same seed they would time out together and livelock.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.Optional;

/**
 * Thin test entry point. One instance is ONE node of a cluster; the tests run the nodes inside a deterministic
 * simulation (see {@code testkit.SimNet}): they deliver messages with {@link #onMessage}, advance time with
 * {@link #onTick} (once per simulated millisecond) and read the state through the getters. Nodes may only
 * communicate through the {@code send} function they were created with. Single-threaded: no locking needed.
 *
 * <p>The protocol is the election half of Raft:
 * <ul>
 *   <li>Every node starts as a FOLLOWER in term 0. A follower that has heard nothing from a leader (or granted a vote)
 *       for its election timeout becomes a CANDIDATE: term + 1, votes for itself, asks every peer for its vote.
 *       The election timeout is drawn uniformly from [150, 300] ms, again after every reset, from a random number
 *       generator seeded with the seed given to the factory (so a run is reproducible).</li>
 *   <li>A node grants at most one vote per term (to the first asker, or again to the same candidate) and resets its
 *       election timer when it does. A candidate with votes from a majority of the whole cluster (itself included)
 *       becomes LEADER and sends a heartbeat at once; a leader sends heartbeats to all peers every 50 ms.</li>
 *   <li>Any message carrying a higher term makes the receiver adopt that term and become FOLLOWER (forgetting its vote).
 *       Messages carrying a lower term are answered with the receiver's term and otherwise ignored. A candidate that
 *       receives a heartbeat of its own term becomes FOLLOWER. A heartbeat of the current term (or higher) resets the
 *       election timer and records the sender as the known leader.</li>
 *   <li>Check-quorum: a leader that has not got heartbeat replies from a majority of the cluster (itself included) within
 *       the last 300 ms steps down to FOLLOWER (same term) and starts a fresh election timer.</li>
 *   <li>A cluster of one node elects itself as soon as its election timer fires.</li>
 * </ul>
 * The payload format is yours.
 */
public interface ElectionNode {

    void onMessage(String from, String payload);

    /** Called once per millisecond with the current simulated time (milliseconds since the start). */
    void onTick(long nowMs);

    Role role();

    long term();

    /** The leader this node currently knows in its current term: itself if it is the leader, empty if unknown. */
    Optional<String> leader();
}

package contract;

/** Provided. */
public enum Role {
    FOLLOWER,
    CANDIDATE,
    LEADER
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    /**
 * Wiring point between the test contract and YOUR design.
 * Create your own classes anywhere under src/main/java and return them from here.
 */
public final class Solution {
    private Solution() {}

    /**
     * @param id    this node's name
     * @param peers the OTHER nodes' names (the cluster has peers.size() + 1 nodes)
     * @param send  {@code send.accept(to, payload)} delivers a message to another node (later, maybe never)
     * @param seed  seed for the election-timeout random numbers
     */
    public static ElectionNode create(String id, List<String> peers, BiConsumer<String, String> send, long seed)
}
```

## What the tests check

These tests are black-box and deterministic: every node runs inside a seeded simulation (one tick = 1 ms of simulated time, nothing sleeps), messages can be delayed, dropped, duplicated and reordered, nodes can crash and recover, the network can be partitioned, and safety invariants are checked after EVERY simulated millisecond. A failing seed reproduces exactly.

- Clusters of 3, 5 and 7 elect exactly one leader within 1.5 s; all nodes agree on leader and term after another 500 ms.
- A single-node cluster elects itself within 600 ms.
- Stability: 10,000 ms of quiet changes neither the leader nor the term.
- Leader crash: a different leader with a higher term appears within 1.5 s; the recovered old leader steps down to FOLLOWER of the new leader within 600 ms.
- No quorum: with four of five nodes crashed no leader exists after 3 s.
- Partition: the majority side elects exactly one leader; nodes in the minority end up not LEADER; after healing there is one leader and one term cluster-wide within 2 s.
- 20% message loss still yields a leader within 4 s and keeps safety for 3 s more.
- Replay: the same seed produces the same leader and term at 3 s.
- **100 random fault schedules** (crash, recover, isolate a node, heal at random, 12 rounds each): never two leaders in one term, and the cluster settles after faults end.
- More edge cases in `interviewer/` (see `INTERVIEWER_NOTES.md`): 11 nodes with 1-40 ms latency, and the leader crashing ten times in a row.

## Language notes

Use a `java.util.Random(seed)` held by the node, never `Math.random()`; encode payloads however you like (for example `"VOTE_REQ|term"`). The harness calls you from a single thread, so plain fields suffice.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/99-leader-election test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`SimNet`, `FakeClock`, `Candidate`): `java/testkit`.
