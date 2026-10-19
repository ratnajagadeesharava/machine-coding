# Hints — 81 Parallel Web Crawler (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

When can a worker that sees an empty queue safely stop? What if another worker is in the middle of a fetch?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Track the number of URLs that are queued *or being processed* (not just the queue size). The crawl is finished exactly when that number drops to zero. Make 'visited.add(url)' decide who gets to enqueue a URL.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shared state under one lock (or a concurrent set + counter): `visited`, `queue`, `pending`. A worker takes a URL (pending stays counted), fetches **outside** the lock, then under the lock adds each unseen link to `visited` and the queue and finally decrements `pending`; if `pending == 0` it wakes everyone to exit. Idle workers wait on a condition for 'queue non-empty or pending == 0'. Errors from the fetcher are caught per page and treated as 'no links'. Before returning, join all workers.

**Rust:** `std::thread::scope` with N worker loops over a `Mutex<State>` + `Condvar`; fetch outside the lock. `HashSet::insert` returning `bool` is the visited check - keep it inside the same critical section as the push.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
