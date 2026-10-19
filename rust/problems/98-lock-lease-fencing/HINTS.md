# Hints — 98 Lease Lock with Fencing Tokens (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

If a lease only means 'valid until a time', what must a storage server see on each write to tell a paused old owner from the current one?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Keep a tiny record per lock name: current owner, current token, expiry time, and the highest token ever issued. Evaluate expiry lazily by comparing the clock with the stored expiry on every call, so no timer thread is needed. Two threads on the same lock must not both think it is free.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

A call reads the clock once, then (under a lock that covers just this lock's record, or an atomic update of it) decides: free-or-expired -> bump the counter and grant; same owner and unexpired -> extend, same token; else refuse. `renew`/`release` check owner, token and expiry together before changing anything. Make sure the token counter lives longer than any single lease (it must survive release and expiry).

**Rust:** A `Mutex<HashMap<String, LockState>>` is enough for the tests; if you want less contention shard it or use `DashMap`-style per-entry locking by hand. Keep time arithmetic in `u64` and use `saturating_add` for `now + lease`.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
