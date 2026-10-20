# 104 · Quorum Key-Value Store (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 3 — infrastructure provided: a deterministic network and clock simulation (`SimNet`) runs your nodes; the protocol code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

We are building a small replicated key-value store in the style of Dynamo/Cassandra. Every node holds a full copy of the data and any node can accept a client request and act as its coordinator; there is no leader. A write is successful when a majority has stored it, and a read consults a majority, so a read always overlaps with the latest acknowledged write.

Build one node. Clients are asynchronous: they start a request with an id and poll for the reply. The network loses, duplicates and delays messages, and nodes crash and recover (keeping their memory). Your job is to make acknowledged writes visible to every later read, to repair replicas that fall behind, and to fail cleanly (not hang, not lie) when no majority is reachable.

## Requirements

1. `put(requestId, key, value)` and `get(requestId, key)` start a request on this node; `reply(requestId)` polls the result: PENDING, OK (with the value for a get), NOT_FOUND, or FAILED. A reply for an unknown id is PENDING.
2. A put gets the version (t, coordinatorId), where t is the last time passed to `onTick`. The higher version wins; equal t is decided by the larger id. A replica overwrites an entry only with a higher version, but acknowledges every write it receives (also an old one).
3. Put: the coordinator writes locally (one ack) and sends the write to all other nodes. The reply is OK as soon as a majority has acknowledged. Nodes that did not acknowledge are re-sent the write every 100 ms until all did or 1000 ms after the request started - also after the reply is already OK. No majority within 1000 ms: FAILED.
4. Get: the coordinator reads locally (one answer) and asks all others, re-asking silent ones every 100 ms. When a majority has answered, the reply is the entry with the highest version among the answers (OK), or NOT_FOUND if none has the key. No majority within 1000 ms: FAILED.
5. Read repair: every node that answered a get with an older entry (or none) is sent the winning entry as a write - also when its answer arrives after the reply was decided (until 1000 ms).
6. A request id is used once: starting a second request with a known id does nothing (client retries are safe) and does not change the stored reply.
7. `local(key)` exposes this replica's own copy (no coordination) so the tests can inspect replicas.

## Simulation rules and constraints

- Everything runs in a deterministic, single-threaded simulation (`SimNet`): the tests call `onMessage` / `onTick` on your object, never your own threads or timers. No locking is needed (the Rust traits take `&self`, so keep state in a `Mutex`).
- Time is simulated: one tick = 1 ms, `onTick(nowMs)` is called once per millisecond with the current time. Never read the wall clock, never sleep; derive every timeout from the `nowMs` you were handed.
- Network model: a message sent now is delivered after a latency of 1-5 ms by default (random per message, so messages can overtake each other). Tests can change the latency range, drop a fraction of the messages, duplicate a fraction, cut the network into partitions (messages across the cut are lost, including ones already in flight) and heal it.
- Crash model: a crashed node gets no ticks and no messages and its sends are ignored, but its memory is kept (it is paused, not wiped). After `recover` it simply continues with its first tick.
- Same seed, same run: any randomness of yours must come from the seed you are given, and iteration order of your collections must not influence behaviour (a replay is compared in the tests).
- A node that was crashed learns the time again with its first tick after recovery; versions use the last time passed to `onTick`, so two puts in the same millisecond on different coordinators get the same t and are ordered by node id.
- The tests inject: crashed replicas (including 2 of 5), a minority/majority partition, 40% message loss, 50% duplication, and random sequences of crashes, loss and partitions over many seeds, with up to 5 nodes.

## Examples

```
cluster n1 n2 n3 (W = R = 2)
t=0    n1.put(r1,"k","v1")  -> n1 stores (0,n1) + sends WRITE to n2, n3
t~4    n2 acks -> majority (n1+n2) -> reply(r1) = OK ;   n3 crashed: re-sent every 100 ms until t=1000
t=900  n3 recovers, receives the re-sent write, acks
t=950  n3.get(r2,"k") : n3 answers itself + n1/n2 -> highest version (0,n1) -> OK "v1"
```

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `Solution.node(id, cluster, send)` with `cluster` including this node; `Reply(status, value)` with `Status.{PENDING, OK, NOT_FOUND, FAILED}` and `value` only for a successful get. Rust: `create(id, cluster, send)`; `Reply::{Pending, PutOk, Value(String), NotFound, Failed}`.
- Quorum = majority of the cluster size (2 of 3, 3 of 5); the coordinator itself counts as one answer / ack.
- Version = (t, coordinatorId) with t = last `onTick` time; compare t first, then the id as a string. Acknowledge every received write, even a stale one, but only overwrite with a higher version.
- Timing: re-send/re-ask every 100 ms to silent nodes; the request is abandoned (FAILED if no majority yet) 1000 ms after it started; a put keeps re-sending to unacknowledged nodes until 1000 ms even after OK; read repair continues until 1000 ms.
- PENDING until a majority has answered (a request must stay PENDING before the deadline, not fail early) and FAILED within about 1100 ms when no majority is reachable. A failed put may still have reached some replicas.
- Idempotence: the second `put`/`get` with a used request id is ignored; replicas must also tolerate duplicated WRITE / READ messages and duplicate acks (an ack counted twice must not fake a majority).
- Guarantee the tests check: a read after an OK put (from any live coordinator) returns that value or a later one; a read of a never-written key is NOT_FOUND. The payload format is yours.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.Optional;

