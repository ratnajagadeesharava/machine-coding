# 135 · Container Orchestrator (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

A cluster runs on a handful of machines ("nodes"), each with a fixed amount of CPU and memory. Teams submit containers that each need some CPU and memory and carry a priority. The **scheduler** decides which node each container runs on, keeps the ones that do not fit waiting, starts them as soon as room appears, lets very important containers push out less important ones, and re-homes everything when a node dies. You are building that scheduling core, in memory, with no real containers: the tests only ask "where is this container now?" and "how full is this node?". Many clients submit at the same time and no node may ever be promised more than it has.

## Requirements

### Nodes

1. `addNode(nodeId, cpuMillis, memMb)` registers a node with a fixed capacity. A null/blank id, an id already in use, or a non-positive `cpuMillis` or `memMb` is an error. After a node is added, pending containers are retried (rule 9).
2. `removeNode(nodeId)` models a node failure or decommission; an unknown id is an error. The node disappears, every container that ran on it becomes PENDING (keeping its priority and its original submission order, rule 9) and all pending containers are immediately retried (rule 9, no preemption) by the active strategy on the remaining nodes. It returns one string per container that was on the node, sorted by container id, formatted `containerId->nodeId` or `containerId->PENDING`, describing the state **after** that rescheduling. An empty node gives an empty list. A removed node id may be added again later.

### Containers and placement

3. `submit(containerId, cpuMillis, memMb, priority)` registers a container and tries to place it at once. A null/blank id, an id that is already known (running **or** pending), or a non-positive `cpuMillis` or `memMb` is an error. The priority is any `int` (negative allowed). It returns the node it was placed on, or `Optional.empty()` if it is PENDING.
4. A container **fits** a node when `freeCpu >= cpuMillis` **and** `freeMem >= memMb` (free = total minus the sum of the containers running there). An exact fit is a fit. CPU and memory are independent resources: spare memory does not make up for missing CPU.
5. The **strategy** picks among the nodes where the container fits. Nodes are considered sorted by node id (`String.compareTo`):
   - `FIRST_FIT`: the first fitting node.
   - `BEST_FIT`: the fitting node with the smallest leftover, where leftover = `(freeCpu - cpu) + (freeMem - mem)` after placement.
   - `WORST_FIT`: the fitting node with the largest leftover.
   - Ties go to the smallest node id. The strategy is given at creation (`Solution.create(strategy)`) and can be changed with `setStrategy`; a change only affects later placements, running containers never move. `setStrategy(null)` is an error.
6. A container that fits is placed immediately, even when other containers are waiting (new submissions do not queue behind pending ones).
7. `remove(containerId)` deletes a running or pending container (unknown id is an error). The capacity it held is freed and pending containers are retried (rule 9). The id may be submitted again afterwards.

### Pending containers

8. A container that fits nowhere (and was not placed by preemption, rule 10) is PENDING: it is remembered and `placement` reports `"PENDING"`.
9. **Retry.** `pending()` lists the pending containers in *retry order*: highest `priority` first, ties by oldest submission first. A container's submission order is fixed when it is submitted; being evicted or displaced from a failed node does not give it a new position. A retry pass walks the pending containers in that order **once**; each one that fits somewhere is placed by the active strategy at that moment, and each one that does not fit is skipped (a large container that cannot be placed never blocks smaller ones behind it). A retry pass runs after `addNode`, `remove`, `removeNode`, and after a preemption (rule 10). A retry pass never preempts.

### Preemption

