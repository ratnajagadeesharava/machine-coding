# Hints — 113 Multi-Module Logging Framework (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Which single place in your design is allowed to say "this is number N", and what does it need to know before it may say it?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Let producers only hand work over and wait for an outcome; let one owner take the work in buffer order, try the write, and only on success give out the next number and record the log. Think about what the waiting producer waits on, and what must happen first so that the log is visible by the time it wakes up. For shutdown, think about who is still waiting when you stop accepting work.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `log()` validates, wraps the input with a one-shot completion, puts it in a bounded blocking queue (this is the back pressure) and waits on the completion. One consumer thread loops: take a request, build the line with the next number, call the sink inside a try/catch; on success append the entry to an append-only store in sequence order, advance the counter, then complete the request with the number; on failure complete it with the failure and leave the counter alone. `query` validates filters, then scans/binary-searches the store under a short lock and returns a copy. `close()` stops new entries from being enqueued, lets the consumer drain what is already queued (for example by closing the sending side), and joins the thread.

**Rust:** `std::sync::mpsc::sync_channel(cap)` is already a bounded queue; send a request carrying a reply `Sender`, and have the caller `recv` on its own reply channel. Dropping the last sender ends the consumer's loop after it drained the queue, which is one way to shut down; keep the sender in an `Option` behind a lock so `close` can take it. A `Mutex<Vec<_>>` store is enough for `query`.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
