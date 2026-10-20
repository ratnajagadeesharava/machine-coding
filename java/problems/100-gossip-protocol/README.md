# 100 · Gossip Protocol with Failure Detection (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 3 — infrastructure provided: a deterministic network and clock simulation (`SimNet`) runs your nodes; the protocol code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Our cluster has no coordinator: any of a few dozen nodes may take a write, and every node must eventually see every write and know which of its peers are still alive. Broadcasting every update to everybody does not scale and a central membership service would be a single point of failure, so the nodes talk to a couple of random peers at a time (gossip) and piggy-back liveness information on the same traffic.

Build one gossip node. It stores a small key/value map, spreads it to the other nodes, and keeps a local opinion of who is alive. Your node is tested inside a simulated network where messages are lost, duplicated, delayed and reordered, nodes crash and recover, and the network splits in two and heals again.

**In short:** one node object with `put`, `get` and `aliveMembers`. Each node periodically sends its whole state to two random peers, merges what it receives, and decides who is alive purely from heartbeat counters that arrive inside that state. The tests look at the data every node ends up with, at who each node believes is alive, and at how many messages each node sent.

## Requirements

1. `put(key, value)` writes locally with version = (highest version this node knows for that key) + 1, tagged with the writer's id. `get(key)` returns the winning value, or empty.
2. The winner for a key is the entry with the higher version; equal versions are decided by the larger writer id. Every node must converge to the same winner whatever the delivery order.
3. Every 100 ms a node starts a gossip round: it increments its own heartbeat counter, picks up to 2 distinct random peers and sends each its whole state. The first round happens at a random moment within the first 100 ms (drawn from the node's seeded generator).
4. A node that receives such a push merges it and answers with its own whole state. The answer is not answered again. A node never broadcasts to all peers.
5. Heartbeat counters travel with the state and are merged by maximum. A peer is alive while its counter increased within the last 2000 ms of the observer's own time; at the start every peer gets the benefit of the doubt (as if its counter had just increased).
6. A peer that shows progress again becomes alive again (recovery, healed partition). `aliveMembers()` is the node itself plus all peers currently believed alive.

## Simulation rules and constraints

- Everything runs in a deterministic, single-threaded simulation (`SimNet`): the tests call `onMessage` / `onTick` on your object, never your own threads or timers. The Rust traits take `&self`.
- Time is simulated: one tick = 1 ms, `onTick(nowMs)` is called once per millisecond with the current time. Never read the wall clock, never sleep; derive every timeout from the `nowMs` you were handed.
- Network model: a message sent now is delivered after a latency of 1-5 ms by default (random per message, so messages can overtake each other). Tests can change the latency range, drop a fraction of the messages, duplicate a fraction, cut the network into partitions (messages across the cut are lost, including ones already in flight) and heal it.
- Crash model: a crashed node gets no ticks and no messages and its sends are ignored, but its memory is kept (it is paused, not wiped). After `recover` it simply continues with its first tick.
- Same seed, same run: any randomness of yours must come from the seed you are given, and iteration order of your collections must not influence behaviour (a replay is compared in the tests).
- The tests inject: message loss (up to 30%), duplication (50%) combined with latencies up to 120 ms, node crashes and recoveries, and a network partition in two halves followed by a heal.

## Examples

```
t=0     n0.put("color","blue")           -> n0 knows (blue, version 1, writer n0)
t~40    n0 pushes its state to n7, n3    -> both merge it, both answer with their own state
t~140   n7 and n3 push to 2 peers each   -> 1 -> 3 -> ~7 -> ~15 nodes: O(log N) rounds
t=1000  n3.crash();  t=3000+: every other node's aliveMembers() no longer contains n3
```

Versions and winners:

```
n0.put("k","a")                              -> n0 holds ("a", version 1, writer n0)
n0 merges a state holding ("b", version 1, writer n4)
                                             -> same version, n4 > n0, so "b" wins: n0.get("k") == "b"
n0.put("k","c")                              -> version = highest known (1) + 1 = 2, writer n0: beats both
every node that later merges ("c", 2, n0)    -> get("k") == "c"; an old ("a", 1, n0) arriving later never brings "a" back
```

One push-pull exchange (n0 gossips to n3):

```
n0 -> n3 : its whole state                  (n0 has just incremented its own heartbeat counter)
n3 merges, then n3 -> n0 : its whole state  (the answer)
n0 merges the answer and does NOT reply     (so a round costs 2 messages per chosen peer, never more)
```

Failure detection, seen from n0 (a peer is "alive" while its counter keeps increasing):

```
t=0      all peers are alive (benefit of the doubt)
t=1000   n0 learns that n5's counter is now 10 (directly from n5, or relayed in n2's state)  -> n5 alive
         ... n0 never sees a counter above 10 for n5 again (n5 crashed)
t=2500   still alive: less than 2000 ms since the counter last increased
t=3500   not alive: aliveMembers() no longer contains n5
t=6000   n5 recovered and gossips again, n0 sees counter 31 -> n5 is alive again
```

## Edge cases

- `get` of a key nobody wrote is empty. `put` on a node that has never heard of the key gets version 1.
- A node with 0 or 1 peers still works: it sends to as many distinct peers as exist (up to 2). A node never sends to itself and never picks the same peer twice in one round.
- A received state with a lower version for a key, or a lower heartbeat counter for a node, changes nothing (counters only go up, versions only go up). Receiving the same state many times is harmless.
- Liveness evidence may arrive indirectly: if n2's state says n5's counter is 12 and you knew 10, then n5 made progress, even though you never heard from n5.
- `aliveMembers()` always contains the node itself, even when it has been cut off from everybody.
- During a 4+4 partition each side stops seeing the other side's counters; after about 2 seconds each node reports only its own side as alive. After the heal the counters flow again and everybody reports all 8.
- The answer to a push is not answered again; if it were, two nodes would exchange messages forever.
- Messages may be duplicated or arrive out of order; the final data must not depend on arrival order.

## Glossary

- **Gossip round**: every 100 ms a node picks a few random peers and sends them its state; information spreads like a rumour, in about log(N) rounds.
- **Fan-out**: how many peers are contacted per round (here up to 2).
- **Push-pull**: the sender pushes its state and the receiver pulls the sender up to date by answering with its own state.
- **Heartbeat counter**: a number each node increments once per round; peers conclude "alive" when they see a higher number than before.
- **Version / writer id**: the pair that orders competing writes of the same key: higher version first, then the larger writer id (string comparison).
- **Converge**: after enough rounds without new writes, every node holds the same winner for every key.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `GossipNode` created by `Solution.create(id, peers, send, seed)`; `peers` are the OTHER nodes, `send.accept(to, payload)`. Rust: `create(id, peers, send, seed)` with `&[&str]`, `get -> Option<String>`, `alive_members() -> Vec<String>` (sorted).
- Round period 100 ms; fan-out up to 2 distinct random peers per round (fewer if there are fewer peers); first round at a random time within the first 100 ms, from the node's own seeded generator.
- A push is answered once with the receiver's full state; the answer is never answered (otherwise traffic would ping-pong forever).
- `put` version = highest known version for the key + 1, so a later write that has seen the earlier one always wins; concurrent writes with equal versions are decided by the larger writer id (string comparison).
- Heartbeats are merged by maximum. Alive means: own id, or the peer's counter increased within the last 2000 ms of the local `nowMs`. The initial benefit of the doubt starts at the node's first view of the time.
- The tests count messages per sender (`sentBy`): a 32-node cluster must send fewer than 250 messages per node in 30 rounds (a broadcast would be about 930).
- The payload format is yours. The run must be reproducible from the seed.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.Optional;
import java.util.Set;

/**
 * Thin test entry point. One instance is ONE node; the tests run the nodes inside a deterministic simulation (see
 * {@code testkit.SimNet}): they deliver messages with {@link #onMessage}, advance time with {@link #onTick} (once per
 * simulated millisecond) and read state through the getters. Nodes may only communicate through the {@code send}
 * function they were created with. Single-threaded: no locking needed.
 *
 * <p>Protocol (push-pull gossip with heartbeat failure detection):
 * <ul>
 *   <li><b>Rounds.</b> Every 100 ms (the first round at a random moment within the first 100 ms, drawn from the node's
 *       own seeded random generator) a node increments its own heartbeat counter, picks up to 2 distinct random peers
 *       and sends each of them its whole state. A node that receives such a push merges it and answers with its own
 *       whole state (the answer is not answered again). That is the only traffic: a node never broadcasts to everybody.</li>
 *   <li><b>Data.</b> {@link #put} writes a key locally with version = (highest version this node knows for the key) + 1,
 *       tagged with the writer's id. The winner for a key is the entry with the higher version; equal versions are
 *       decided by the larger writer id. Merging is therefore commutative, associative and idempotent, so every node
 *       ends with the same value once the gossip has spread.</li>
 *   <li><b>Failure detection.</b> Heartbeat counters travel with the state and are merged by maximum. A node counts a
 *       peer as alive as long as the peer's counter has increased within the last 2000 ms of this node's own time; at
 *       the start every peer is given the benefit of the doubt (as if its counter had just increased). A peer that
 *       shows progress again becomes alive again.</li>
 * </ul>
 * The payload format is yours.
 */
public interface GossipNode {

    void onMessage(String from, String payload);

    /** Called once per millisecond with the current simulated time (milliseconds since the start). */
    void onTick(long nowMs);

    /** Writes locally; the value spreads by gossip. */
    void put(String key, String value);

    /** The value this node currently believes wins for the key. */
    Optional<String> get(String key);

    /** This node itself plus every peer it currently believes to be alive. */
    Set<String> aliveMembers();
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    /**
 * Wiring point between the test contract and YOUR design.
 * Create your own classes anywhere under src/main/java and return them from here.
 */
    public static GossipNode create(String id, List<String> peers, BiConsumer<String, String> send, long seed)
}
```

## What the tests check

These tests are black-box and deterministic: every node runs inside a seeded simulation (one tick = 1 ms of simulated time, nothing sleeps), messages can be delayed, dropped, duplicated and reordered, nodes can crash and recover, the network can be partitioned, and safety invariants are checked after EVERY simulated millisecond. A failing seed reproduces exactly.

- A single write reaches all 16 nodes within 1.5 s; many keys written by many writers (12 nodes) all converge within 2.5 s.
- A later write overwrites an earlier one everywhere and the old value never comes back; concurrent equal-version writes resolve to the larger writer id (`from-n5` beats `from-n2`).
- Convergence with 30% message loss, and with 50% duplication plus latencies up to 120 ms (reordering).
- No false suspicion in a healthy 16-node cluster for 10 s, nor with 20% message loss.
- A crashed node is still considered alive after 1.5 s but is dropped by all 15 others within 3.5 s of the crash; a recovered node is readmitted everywhere and catches up on writes it missed.
- An 8-node cluster split in 4+4: each side keeps writes on its side, sees only its own members alive, and after `heal` both sides converge and see 8 members.
- Fan-out: no node sends 250+ messages in 30 rounds with 32 nodes, yet the value still spreads everywhere.
- Same seed gives the same run (per-node message counts and delivery state are compared); 100 random schedules of writes under 10-30% loss converge on one of the written values.
- More edge cases (100-node clusters, ten simultaneous crashes) in `interviewer/` (see `INTERVIEWER_NOTES.md`).

## Language notes

Randomness must come from the seed given to the constructor; the replay test compares two runs with the same seed.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/100-gossip-protocol test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`SimNet`, `FakeClock`, `Candidate`): `java/testkit`.
