# Hints — 83 Feature Flag Service (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

For one user and one flag, in what order do you look at the pieces of configuration, and which piece makes the other ones irrelevant?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Hold each flag's configuration as one immutable snapshot that readers pick up in a single step, so evaluation never needs to wait on writers. For the rollout, compute a number from the user and the flag name that does not change over time and compare it with the percentage.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `Flag { default, rules (ordered), rollout (optional percent), killed }` stored as an immutable value in a concurrent map keyed by flag name; every edit builds a new value from the old one and swaps it in atomically. A bucket is a stable hash of `flag + user` reduced to 0..99; a user is inside when `bucket < percent`. The hash must be well mixed, otherwise the independence and accuracy checks fail.

**Rust:** `RwLock<HashMap<String, Arc<FlagConfig>>>` lets readers clone the `Arc` and release the lock before evaluating. Do not use `DefaultHasher` with a random seed for buckets (`RandomState` differs per process); write or import a fixed hash such as FNV-1a.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
