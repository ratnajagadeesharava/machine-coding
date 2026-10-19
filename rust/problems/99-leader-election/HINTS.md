# Hints — 99 Raft Leader Election (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

What is the smallest set of facts a node needs to remember to refuse to vote twice, and to know that it is out of date?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Model the node as a small state machine driven by two events (a message arrived, a millisecond passed). Keep the term, your vote for this term, the role, and two timers: how long until you start an election, and (as a leader) when to send the next heartbeat and when you last heard from each peer. The rule 'a higher term always wins' fixes most of the tricky cases.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

On each tick decrement/compare deadlines: followers and candidates start an election when the election deadline passes; leaders send heartbeats on schedule and check that a majority replied recently. On each message, first compare terms (adopt and step down on higher; answer and ignore on lower), then handle by type: vote request (grant under the one-vote rule and reset the timer), vote reply (count toward a majority), heartbeat (reset the timer, remember the leader, reply), heartbeat reply (note the time for check-quorum). Draw each new timeout from your seeded generator in [150, 300].

**Rust:** Put the state in a `Mutex<State>` (the trait methods take `&self`), and release the guard before calling `send` if you want to stay safe against re-entrant delivery. Use a seeded PRNG you own (a tiny xorshift is fine); do not use `thread_rng`.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
