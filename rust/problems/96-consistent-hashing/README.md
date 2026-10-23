# 96 · Consistent Hash Ring (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 60 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

You are building the routing component in front of a sharded cache fleet. Today a key goes to machine `hash(key) % N`, so
adding or removing one machine remaps almost every key, empties the caches and floods the database. Your component maps keys to
nodes so that when a node joins or leaves only the keys that must move do move, keys are spread evenly, a node with weight 3
gets about three times the keys of a node with weight 1, and for replication it can name the next few distinct nodes for a
key. Two processes holding the same membership must route every key to the same node. Lookups run constantly from many threads
while nodes are added and removed.

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
9. `create(vnodes)` takes the number of ring points a weight-1 node gets (at least 1); a node of weight `w` gets `w * vnodes` points. A blank name means empty or whitespace only.
10. Ownership is a pure function of the membership (the set of node names with their weights) and the key. Removing a node and adding it back with the same weight restores exactly the previous ownership of every key.
11. The tests do not fix which node owns which key (the hash function is yours); they check the properties above: determinism, evenness, minimal movement, replica order, weight proportions.
12. `nodes_for(key, n)` is a prefix-stable list: `nodes_for(k, 2)` is the first two entries of `nodes_for(k, 3)`.

## Simulation rules and constraints

- This component runs as plain in-process code (no network simulation); the tests exercise it from real threads.
- `nodeFor` and `nodesFor` are called continuously from many threads while one thread adds and removes nodes. A lookup must see either the membership before or after a change, never a torn ring, and never throw because of a concurrent change.
- Lookups vastly outnumber changes.
- Determinism is part of the contract: the same nodes and keys give the same results in every run and on every runtime.

## Glossary

- **Ring**: the circle of positions that keys and nodes are hashed onto. A key belongs to the first node position met going *clockwise* from the key's position, wrapping around at the end.
- **Virtual node (vnode)**: one of the many positions a physical node occupies on the ring; more positions = smoother balance.
- **Weight**: a positive integer multiplier on the number of positions, so the node owns proportionally more keys.
- **Primary / replicas**: the primary is `node_for(key)`; the replicas are the next *different* nodes found continuing clockwise.
- **Minimal movement**: after a membership change, a key changes owner only if the change forces it (the new node takes it, or its old owner left).

## Examples

**Example 1: basics** (`create(vnodes = 200)`, nodes `node-0` .. `node-9` added with weight 1; the exact owners depend on your hash)

```
node_for("key-1")            -> some node, always the same one, in every process and for any insertion order of the nodes
over 50 000 keys            -> every node owns between 65% and 135% of 5 000 keys
node_for on an empty ring    -> None;   nodes_for on an empty ring -> []
```

**Example 2: a node joins, a node leaves**

```
before = owner of each of 50 000 keys
add_node("newcomer")  (11 nodes)
after  = owner of each key
  every key whose owner changed now belongs to "newcomer";  4% .. 15% of keys changed (about 1/11)
remove_node("node-3") -> true
  keys owned by node-3 go to other nodes; every other key keeps its owner
remove_node("nope")   -> false (no error)
```

**Example 3: replicas**

```
nodes_for(k, 3) -> [p, r1, r2]    three DISTINCT nodes, p == node_for(k)
nodes_for(k, 1) -> [p]            nodes_for(k, 0) -> []
nodes_for(k, 10) on a ring of 6 nodes -> all 6 nodes, no duplicates
remove_node(p)  -> node_for(k) is now r1 (the old second replica is promoted)
```

**Example 4: weights**

```
add_node("small-1"); add_node("small-2"); add_node("big", 3)
over 50 000 keys: big owns about 60% (between 48% and 72% of all keys), each small node about 20%
```

## Edge cases

- Rejected with `Err(RingError::InvalidNode)`: a node name that is already on the ring, a blank name, a weight below 1 (e.g. `add_node("x", 0)`). A rejected call changes nothing.
- A ring with a single node sends every key to it; `nodes_for(k, 5)` then returns just that one node.
- A lookup that runs while another thread adds or removes nodes returns an owner that is a member either before or after that change, never an error and never a node that never existed.
- `nodes()` returns the names sorted alphabetically.
- Several threads adding different nodes at the same time all succeed; afterwards the ring has all of them.

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

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`): lookups run while nodes join and leave.
//!
//! The mapping must depend only on the set of nodes (names and weights) and the key - never on the order in which
//! nodes were added or on a random seed - so that two processes holding the same membership agree on every key.

#[derive(Debug, PartialEq, Eq)]
pub enum RingError {
    /// the node is already on the ring, the name is blank, or the weight is 0
    InvalidNode,
}

pub trait HashRing: Send + Sync {
    /// Adds a node. A node of weight w owns about w times as many keys as a node of weight 1 (weight >= 1).
    fn add_node(&self, node: &str, weight: u32) -> Result<(), RingError>;

    /// Returns false if the node was not on the ring. Its keys move to the remaining nodes.
    fn remove_node(&self, node: &str) -> bool;

    /// The node responsible for the key; `None` if the ring is empty.
    fn node_for(&self, key: &str) -> Option<String>;

    /// Up to `n` DISTINCT nodes for replication: first `node_for(key)`, then the following distinct nodes clockwise
    /// on the ring. Fewer than `n` if the ring has fewer nodes. Empty for an empty ring.
    fn nodes_for(&self, key: &str, n: usize) -> Vec<String>;

    /// All nodes, sorted.
    fn nodes(&self) -> Vec<String>;
}
```

Your wiring point (`src/lib.rs`):

```rust
/// `virtual_nodes` points per unit of weight on the ring (at least 1).
pub fn create(_virtual_nodes: usize) -> Option<Arc<dyn HashRing>> { ... }
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Time-dependent behaviour uses an injectable fake clock, never real sleeping. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Empty ring: `nodeFor` fails, `nodesFor` is empty; a single node owns everything.
- Duplicate node, blank name, weight 0 rejected; unknown removal returns false.
- Same key always maps to the same node, and a ring built with a shuffled insertion order agrees on 5,000 keys.
- Balance with 10 nodes and 50,000 keys; minimal movement on join and on leave; replica distinctness and promotion; weight proportionality.
- Concurrency: 7 reader threads x 20,000 lookups while a writer adds 20 and removes some nodes - owners are always known nodes and replicas are never empty. **Repeat 100x**: four threads adding nodes concurrently end with a consistent ring of 8 nodes.
- More edge cases and scale scenarios in `interviewer/` (see `INTERVIEWER_NOTES.md`).

## Running

```bash
cargo test -p p96_consistent_hashing
STRICT=1 cargo test -p p96_consistent_hashing   # fail instead of skip while unimplemented
cargo test -p p96_consistent_hashing --features hidden   # interviewer scenarios
cargo play run 96              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`SimNet`, `Rng`, `sut!`): `rust/support`.
