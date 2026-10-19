# Hints — 76 Ordered Printer (Turn Taking) (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Which question does a waiting thread ask itself each time it wakes up? Where does the answer live, and who is allowed to change it?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

One shared 'whose turn is it' value protected by a lock, plus a way to sleep until it changes. Always re-check the value in a loop after waking up. The hand-over is: change the value, then wake the waiters.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Keep a `turn` field under one lock and one condition variable. `takeTurn`: lock; `while turn != me: wait`; run the action; in a `finally`/drop-guard set `turn = (me + 1) % n` and `signalAll` / `notify_all`; unlock. Decide whether the action runs while you still hold the lock (simple, but think about what that means for the action) or after releasing it (then think about who may run next).

**Rust:** `Mutex<usize>` + `Condvar`. Use `while` with `cv.wait(guard)`; a guard struct whose `Drop` passes the turn makes the panic case trivial.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
