# Interviewer notes — 105 Raft Log Replication

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: safety violation (split brain, lost or divergent committed data), liveness problem (no progress), missed failure case (crash, loss, duplicate, reorder, partition), non-determinism, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Two candidates in the same term: only one wins (one vote per term); split votes resolve via re-drawn timeouts.
- A stale AppendEntries (shorter than the follower's log) arrives after newer ones: the log must not shrink.
- A leader sees an old-term entry replicated on a majority: it must NOT commit it by counting - only through a later own-term entry (the no-op).
- A candidate with a longer log but a lower last term must lose the vote (term before length).
- A recovered node whose clock jumped: no election burst, it first listens for a leader.
- A leader without replies from a majority steps down within 300 ms of silence, and then `propose` is refused.
- Duplicate and reordered replies: matchIndex only moves forward; nextIndex backs off by one exactly per rejection.
- The very first client index is >= 2 because of the hidden no-op, and `commitIndex` counts it.

## Likely interviewer questions
- Why can a leader not commit an entry from a previous term just by counting replicas? Describe the Figure 8 scenario.
- What does the election restriction guarantee, and how does it relate to the commit rule?
- Why is the no-op appended at the start of a term?
- Why do followers delete only conflicting entries, and what would a naive 'truncate to prevIndex' do under duplication?
- How does check-quorum help, and what is the cost (a leader with a flaky network steps down)?
- What is lost when a node crashes and loses `votedFor`? Construct a double-vote scenario.

## Invariants
- Election safety: at most one leader per term.
- Leader append-only: a leader never overwrites or deletes entries in its own log.
- Log matching: if two logs contain an entry with the same index and term, they are identical up to that index.
- Leader completeness: every committed entry is present in the log of every later leader.
- State machine safety: committed lists are prefixes of one another and a node's committed list only grows.

## Common mistakes
- Committing by counting replicas for entries of an older term.
- Truncating the follower's log on every AppendEntries instead of only on conflict.
- Granting votes without comparing the logs (or comparing length before last term).
- Not resetting the election timer on a valid AppendEntries / granted vote, or resetting it on a rejected one.
- Not adopting a higher term seen in a reply, so a stale leader keeps acting.
- Letting the commit index move backwards or beyond the verified prefix of the log.
- Counting the leader itself twice (or not at all) in the majority; handling a 3-node and a 5-node quorum differently by accident.
- Reusing a vote across terms (votedFor not reset on a new term).

## Over-engineering traps
- Implementing snapshots, membership changes or pre-vote - none are required.
- A generic state-machine-replication framework with command serialisers and a plug-in transport.
- Splitting into many classes (RPC, timer, log store) before the node passes the single-leader test.

## Distributed-systems discussion
- etcd, Consul, CockroachDB and TiKV (per-range Raft groups), Kafka KRaft and RabbitMQ quorum queues run Raft; ZooKeeper's Zab and Paxos-based systems (Spanner, Chubby) solve the same problem with a similar leader+quorum structure.
- FLP says deterministic consensus cannot be both safe and live in a purely asynchronous system with one crash; Raft keeps safety always and liveness only when the network is eventually stable (randomised timeouts make a split vote unlikely, not impossible).
- CAP: a minority partition can neither elect a leader nor commit, so Raft is CP; reads from followers or a deposed leader may be stale unless you add ReadIndex/leases (V4).
- Production adds what is simplified here: durable term/vote/log with fsync before replying, snapshots and compaction, pipelined and batched AppendEntries, pre-vote, joint consensus, and clients that retry with idempotency keys because a proposal can be lost or applied twice.

## Trade-off questions to ask when the candidate is done
- Why 150-300 ms election timeouts with a 50 ms heartbeat? What changes on a WAN?
- Throughput: batching and pipelining AppendEntries versus latency of a single command.
- Raft vs Paxos vs a leaderless quorum store (problem 104): consistency, availability and complexity.
- Check-quorum and pre-vote: which failure modes do they remove, and which behaviour do they cost you?
- How would you test a consensus implementation beyond these seeds (model checking, Jepsen-style fault injection)?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
