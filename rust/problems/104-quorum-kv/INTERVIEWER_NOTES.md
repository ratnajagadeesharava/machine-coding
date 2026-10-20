# Interviewer notes — 104 Quorum Key-Value Store

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: safety violation (split brain, lost or divergent committed data), liveness problem (no progress), missed failure case (crash, loss, duplicate, reorder, partition), non-determinism, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- A duplicated ack from the same node must not count twice (otherwise a lone coordinator at n=3 gets a 'majority').
- Put must remain PENDING (not FAILED) before 1000 ms when only the local ack exists, and FAILED shortly after.
- An old write (lower version) arrives at a replica: acknowledged, not applied.
- Same millisecond, two coordinators: the larger node id wins on every replica.
- A read answer arrives after the reply was decided and shows an older entry: read repair still sent.
- A coordinator crashed and recovered: its next tick sets the time; requests that expired meanwhile are FAILED, not resurrected.
- 5 nodes with 2 crashed: still OK; with 3 crashed: FAILED.
- Partition (3 | 2) followed by heal: a read through the old minority returns the value written on the majority side and repairs the minority.

## Likely interviewer questions
- Why do W = R = majority give you reads that see acknowledged writes? What if N = 4?
- Why acknowledge a stale write at all? What if you refused it?
- Why must the coordinator keep re-sending after OK? Which property does that serve?
- What can still be inconsistent with this design (concurrent writes, two sequential reads going backwards, a failed write that surfaces later)?
- Why is the version taken from the node's last tick and not a wall clock? What happens with clock skew in a real system?
- How does read repair interact with a concurrent write to the same key?

## Invariants
- Every OK put has been stored on a majority; every OK/NOT_FOUND get was answered by a majority, so quorums intersect.
- A get after an acknowledged put returns that put's value or a newer one.
- A replica's entry for a key only ever moves to a strictly higher version.
- A request is answered at most once and its reply never changes once final (OK/NOT_FOUND/FAILED).
- No request stays PENDING after 1000 ms plus a re-send interval.

## Common mistakes
- Counting acks with an integer so duplicates complete the quorum.
- Refusing to acknowledge a write with a lower version (the coordinator then never reaches a majority and returns FAILED).
- Returning the first answer instead of the highest version among a majority.
- Giving up re-sending as soon as the quorum is reached, so replicas that were down never converge.
- Replying NOT_FOUND after hearing from fewer than a majority.
- Using a majority of the nodes that are currently believed alive instead of the full cluster size.
- Forgetting that the coordinator's own copy is one of the answers.

## Over-engineering traps
- Building a failure detector, hinted handoff, or a gossip layer that the contract does not call for.
- Vector clocks and sibling resolution before last-writer-wins by (time, id) is passing.
- A generic RPC framework with futures/promises; the contract is message handlers plus polling.

## Distributed-systems discussion
- Amazon Dynamo, Cassandra and Riak are leaderless with tunable N/R/W, hinted handoff, read repair and Merkle-tree anti-entropy; DynamoDB (as a service) uses leader-based Paxos per partition instead.
- Quorum overlap (R + W > N) gives the latest acknowledged value only in the absence of concurrent writes and sloppy quorums; it is NOT linearizable (Kleppmann): readers can see stale or going-back values without a write-back phase. Raft (problem 105) trades availability for linearizability.
- CAP: here a minority partition cannot write or read at all (CP-ish for the minority) - Dynamo's sloppy quorums and hinted handoff choose availability instead, at the price of weaker guarantees.
- Last-writer-wins by timestamp loses concurrent writes and breaks under clock skew; production uses vector clocks / dotted version vectors or CRDTs (problem 103) for conflict handling.
- Simplified here: full replication, a fixed 1 s request lifetime, no hints or anti-entropy, no deletes, no durable storage (crash = pause).

## Trade-off questions to ask when the candidate is done
- W=3,R=1 vs W=1,R=3 vs W=R=2 for N=3: availability, latency and read-after-write for each.
- What does a longer request lifetime buy (more repairs) and cost (memory, slow failures)?
- How would you add deletes? (tombstones with versions and their garbage collection)
- How would you make reads linearizable, and what would that do to availability and latency?
- How would you test the anomalies that quorum systems still allow?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.