10. When `submit` finds that a container fits nowhere **and its priority is >= 100**, the scheduler may evict running containers to make room:
    - Only running containers with a priority **strictly lower** than the new container's priority can be evicted.
    - For each node, order its evictable containers by priority ascending, then by newest submission first. Take them from the front of that order, **only as many as needed**, until the node would have enough free CPU **and** memory for the new container. If even all of them are not enough, that node cannot be used.
    - Pick the usable node that needs the **fewest evictions**; ties go to the smallest node id. The strategy is not used for this choice.
    - If no node is usable, nothing is evicted and the new container is PENDING.
    - Otherwise the chosen victims become PENDING (their priority and submission order are kept), the new container is placed on that node and `submit` returns it, then a retry pass runs (rule 9), so an evicted container may immediately find room again, on the same node or elsewhere.
    - Preemption happens only inside `submit`. A container with priority below 100, or one that fits somewhere, never evicts anybody.

### Queries

11. `placement(containerId)` is the node id or `"PENDING"`; an unknown container is an error.
12. `nodeUsage(nodeId)` is `"cpuUsed/cpuTotal,memUsed/memTotal"`, for example `"500/1000,256/2048"`; unknown node is an error.
13. `containersOn(nodeId)` lists the container ids running on the node, sorted ascending (`String.compareTo`); unknown node is an error.
14. `pending()` as described in rule 9.

## Constraints

- Thread-safe: many threads call `submit`, `remove`, `addNode`, `removeNode` and the queries at the same time. No node may ever be over-committed (`used <= total` for both resources at every moment, hidden tests watch it from another thread), no container may be lost or listed in two places, and every operation behaves as if it ran alone and atomically (including its retry pass).
- A failed call (any error) changes nothing.
- Everything is deterministic: no clock, no randomness.

## Examples

### Strategies

```
addNode("n1",1000,1000); addNode("n2",400,400); addNode("n3",2000,2000)
submit("c",300,300,1) with FIRST_FIT -> n1
                      with BEST_FIT  -> n2     leftovers: n1 1400, n2 200, n3 3400
                      with WORST_FIT -> n3

addNode("n1",1000,100); addNode("n2",500,500);  submit("c",100,50,1)
  leftover n1 = 900+50 = 950, n2 = 400+450 = 850   -> BEST_FIT n2, WORST_FIT n1
```

### Pending, retry order and node failure

```
addNode("n",100,100)
submit("blocker",100,100,1)  -> n
submit("p1",100,100,5) -> empty; submit("p2",100,100,9) -> empty; submit("p3",100,100,5) -> empty
pending() -> [p2, p1, p3]                       (priority 9 first, then p1 before p3)
remove("blocker")  -> p2 is placed on n, pending() -> [p1, p3]

addNode("n1",100,100); addNode("n2",100,100)    (FIRST_FIT)
submit("b",40,40,1)->n1; submit("a",60,60,1)->n1; submit("c",50,50,1)->n2; submit("d",10,10,1)->n2
removeNode("n1") -> ["a->PENDING", "b->n2"]     (retry order b then a: b takes n2's last 40, a no longer fits)
pending() -> [a]
```

### Preemption

```
addNode("n",1000,1000)
submit("lowA",400,400,10); submit("lowB",400,400,20)       (both run on n, 200 free)
submit("urgent",500,500,100) -> n                          evicts lowA only (lowest priority, 600 free is enough)
placement("lowA") -> "PENDING", placement("lowB") -> "n", pending() -> [lowA]

same setup but submit("urgent",500,500,99) -> empty        (priority < 100 never preempts)
two equal-priority victims "old","new" (400 each): urgent 500 evicts "new" (newest first)

n1 holds s1,s2,s3 (300 each, prio 1), n2 holds big (900, prio 50), submit("urgent",600,600,100)
  n1 would need 2 evictions, n2 needs 1 -> placed on n2, "big" becomes PENDING
```

## Edge cases

