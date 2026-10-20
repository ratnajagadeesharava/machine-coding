# Interviewer notes — 100 Gossip Protocol with Failure Detection

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: safety violation (split brain, lost or divergent committed data), liveness problem (no progress), missed failure case (crash, loss, duplicate, reorder, partition), non-determinism, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Two writers write the same key concurrently with equal versions: the larger writer id must win on every node.
- A write that starts from a node that already learned a higher version must overtake it (version = known + 1, not a local counter).
- Duplicated and reordered pushes: an old state arriving late must not overwrite a newer value or lower a heartbeat counter.
- A push answered with a push-answer that is answered again: message storm (visible in the per-node message count).
- A node whose peer list has fewer than 2 entries (fan-out must be min(2, peers)).
- 100 nodes: dissemination time must stay logarithmic; ten simultaneous crashes must be detected by all survivors.
- A healed partition: the alive set must grow back, not stay stuck at the side's size.

## Likely interviewer questions
- How many rounds does it take for a rumor to reach N nodes with fan-out 2? Why logarithmic?
- Why must the push answer not be answered again? What would the message count look like?
- Why is `version + 1` better than a per-node Lamport counter or wall-clock time here? What still goes wrong with concurrent writes?
- Why do you use 'time since the counter last grew on MY clock' rather than comparing timestamps sent by the peer?
- How would you choose the 2000 ms timeout? What happens to false positives when the loss rate goes up?
- What does a node do after a restart that wiped its memory (heartbeat counter back to zero)?

## Invariants
- Merge is commutative, associative and idempotent: all nodes holding the same set of updates agree on every key.
- A heartbeat counter and a key's version never decrease on any node.
- Eventual delivery: with fair (non-total) loss every write reaches every connected node.
- No node sends to every peer in one round (bounded fan-out).
- Completeness of failure detection: a crashed node is eventually suspected by all live nodes; accuracy: a healthy node is not suspected in a healthy network.

## Common mistakes
- Broadcasting to all peers, or answering a push-answer so that messages ping-pong.
- Using the sender's wall-clock timestamp for liveness (clocks differ), or never updating 'last grew' because the merge ignores equal counters.
- Overwriting on arrival instead of comparing versions, so a delayed old state resurrects an old value.
- Resolving equal versions by arrival order, giving different winners on different nodes.
- Using unseeded randomness (`Math.random`, hash-map order) so the replay test differs between runs.
- Counting the own node as dead or leaving it out of `aliveMembers`.

## Over-engineering traps
- A pluggable transport / serializer / membership-provider hierarchy before the three-line merge works.
- Implementing SWIM, vector clocks or a Merkle-tree anti-entropy that the contract does not ask for.
- Scheduling with threads or timers instead of reacting to `onTick`.

## Distributed-systems discussion
- Cassandra gossips every second with up to 3 peers, using version numbers per node plus the phi-accrual failure detector instead of a fixed timeout; Consul/Serf (memberlist) use SWIM with indirect probes and suspicion.
- Gossip is AP: during a partition both sides stay writable and diverge, and last-writer-wins by version silently drops one of two concurrent writes. CRDTs or vector clocks keep both; here we accept the loss.
- Failure detection is only 'eventually perfect' in a partially synchronous network (FLP/Chandra-Toueg): a slow node is indistinguishable from a dead one. The timeout is a trade of detection speed against false positives.
- Simplified here: whole-state exchange (production sends deltas / digests), a fixed peer list, no tombstone GC, no authentication, no clock-skew handling.

## Trade-off questions to ask when the candidate is done
- Push vs pull vs push-pull: which converges fastest, and which sends the least?
- What happens to message volume and detection latency when you double the cluster size? Which parameters would you scale?
- Fixed 2 s timeout vs phi-accrual / adaptive timeout: when does the fixed value hurt?
- What does a delete cost in a system whose merge is 'take the bigger version'?
- How would you test convergence without a simulator?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.
