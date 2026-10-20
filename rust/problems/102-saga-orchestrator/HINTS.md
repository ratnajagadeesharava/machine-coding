# Hints — 102 Saga Orchestrator (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

If your process died between two lines of `run`, what would a new process need to read from the log to know exactly what is left to do?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Write a record before and after each unit of work (step started / step done / compensation done / saga finished) and make `status` and `run` both interpret the same records. A per-saga guard (not a global lock) gives you the RUNNING behaviour.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Think of `run` as: load the saga's progress from the log; skip what the log says is done; then loop forward through steps with the retry rule, and if one fails for good loop backwards through the completed ones with the other retry rule, logging every result. The decision 'forward or backward, and from where' must be computable from the records alone.

**Rust:** A `Mutex<HashSet<String>>` of running saga ids (released with a drop guard even on panic) is enough for the guard; do not hold it while a step executes. Steps return only `bool` here, there is no exception path.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
