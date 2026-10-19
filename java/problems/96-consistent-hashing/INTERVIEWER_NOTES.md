# Interviewer notes — 96 Consistent Hash Ring

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: safety violation (split brain, lost or divergent committed data), liveness problem (no progress), missed failure case (crash, loss, duplicate, reorder, partition), non-determinism, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Empty ring: `nodeFor` is an error, `nodesFor` is empty.
- Single node owns every key, and `nodesFor(k, 5)` returns just that node.
- Weight 0, blank name, duplicate name are all rejected; removing a missing node is just `false`.
- Insertion order does not change any answer.
- Wrap-around: keys hashing after the last point belong to the first point's node.
- Replica walk skips repeated nodes and stops when it has been around the ring once.
- A million lookups on a 1,000-node ring (200,000 points) are fast; balance stays within bounds after many joins and leaves.

## Likely interviewer questions
- Why does `hash % N` fail, and how many keys move when a node is added to it vs to your ring?
- How many virtual nodes do you use and what does that cost in memory and in balance?
- How do you make the hash identical in every process and every runtime?
- How does `nodesFor` avoid returning the same physical node twice?
- How do readers stay consistent while the ring changes? What is the cost of a change?
- What happens with a hash collision between points of different nodes?

## Invariants
- The ring points are sorted and each belongs to a present node.
- `nodeFor(k)` equals the first element of `nodesFor(k, n)` for any n >= 1.
- Mapping is a pure function of (membership with weights, key).
- Removing a node changes only keys that mapped to it.

## Common mistakes
- Using `hashCode()` (poor spread, differs across runtimes) or a seeded/random placement.
- One point per node, giving a visibly uneven load.
- Not wrapping around at the end of the circle.
- `nodesFor` returning duplicates of one physical node because it walked vnodes, not nodes.
- Rebuilding the whole ring on every lookup, or mutating a shared sorted list while readers iterate.
- Breaking determinism by tie-breaking colliding points by insertion order.

## Over-engineering traps
- A general `Hasher` strategy plus `RingFactory` plus `NodeSelector` hierarchy for one hash function.
- Persisting the ring or gossiping membership: out of scope.
- Elaborate lock-free structures before a snapshot-swap passes.

## Distributed-systems discussion
- Dynamo, Cassandra (token ring with vnodes), Riak and Memcached clients (ketama) all use consistent hashing; Cassandra's replication walks the ring for distinct nodes and racks, exactly like `nodesFor`.
- Alternatives: rendezvous hashing (no ring state, O(N) per lookup), jump consistent hash (no memory, but only supports appending/removing the last bucket), Maglev hashing (lookup table, near-perfect balance, used in load balancers).
- Load balance improves as O(1/sqrt(vnodes)); more vnodes cost memory and slow membership changes; bounded-load variants cap the worst case.
- Membership must be agreed on: a gossip protocol or a coordination service distributes the node set; the ring is derived state.
- Data movement after a membership change is a separate protocol (streaming ranges, hinted handoff, read repair).

## Trade-off questions to ask when the candidate is done
- Ring with vnodes vs rendezvous vs jump hash: memory, lookup cost, replica support, weights.
- Few vs many vnodes per node.
- Locking vs immutable snapshots: reader cost vs change cost.
- How would you measure and report imbalance in production?
- What happens to cache hit rate and to the database when a node dies, and how do replicas soften it?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
