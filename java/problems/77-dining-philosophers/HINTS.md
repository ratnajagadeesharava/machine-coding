# Hints — 77 Dining Philosophers (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Deadlock needs a cycle of waiting. Where is the cycle in 'everybody takes the left fork first'? What is the smallest change that breaks it?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Give the forks a global order and always take the lower-numbered fork first (the last philosopher is the 'odd one out'). Alternatively let at most N-1 philosophers sit down at the same time.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Forks `0..n-1`, philosopher `p` uses forks `p` and `(p+1)%n`. Lock `min(a,b)` first, then `max(a,b)`, run `eat`, unlock in reverse in a `finally`/drop guard. For n=2 both philosophers use the same two forks - check that the ordering still holds. If you want a stronger fairness argument, look at an arbitrator (a semaphore of size n-1) or per-philosopher state + condition variables.

**Java:** `ReentrantLock[]` for forks. `lockInterruptibly()` keeps `dine` interruptible. A `Semaphore(n - 1)` is the other classic.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
