# Hints — 105 Raft Log Replication (Java)

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

Write one small handler per message type plus one per timer, and let every handler begin by comparing terms. The leader keeps, per follower, how far it believes the follower matches; the commit decision is a function of those numbers and of the terms in the log. The Raft paper's Figure 2 is the checklist - turn each rule in it into a method and test each against duplicated and stale messages.

**Java:** Keep `log` as a `List<Entry>` with a dummy entry at index 0 so indexes are 1-based, and draw the election timeout from a `java.util.Random` seeded with the constructor seed. Do not mutate the list inside a loop over it; the tests call getters from an observer after every millisecond.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
