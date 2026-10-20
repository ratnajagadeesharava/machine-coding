# Hints — 97 Idempotent Request Processor (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

For one key, what are the possible situations a new call can find (nothing yet, someone is running it, a result is stored), and what should each do?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Keep one record per key: the body fingerprint plus a state (in progress, done with a response and a stored-at time). The in-progress state needs a way for other callers to wait for its outcome without blocking callers of other keys. Lock per key, not globally, and never while the handler runs.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Look up or create the key's record under a short critical section, then decide: caller creates it -> becomes the executor; record exists with another body -> conflict; record done and unexpired -> replay; record in progress -> wait. The executor runs the handler outside any lock, then publishes the result or removes the record and wakes the waiters, who re-check from the top (so one of them executes next after a failure). Think about when expired records are removed so memory stays bounded.

**Rust:** A `Mutex<HashMap<String, Arc<Entry>>>` held only for lookup/insert, plus a `Mutex`+`Condvar` inside each entry for waiting, avoids holding the map lock during a handler. Use `Result` to carry `HandlerFailed` out of the executing thread and drop every guard before calling the handler.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
