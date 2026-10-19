# Interviewer notes — 99 Raft Leader Election

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: safety violation (split brain, lost or divergent committed data), liveness problem (no progress), missed failure case (crash, loss, duplicate, reorder, partition), non-determinism, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- A lone node and a two-node cluster: majority arithmetic (2 of 2, not 1 of 2).
- Duplicate or delayed vote replies from an old term must not count toward the current election.
- A vote request with a higher term from a node whose log is irrelevant here: still adopt the term and grant if you have not voted in it.
- Granting a vote resets the election timer; refusing one does not.
- A leader isolated from the majority steps down by check-quorum even though no higher term is ever seen.
- A candidate that receives a same-term heartbeat becomes follower; a stale lower-term heartbeat does not reset its timer.
- Eleven nodes with up to 40 ms latency still elect within 5 s; ten successive leader crashes always lead to a new leader and non-decreasing terms.

## Likely interviewer questions
- Why are election timeouts randomised? What goes wrong with a fixed timeout?
- Why does a node vote at most once per term, and what happens if it forgets that rule?
- Walk through a split vote: how does the cluster recover?
- What does check-quorum buy you, and how does it differ from just waiting to hear a higher term?
- A partitioned minority node keeps timing out. What happens to its term, and what happens when it rejoins?
- Why do you compare terms before looking at the message type?

## Invariants
- Election safety: at most one leader per term, ever.
- A node's term never decreases; a node votes at most once per term.
- A node is LEADER only in the term in which it won a majority.
- A node that has seen a higher term is a FOLLOWER of that term.

## Common mistakes
- Counting votes without a majority of the whole cluster (using only reachable peers).
- Not forgetting the vote when adopting a higher term, or granting two votes in one term.
- Letting a stale vote reply from an earlier term or election count.
- Resetting the election timer on any message, including lower-term ones and refused votes.
- No jitter or identical seeds: repeated split votes.
- A leader that never checks quorum, so a partitioned old leader stays LEADER forever.
- Using wall-clock time, `Math.random()` or a hash-map iteration order that breaks determinism.

## Over-engineering traps
- Implementing log replication, snapshots and membership change now; the tests only need elections.
- A generic state-machine framework with transition tables for three roles.
- Threads and real timers inside the node; the harness provides ticks.

## Distributed-systems discussion
- Raft (etcd, Consul, CockroachDB, TiKV) and Paxos-family systems (Spanner, Chubby) rely on the same ingredients: terms/ballots, majority votes, and randomized or leased leadership.
- Randomised timeouts make split votes rare rather than impossible; liveness is only guaranteed once the network is stable enough (FLP says you cannot have both safety and guaranteed liveness in a fully asynchronous system).
- Pre-vote and check-quorum (etcd both) exist to stop disruptive nodes and zombie leaders; leader leases give fast reads at the price of clock assumptions.
- Persistence of term and vote is a safety requirement: a node that forgets them can vote twice in a term.
- Alternatives: ZooKeeper's ZAB, lease-based leadership on top of a lock service (problem 98), gossip with a deterministic 'smallest id wins' rule (not safe under partitions).
- A deterministic simulator with seeded faults (like the one here, or FoundationDB's) is how such protocols are tested for real.

## Trade-off questions to ask when the candidate is done
- Election timeout range vs heartbeat interval: failover speed vs spurious elections.
- Check-quorum and pre-vote: what each prevents and what each costs.
- Majority quorums vs larger quorums vs flexible quorums.
- A single leader as bottleneck vs leaderless designs (Dynamo-style).
- How would you extend the tests to catch a node that forgets its vote after a crash?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
