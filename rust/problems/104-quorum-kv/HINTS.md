# Hints — 104 Quorum Key-Value Store (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

A write reached a majority and a read asks a majority: why do those two sets always share at least one node, and what must the reader do with the different answers it gets from them?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Per request keep a small record: which nodes answered, with what, when it started, when you last re-sent. Per node keep key -> (version, value). Acks and answers are counted per distinct node (a set), not as a counter, so duplicates cannot fake a quorum.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

On tick, walk the open requests: re-send to the silent nodes every 100 ms, mark FAILED when 1000 ms passed without a majority, and forget the request after the deadline. On a get response, compare versions; once a majority answered fix the reply, but keep the request alive for late answers so you can send read-repair writes. Choose a message kind per case (write, ack, read, read-answer) and make every handler idempotent.

**Rust:** Derive `PartialOrd`/`Ord` on a `(u64, String)` version tuple so comparison is one expression; keep requests in a `BTreeMap` for deterministic iteration and send only after the state update is finished.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
