# Hints — 101 Two-Phase Commit (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

At which exact moments must a node write to its log so that, whatever instant it crashes at, it can come back and not contradict something it already told another node?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Model each side as a small state machine per transaction (coordinator: preparing -> committed/aborted; participant: unknown -> prepared -> committed/aborted). The log is the only thing that survives a restart, so the in-memory state must be rebuildable by replaying it. Retries are driven by `onTick` with timers per transaction.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Coordinator: on `begin` log, send, remember when it started; on a vote update the tally; on decision log first, then send and keep resending until every ack; on restart replay the log and fill in the missing decisions. Participant: on PREPARE decide the vote (locks, rule), log PREPARED, then answer; on DECISION apply, release, log, ack; while in doubt ask the coordinator periodically. Every handler must tolerate duplicates.

**Rust:** Keep the per-transaction state in a `HashMap` inside a `Mutex`, and send messages after you finished updating the state (collect them first, send after releasing or at the end). Use `BTreeMap`/sorting for `in_doubt` output.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
