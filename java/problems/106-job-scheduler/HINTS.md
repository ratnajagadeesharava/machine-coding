# Hints — 106 Job Scheduler (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

For one submission, which machines are even allowed to compete, and what is the only thing the two criteria have in common when they look at a machine?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Treat "how to rank two eligible machines" as one replaceable piece of behaviour with the same inputs for every criterion, and keep the tie-break out of the individual criteria so it cannot be forgotten. Choose-and-record must happen as a single step with respect to other callers.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: a `Machine { id, capabilities (immutable copy), unfinished, finished }`, a registry of machines, a map `jobId -> (machine, finished?)`, and a criterion that can compare two machines (or produce a sort key). Submission = filter by "capabilities contain all required", pick the minimum under the criterion with the id as final tie-break, increment, record the job; all under one critical section. Finish = look up the job, check its state, flip it, move one unit between the counters.

**Java:** a `Comparator<Machine>` per criterion (chained with `thenComparing(Machine::id)`) and a `Map<String, Comparator<Machine>>` from criterion name keeps a third criterion to one entry; a single `synchronized`/`ReentrantLock` around submit and finish is enough to start with.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
