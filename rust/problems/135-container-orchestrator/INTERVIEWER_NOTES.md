# Interviewer notes — 135 Container Orchestrator

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- An evicted container is retried right after the preemptor is placed and can land on another node, or on the same node.
- A displaced container of a failed node keeps its original submission order, so it beats a newer pending one.
- A pending container of priority >= 100 (for example displaced by a node failure) must not preempt in a retry pass.
- Victims are ordered by priority first, then newest: a newer high-priority container is safer than an older low-priority one.
- Preemption target ignores the strategy: fewest evictions, then smallest node id.
- Strictly lower priority only; priority-100 residents can be evicted by 150 but not by 100.
- A long random mixed sequence: after every operation nothing pending fits any node (the system is work-conserving), usage equals the sum of sizes, pending is sorted.
- Concurrent submits, removals and node add/remove churn must never lose or duplicate a container.

## Likely interviewer questions
- Why does `removeNode` return the state after rescheduling and not "where it was"? What would you return if the same call also had to be atomic?
- What is the complexity of a retry pass? Do you re-scan every node for every pending container? Can you stop early?
- Why does the retry pass skip containers that do not fit instead of stopping at the first one (head-of-line blocking)? What does that cost in fairness (starvation of a large high-priority container by a stream of small ones), and how would a real scheduler handle it (reservations, backfill)?
- Is the greedy "lowest priority then newest" prefix really the fewest evictions? (No: choosing a different set could free enough with fewer victims; the spec fixes the greedy rule on purpose. What would an optimal choice cost?)
- Why is preemption limited to `submit`? What would happen with cascading evictions if retries could preempt (ping-pong between two priority levels)?
- Where is the lock, and why is one lock for the whole cluster acceptable here? When would you shard by node and what breaks (cross-node preemption, retry)?

## Invariants
- For every node `used <= total` on both resources, and `used` equals the sum of its containers' requests.
- Every known container is in exactly one place: on one node or in the pending set.
- After any public call returns, no pending container fits any node (a retry pass has run, or the container was just found not to fit).
- A container's submission sequence number never changes while the container is known.
- Strategies only affect which of the fitting nodes is chosen, never whether a container fits.

## Common mistakes
- Computing leftover as a ratio or per-resource, or comparing only CPU for BEST_FIT.
- Choosing the strategy node among all nodes and checking fit afterwards (picks a node that cannot hold it).
- Giving evicted or displaced containers a new submission number, so they lose their place in the queue.
- Stopping the retry pass at the first container that does not fit, or running it only once for the first freed node.
- Preempting on equal priority, on priority 99, or evicting more victims than needed because the loop condition checks only CPU.
- Mutating state while evaluating candidate nodes for preemption, then failing to roll back when no node works (partial eviction).
- Returning `removeNode` results in node or retry order instead of container-id order, or reporting the pre-rescheduling state.
- Checking `used <= total` outside the critical section (check-then-act race), or releasing the lock between placing the container and running the retry pass.
- Taking the `Mutex` in a public method and calling another public method that locks it again (deadlock), or dropping the guard between placing and retrying.
- Letting `remove` of a pending container try to free node capacity (unwrapping a missing node).

## Over-engineering traps
- A plugin framework or event bus for scheduling policies before three comparators work.
- A separate service per concern (scheduler, resource manager, queue manager) with its own locks, creating lock-ordering problems.
- Modelling pods, namespaces, labels or real resource quantities.
- Persistence or a watch API as part of the core.

## Design and scaling discussion
- Strategy over node selection (a comparator on candidates) is the natural fit; the pending queue is a priority queue with a stable tie-break; the node set is a sorted map. A good candidate sees that `place`/`unplace` are the only functions touching usage.
- One cluster lock makes every operation (including its retry pass) atomic and trivially correct. Reads can use the same lock or a snapshot. Finer locking needs a total lock order on nodes and a way to do preemption across nodes atomically; usually not worth it at this scale.
- Complexity: a naive retry pass is O(P * N) per operation; keeping a per-node free-capacity index (or skipping the pass when nothing was freed) helps. A real scheduler filters and scores nodes in parallel and caches.
- Starvation: with no reservations a stream of small low-priority containers can keep a large container pending forever. Preemption is the escape hatch only for priority >= 100.
- Node failure in real clusters is detected by heartbeats and handled with grace periods; here it is instantaneous and deterministic.

## Trade-off questions to ask when the candidate is done
- Would you keep containers on a node in a sorted set or sort on read? What is the read/write ratio?
- What if the strategy could differ per container? Where would that choice live?
- How would you test that retry never preempts, and that a failed preemption evicts nothing?
- What breaks first if the cluster has 10,000 nodes and 100,000 pending containers?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.
