# Hints — 105 Raft Log Replication (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

If a leader could be wrong about being the leader, what makes a new leader's log safe to build on - and what must a voter check before it trusts a candidate?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Split the node into the same pieces the paper does: persistent-ish state (term, vote, log), volatile state (commit index, and on the leader nextIndex / matchIndex per peer), and three timers (election, heartbeat, check-quorum). RPC handlers must be written so that a duplicated, delayed or old message changes nothing.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

RequestVote: reject lower terms, adopt higher ones, grant if you have not voted this term and the candidate's (lastTerm, lastIndex) is at least yours. AppendEntries: reject lower terms, reset the election timer, check prev, append only on conflict, advance commit. Leader: after each reply update matchIndex / back off nextIndex, then compute the highest N stored by a majority whose entry is from your own term. The paper's Figure 2 is the checklist; write each rule as a small method.

**Rust:** Use a `Vec<Entry>` with a sentinel entry at 0 so indexes are 1-based; keep everything in one `Mutex<Inner>` and collect outgoing messages in a Vec, sending them after the state update. Your own xorshift/`Rng` seeded from `seed` for the timeouts.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