/**
 * Thin test entry point. One instance is ONE replica of a fully replicated key-value store (Dynamo style, leaderless,
 * N = cluster size, W = R = majority). The tests run the nodes inside a deterministic simulation (see
 * {@code testkit.SimNet}): they deliver messages with {@link #onMessage}, advance time with {@link #onTick} (once per
 * simulated millisecond), drop / duplicate / delay messages and crash nodes (a crashed node keeps its memory but sees
 * nothing). Nodes may only communicate through the {@code send} function they were created with. Single-threaded.
 *
 * <p>A node that was crashed learns the time again with its first tick after the recovery.
 *
 * <p>Any node can be the coordinator of a request; clients are asynchronous: they start a request with an id and poll
 * {@link #reply}. Rules:
 * <ul>
 *   <li><b>Versions.</b> A put gets the version (t, coordinatorId) where t is the last time passed to {@code onTick}.
 *       The higher version wins, equal t is decided by the larger id. A replica only overwrites an entry with a higher
 *       version and acknowledges every write it receives (also an old one).</li>
 *   <li><b>put.</b> The coordinator writes locally (that is one ack) and sends the write to all others. The reply is OK
 *       as soon as a majority acknowledged. Unacknowledged nodes are re-sent the write every 100 ms until all of them
 *       acknowledged or 1000 ms after the request started, also after the reply was already OK. No majority within
 *       1000 ms: FAILED (the write may still have reached some replicas).</li>
 *   <li><b>get.</b> The coordinator reads locally (one answer) and asks all others, re-asking silent ones every 100 ms.
 *       When a majority answered, the reply is the entry with the highest version among the answers (OK), or NOT_FOUND
 *       if none of them has the key. <b>Read repair:</b> every node that answered with an older entry (or none) gets the
 *       winning entry sent as a write, also if its answer arrives after the reply was decided (until 1000 ms).
 *       No majority within 1000 ms: FAILED.</li>
 *   <li>A request id is used once; starting a second request with a known id does nothing (client retries are safe).
 *       {@code reply} of an unknown id is PENDING.</li>
 * </ul>
 * The payload format is yours.
 */
public interface KvNode {

    void onMessage(String from, String payload);

    /** Called once per millisecond with the current simulated time (milliseconds since the start). */
    void onTick(long nowMs);

    void put(String requestId, String key, String value);

    void get(String requestId, String key);

    Reply reply(String requestId);

    /** This replica's own copy, for inspection (no coordination). */
    Optional<String> local(String key);
}

package contract;

/** Result of a client request. {@code value} is set only for a successful get. */
public record Reply(Status status, String value) {

    public enum Status {
        /** not finished yet */
        PENDING,
        /** put: stored on a majority; get: majority answered and a value exists */
        OK,
        /** get: a majority answered and none of them has the key */
        NOT_FOUND,
        /** a majority did not answer within 1000 ms of the request */
        FAILED
    }
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    /**
 * Wiring point between the test contract and YOUR design.
 * Create your own classes anywhere under src/main/java and return them from here.
 */
    public static KvNode node(String id, List<String> cluster, BiConsumer<String, String> send)
}
```

## What the tests check

These tests are black-box and deterministic: every node runs inside a seeded simulation (one tick = 1 ms of simulated time, nothing sleeps), messages can be delayed, dropped, duplicated and reordered, nodes can crash and recover, the network can be partitioned, and safety invariants are checked after EVERY simulated millisecond. A failing seed reproduces exactly.

- A write on one node is readable from all three; an unknown key is NOT_FOUND.
- With two of three nodes crashed, put stays PENDING for 500 ms and both put and get are FAILED within 1.1 s.
- One node down: put and get still succeed. A write acknowledged while n3 was down is read correctly from n3 after n3 recovered and n1 crashed (quorum overlap).
- Read repair: a stale or empty replica receives the winning value after a read, also when its answer comes in after the reply was decided.
- 40% message loss: the put still succeeds and the re-sends bring both other replicas up to date within the deadline.
- Partition 1 | 2: the lone node's put is FAILED, the majority side's put is OK; after healing a read through the old minority returns the majority's value.
- Newer writes win; two puts in the same millisecond on different coordinators converge on the larger node id on every replica.
- A reused request id is ignored; a reply for an unknown id is PENDING; duplicated messages are harmless.
- 100 random schedules (crashes, 10% loss, mixed puts and gets): every get after an OK put returns the last acknowledged value; a never-written key reads NOT_FOUND.
- More edge cases (5-node majority of three, rotating crashes and partitions, convergence after healing) in `interviewer/` (see `INTERVIEWER_NOTES.md`).

## Language notes

Use `Set<String>` of responders per request so a duplicated ack is counted once; compare versions with a small `record`/`Comparable` (time, then id). Keep requests in a `LinkedHashMap` so iteration is deterministic.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/104-quorum-kv test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`SimNet`, `FakeClock`, `Candidate`): `java/testkit`.
