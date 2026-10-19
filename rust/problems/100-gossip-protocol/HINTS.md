# Hints — 100 Gossip Protocol with Failure Detection (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

If every node only knows a value it was told about, what must be true about two states you merge so that the order in which you merge them does not matter?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Think of the state as a map from key to (version, writer, value) plus a map from node to heartbeat counter. Both merge entry by entry with a simple 'take the bigger one' rule. For liveness you need, per peer, the highest counter seen and the local time at which it last grew.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Per tick: if a round is due, bump your counter, choose peers using your seeded generator, send the serialized state. On a message: merge, and reply only if the message was a push (so the message needs a kind). Alive = peers whose 'last grew' time is within the window. Pick the serialization and the kind marker yourself.

**Rust:** The trait takes `&self`, so keep the state in a `Mutex<Inner>` and your own small PRNG (or a seeded xorshift) inside it; do not use thread-local or OS randomness. `BTreeMap` keeps iteration order deterministic.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
