# Interviewer notes — 98 Lease Lock with Fencing Tokens

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: safety violation (split brain, lost or divergent committed data), liveness problem (no progress), missed failure case (crash, loss, duplicate, reorder, partition), non-determinism, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Exactly `t + d` is expired; `t + d - 1` is still valid, for acquire, holder, renew and release alike.
- Same owner re-acquiring after its own lease lapsed gets a new, higher token.
- Token numbering is per lock and continues after release and expiry.
- A stale owner's release or renew is a no-op and must not disturb the current holder.
- Release twice: second is false.
- 100,000 distinct locks created from eight threads.
- A hot lock stolen every 10 ms by a new owner for 1,000 rounds while readers poll `latestToken`: tokens are exactly 1..1000 and readers never see them go backwards.

## Likely interviewer questions
- Why is a lease alone not enough for mutual exclusion? Walk through the GC-pause scenario.
- Who enforces the fencing token, and what must that system do?
- Why must the token counter outlive the lease?
- What is your per-lock atomicity story? Can unrelated locks proceed in parallel?
- How is expiry checked without a background thread?
- What would change if `acquire` had to block?

## Invariants
- Tokens issued for a lock are strictly increasing, with no repeats.
- At any instant at most one owner holds an unexpired lease on a lock.
- `renew`/`release` succeed only for the current owner and token with an unexpired lease.
- `latestToken` never decreases.

## Common mistakes
- Resetting the token counter when the lock is released or expires.
- Giving a new token to the same owner on every extension (or none after its own lapse).
- Off-by-one at expiry (`<=` instead of `<`).
- Allowing `renew` after expiry because nobody else took the lock yet.
- Not re-checking the token on release, so a stale owner frees the new holder's lock.
- Check-then-set across two map operations (two winners).
- Using the wall clock instead of the injected one.

## Over-engineering traps
- A timer or scheduler thread that removes expired leases; lazy checks are simpler and deterministic.
- Wait queues, fairness policies and condition variables for an API that never blocks.
- A pluggable storage backend and event system before the in-memory rules are right.

## Distributed-systems discussion
- Martin Kleppmann's critique of Redlock makes the point: a lease needs a fencing token checked by the resource, otherwise a paused client can corrupt data. Google Chubby introduced 'sequencers' for the same reason.
- ZooKeeper (ephemeral nodes plus zxid), etcd (leases plus revision numbers) and Consul sessions provide the building blocks; the token is a monotonic revision.
- A single-node lock service is a single point of failure; making it replicated means consensus (Raft/Paxos) for token issuance, and tokens must survive failover.
- Time assumptions: leases rely on bounded clock drift, not synchronised clocks; the holder should stop acting before the lease expires by a safety margin.
- Locks vs leader election: both are 'one winner for a while'; the difference is granularity and who needs the guarantee.

## Trade-off questions to ask when the candidate is done
- Short vs long leases: failover time vs renewal load and false expirations.
- Lazy expiry vs active expiry: determinism and cost vs prompt wake-ups of waiters.
- Fencing in the resource vs trying harder to make the lock 'safe': why the second cannot work.
- Token per lock vs a global token: ordering guarantees and contention.
- What does the resource need to store to enforce fencing, and what if it cannot?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.
