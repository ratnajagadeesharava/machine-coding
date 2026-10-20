# 96 · Consistent Hash Ring (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 60 min | Level 3 — infrastructure provided: a deterministic network and clock simulation (`SimNet`) runs your nodes; the protocol code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Our cache fleet is sharded by key. Today the shard is `hash(key) % N`, and every time we add or remove a machine almost every
key moves, which empties the caches and takes the database down. You are asked to build the routing component that maps keys to
nodes so that membership changes move only the keys that have to move, load stays balanced, big machines get more load than
small ones, and the component can also name the next few nodes for replication. Two processes that hold the same membership
must route every key identically, and lookups run constantly while nodes join and leave.

## Requirements

1. `addNode(name)` adds a node of weight 1; `addNode(name, weight)` adds a node that owns about `weight` times as many keys. A duplicate node, a blank name or a weight < 1 is rejected (`IllegalArgumentException` / `Err(InvalidNode)`).
2. `removeNode(name)` returns whether the node was present; its keys go to the remaining nodes.
3. `nodeFor(key)` returns the node responsible for the key; an empty ring is an error (`IllegalStateException`; Rust: `None`).
4. `nodesFor(key, n)` returns up to `n` **distinct** nodes: first `nodeFor(key)`, then the following distinct nodes clockwise. Fewer if the ring has fewer nodes; empty for an empty ring or `n = 0`.
5. The mapping depends only on the set of (node name, weight) pairs and the key: not on insertion order, not on a random seed, not on the process. Two rings with the same membership agree on every key.
6. Adding a node only moves keys to that node; removing a node only moves the keys that node owned. No key moves between two untouched nodes.
7. The ring is created with a virtual-node count per unit of weight (`create(vnodes)`); a node of weight w gets `w * vnodes` points on the ring. Keys spread evenly across nodes.
8. `nodes()` returns the current node names.

## Simulation rules and constraints

- This component runs as plain in-process code (no network simulation); the tests exercise it from real threads.
- `nodeFor` and `nodesFor` are called continuously from many threads while one thread adds and removes nodes. A lookup must see either the membership before or after a change, never a torn ring, and never throw because of a concurrent change.
- Lookups vastly outnumber changes: reads should not queue behind each other.
- Determinism is part of the contract: no `Random`, no use of `hashCode()` of strings across runtimes, no iteration-order dependence. Use a hash function whose output you control.

## Examples

```
create(vnodes = 200); addNode n0..n9
nodeFor("key-1") -> "node-4"        (same answer in every process and every insertion order)
addNode("newcomer") -> about 1/11 of the keys move, all of them to "newcomer"
removeNode("node-3") -> only keys owned by node-3 move
nodesFor("key-1", 3) -> ["node-4", "node-7", "node-1"]   // distinct, primary first
if node-4 leaves, nodeFor("key-1") == "node-7" (the next replica was promoted)
```

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `HashRing` with overloads `addNode(String)` / `addNode(String, int)`, factory `Solution.create(int virtualNodes)`, `nodes()` returns a `Set<String>`. Rust: single `add_node(&self, &str, u32) -> Result<(), RingError>`, `node_for -> Option<String>`, `nodes()` returns a **sorted** `Vec<String>`, `create(virtual_nodes: usize)`.
- Even spread: with 200 vnodes and 10 nodes, each node owns between 65% and 135% of the mean over 50,000 keys.
- Join: moved keys are 4%-15% of all keys with 10 nodes + 1 newcomer (expected about 9%), and every moved key's new owner is the newcomer.
- Leave: every key not owned by the leaving node keeps its owner; the node disappears from `nodes()`.
- Replicas: `nodesFor(k, 3)` has 3 distinct entries with the primary first; `nodesFor(k, 10)` on a 6-node ring has 6; `n = 1` equals `[nodeFor(k)]`; `n = 0` is empty.
- When the primary of a key leaves, the previous second replica becomes the new primary.
- Weights: with nodes of weight 1, 1 and 3 the heavy node owns 60% +/- 20% (relative) of the keys and a light node 20% +/- 30% (relative).
- Removing an unknown node returns false; no exception.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.List;
import java.util.Set;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe (lookups run while nodes join and leave).
 *
 * <p>The mapping must depend only on the set of nodes (names and weights) and the key - never on the order in which
 * nodes were added or on a random seed - so that two processes holding the same membership agree on every key.
 */
public interface HashRing {

    /** Adds a node with weight 1. IllegalArgumentException if it is already on the ring or the name is blank. */
    void addNode(String node);

    /** A node of weight w owns about w times as many keys as a node of weight 1. Weight >= 1. */
    void addNode(String node, int weight);

    /** Returns false if the node was not on the ring. Its keys move to the remaining nodes. */
    boolean removeNode(String node);

    /** The node responsible for the key. IllegalStateException if the ring is empty. */
    String nodeFor(String key);

    /**
     * Up to {@code n} DISTINCT nodes for replication: first {@code nodeFor(key)}, then the following distinct nodes
     * clockwise on the ring. Fewer than n if the ring has fewer nodes. Empty list for an empty ring.
     */
    List<String> nodesFor(String key, int n);

    Set<String> nodes();
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

    /** {@code virtualNodes} points per unit of weight on the ring (at least 1). */
    public static HashRing create(int virtualNodes)
}
```

## What the tests check

These tests are black-box and deterministic: every node runs inside a seeded simulation (one tick = 1 ms of simulated time, nothing sleeps), messages can be delayed, dropped, duplicated and reordered, nodes can crash and recover, the network can be partitioned, and safety invariants are checked after EVERY simulated millisecond. A failing seed reproduces exactly.

- Empty ring: `nodeFor` fails, `nodesFor` is empty; a single node owns everything.
- Duplicate node, blank name, weight 0 rejected; unknown removal returns false.
- Same key always maps to the same node, and a ring built with a shuffled insertion order agrees on 5,000 keys.
- Balance with 10 nodes and 50,000 keys; minimal movement on join and on leave; replica distinctness and promotion; weight proportionality.
- Concurrency: 7 reader threads x 20,000 lookups while a writer adds 20 and removes some nodes - owners are always known nodes and replicas are never empty. **Repeat 100x**: four threads adding nodes concurrently end with a consistent ring of 8 nodes.
- More edge cases and scale scenarios in `interviewer/` (see `INTERVIEWER_NOTES.md`).

## Language notes

Do not use `String.hashCode()` or `Object.hashCode()` for ring positions; implement or import a real mixing hash (murmur3, FNV-1a with a final mix) over UTF-8 bytes. `TreeMap.ceilingEntry` or a sorted `long[]` plus `Arrays.binarySearch` both work.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/96-consistent-hashing test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`SimNet`, `FakeClock`, `Candidate`): `java/testkit`.