- Fit uses `>=` on both resources; the leftover for BEST/WORST_FIT is the plain sum of the two remaining amounts (no normalisation, use 64-bit arithmetic).
- A pending container is retried only by a retry pass (rule 9); a pending container with priority >= 100 never evicts anybody in a retry pass, only the `submit` call that created it could.
- A container that is larger than every node can never be placed or made room for; it stays PENDING (nothing is evicted for it).
- Evicting is all or nothing: if the best node cannot be freed enough, no container is evicted anywhere.
- A new container of priority >= 100 that fits somewhere is just placed by the strategy; nobody is evicted.
- A victim must have a priority strictly lower than the new container's: equal priority is safe, and priority-100 containers can be evicted by priority 101 and above.
- `removeNode` returns the state after rescheduling: a displaced container that fits another node is reported with that node. Displaced containers compete with already pending ones in retry order.
- `remove` of a pending container frees nothing and changes nothing else.
- Duplicate ids are rejected even when the first container is pending; an id is free again after `remove`.
- Queries on unknown ids are errors; `pending()` on an empty cluster is an empty list.

## Glossary

- **Node**: a machine with fixed CPU (millicores) and memory (MB).
- **Container**: a unit of work with a resource request and a priority; it is RUNNING on one node or PENDING.
- **Free capacity**: node total minus what its running containers use.
- **Leftover**: free CPU plus free memory that would remain on a node after placing a container.
- **Retry order**: priority descending, then submission order ascending.
- **Preemption**: evicting lower-priority running containers so a high-priority one can start.
- **Eviction / displacement**: a running container turning PENDING, because of preemption or because its node was removed.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `IllegalArgumentException` for every error: blank/null/duplicate ids, unknown nodes or containers, non-positive cpu/memory, null strategy. `submit` returns `Optional<String>`, strings in `removeNode` use the exact format `id->node` / `id->PENDING`, `nodeUsage` uses `used/total,used/total`.
- Rust: `Result<_, OrchError>` with `UnknownNode`, `UnknownContainer`, `AlreadyExists`, `InvalidArgument`; cpu and memory are `u32` (zero is invalid), priority is `i32`; `submit` returns `Result<Option<String>, _>`.
- Ids are compared and sorted as plain strings (the tests use ASCII ids).
- The placement/preemption/retry rules above are exact, including retry order, victim order and tie-breaks.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

public enum Strategy { FIRST_FIT, BEST_FIT, WORST_FIT }

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface Orchestrator {
    void addNode(String nodeId, int cpuMillis, int memMb);
    List<String> removeNode(String nodeId);                 // "id->node" / "id->PENDING", sorted by container id
    void setStrategy(Strategy strategy);
    Optional<String> submit(String containerId, int cpuMillis, int memMb, int priority);
    void remove(String containerId);
    String placement(String containerId);                   // node id or "PENDING"
    String nodeUsage(String nodeId);                        // "cpuUsed/cpuTotal,memUsed/memTotal"
    List<String> containersOn(String nodeId);               // sorted ascending
    List<String> pending();                                 // in retry order
}
```

The Javadoc in `src/main/java/contract/Orchestrator.java` repeats the per-method rules. Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static Orchestrator create(Strategy strategy)
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. There is no clock and no randomness. More scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- The three strategies, the leftover formula, tie-breaks by node id, exact fit, memory-bound placement, changing the strategy.
- Pending containers, retry after `remove` / `addNode`, retry order, no head-of-line blocking, immediate placement of new submissions.
- Node usage and sorted container queries, id reuse.
- `removeNode`: rescheduling, `PENDING` results, result order, priority competition.
- Preemption: lowest priority and newest victims first, only as many as needed, fewest-evictions node and tie-break, priority threshold 100, strictly lower priority only, nothing evicted when no node can be freed, both resources considered.
- Validation of nodes and containers, with state unchanged after an error.
- Concurrency: 16 threads submit into 4 nodes while another thread watches usage (never above capacity, exactly 400 containers placed); submits racing removals keep every node's usage equal to its containers.
- Harder scenarios (evicted containers re-placed, no preemption in retries, submission order after node failure, victim order across priorities, a long mixed sequence with invariants, concurrent churn) are in `interviewer/`.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/135-container-orchestrator test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`Candidate`, `Concurrently`): `java/testkit`.
