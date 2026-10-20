# Hints — 91 Online Auction (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

What is the one fact that every bid must be compared against, and what must happen between reading it and updating it?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

The decision 'is my bid higher than the current top, is the auction open, should I extend the end?' and the update of top bid, history, end time and the outbid notification must be a single atomic step per auction. Per-auction state is a natural unit of locking; compute 'ended' from the injected clock rather than a timer.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Keep one record per auction (seller, start, reserve, end, top bid, accepted-bid list, cancelled flag) behind its own lock, in a map from id to record. `placeBid`: lock the record, check state (cancelled/ended -> error, seller -> error), compare with `max(start, top+1)`, then append to history, set top, extend the end if `end - now <= 5 min`, and push an outbid message for the previous leader if it is a different person. Notifications are a per-user append-only list, so they need their own safe container.

**Rust:** `RwLock<HashMap<String, Arc<Mutex<Auction>>>>` or a single `Mutex` first; the notifications map can live in its own `Mutex<HashMap<String, Vec<String>>>`. Read the clock once per call and keep amounts as `u64`; avoid holding two locks at the same time to stay deadlock free.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
