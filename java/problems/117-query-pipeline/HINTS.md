# Hints — 117 Search Query Pipeline (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Look at how the built-in stages and a stage supplied by a test are described in the requirements. Could the code that runs the flow treat them identically, without knowing which is which?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

This is a pipeline of steps, each taking the current query and returning the next one. Keep the ordered list of steps (name, behaviour, enabled flag) as one immutable value that `process` reads once at the start, and replace it whole when someone registers, removes or toggles a step. Failure handling and trace recording belong to the code that runs the list, not to the steps.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `Step { name, transform }` (chain of responsibility / pipeline). Built-in query-phase steps (parse, spell, synonyms, filter) implement the same interface as test steps and sit in a list `[name, step, enabled]`. A runner loops: skip disabled, call `transform` inside try/catch, record the name when the output differs, record `name:failed` when it throws or returns null, stop on an empty query. Rank and highlight run after the loop and need a parsed form of the query (terms made of one or more words) that filter and rank can share. The list lives in a volatile/atomic reference to an immutable list; editors copy, modify and swap under a lock.

**Java:** `volatile List<Slot>` replaced under a `synchronized` block (or `AtomicReference` with compare-and-set), `List.copyOf` for the swap, records for `Slot` and `Result`, a `TreeMap` for the corpus so ties by id come out in order.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
