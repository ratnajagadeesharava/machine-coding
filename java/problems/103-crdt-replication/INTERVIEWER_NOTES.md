# Interviewer notes — 103 CRDT Replicated Store

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: safety violation (split brain, lost or divergent committed data), liveness problem (no progress), missed failure case (crash, loss, duplicate, reorder, partition), non-determinism, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Merging a snapshot taken before a decrement or an increment after a newer one is already merged: the value must not go back.
- Remove of an element that was added on another replica but never observed here: the add must survive the sync.
- Re-add of a removed element, then the pre-re-add snapshot (which contains the remove) arrives late.
- Equal timestamps on a register written by two replicas; a write with an older timestamp locally.
- Namespaces: `counter("x")`, `elements("x")`, `read("x")` coexist; `counter("e")` is 0 for a set element named `e`.
- Names containing newlines, `|`, `;`, `,`, quotes and non-ASCII characters.
- Snapshot growth: 2,000 increments on one counter must not make the snapshot ten times bigger; merge time for 20,000 elements stays in seconds, not minutes.

## Likely interviewer questions
- Why is a plain 'map name -> long' not a valid counter CRDT? What goes wrong with a merge by sum?
- Why does the OR-set need unique tags? What does 'add wins' mean and why not 'remove wins'?
- Can the set forget removed tags? What would you need to know first?
- What is lost by LWW? Give a scenario where a user's write disappears.
- Why can the merge not be allowed to depend on the order of arrival?
- How do you prove commutativity / associativity / idempotence for your merge (hint: it is a join in a lattice)?

## Invariants
- Merge is idempotent: merge(s, s) = s; commutative: merge(a, b) = merge(b, a); associative.
- Monotone state: merging never makes the state 'smaller' in the lattice (no lost increments, no resurrected removals, no older register value).
- Strong eventual consistency: replicas that received the same set of updates (in any order) show identical values.
- A counter's value equals the global sum of increments minus decrements once all snapshots have been merged.
- A snapshot is immutable; it never changes after being taken.

## Common mistakes
- Merging counters by adding the values (double counts on every repeated merge) or by taking the max of the total (loses concurrent increments).
- Representing the set as a plain `Set<String>` merged by union (resurrects removed elements) or by intersection (loses adds).
- Tagging additions with a local counter that is not unique across replicas, or reusing tags after a remove.
- Returning a live view from `elements` or a snapshot that shares mutable state.
- Breaking on strings with newlines/separators because of naive `split`.
- Locking only some operations, so `snapshot` can see a half-applied update.
- Comparing replica ids numerically instead of as strings for LWW ties.

## Over-engineering traps
- A generic `CRDT<T>` hierarchy with visitors before the three types work.
- Implementing vector clocks, delta CRDTs or garbage collection that no test requires.
- Choosing JSON libraries / protobuf for the snapshot.

## Distributed-systems discussion
- Riak (data types: counters, sets, maps, registers), Redis Enterprise CRDB, Akka Distributed Data, Automerge/Yjs for collaborative editing and Figma's multiplayer all use CRDT variants; SoundCloud's Roshi is an LWW-element-set.
- CRDTs choose availability and partition tolerance (AP) and give strong eventual consistency; they cannot enforce cross-object invariants like a non-negative balance or uniqueness without coordination (CALM theorem: monotone programs need no coordination).
- Operation-based CRDTs send operations and need reliable causal broadcast; state-based ones (this problem) need only an eventually delivering, possibly duplicating channel but ship more data - delta-state CRDTs close the gap.
- Simplified here: whole-state snapshots, caller-supplied timestamps, no causal context and no garbage collection of tombstones, no persistence, three fixed types.

## Trade-off questions to ask when the candidate is done
- State-based vs operation-based vs delta-state CRDTs: bandwidth, delivery guarantees and implementation complexity.
- Add-wins vs remove-wins: which fits a shopping cart vs a ban list?
- LWW vs multi-value register: what does your product prefer to lose?
- How long may tombstones live, and what happens if a replica is offline longer than that?
- When is a CRDT the wrong tool compared with consensus (Raft, problem 105)?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
