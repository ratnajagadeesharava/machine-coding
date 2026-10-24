# Hints — 135 Container Orchestrator (Rust)

Try the problem for at least 20 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Write down what state changes when a container starts, stops, is evicted, or when a node disappears. Is there one operation that all of those can be expressed with, so you do not have four slightly different versions of the bookkeeping?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Selecting a node is a swappable policy over the same list of fitting candidates, so the three strategies differ only in how they compare candidates. Keep the pending containers in a structure ordered by the retry order, and make "retry" one walk over it that is called from every operation that frees capacity. Keep all of the state behind a single lock first, and only then ask whether finer locks are worth it.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `Node { total, used, containers (sorted) }` and `Container { request, priority, seq, node? }` in two maps; a strategy is a comparator (or function) over the fitting nodes with node id as final tie-break; `place(container)` and `unplace(container)` are the only places that touch `used`. `pending` is an ordered set keyed by `(-priority, seq)`. Every public mutation does its work, then calls `retryPending()`, inside one critical section. Preemption computes, per node, the minimal prefix of the evictable list (sorted by priority asc, seq desc), picks the node with the shortest prefix, and only then mutates.

**Rust:** a `BTreeMap<String, Node>` gives the sorted node walk; a `BTreeSet<(Reverse<i32>, u64, String)>` (or a sorted `Vec`) for pending; one `Mutex` around the whole state, taken once per public method, is enough to satisfy the tests. Avoid calling a locking public method from inside another one.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
