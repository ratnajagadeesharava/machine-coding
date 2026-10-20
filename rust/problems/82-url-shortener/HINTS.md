# Hints — 82 URL Shortener (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

What two lookups do you need so that both 'URL -> code' and 'code -> URL' are fast, and what has to be true about them at all times?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Keep two maps that always agree, and a counter that produces fresh codes. The interesting part is making 'check whether it exists, then insert' a single atomic step, for URLs and for aliases alike. Think about what a code looks like if it is a number written in a larger alphabet.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: one authoritative record per code (URL, click counter, deleted flag or removed from the map) plus a reverse index from URL to code. Generated codes come from a monotonically increasing counter rendered in base 62; that counter is never decremented, which is what makes deleted codes unreissuable. An alias claims a slot in the same code namespace, and the generator must skip a slot an alias already took. Clicks use their own counter so resolves do not need the global lock.

**Rust:** `Mutex<Inner>` (or `RwLock`) around two `HashMap`s plus a `u64` counter is enough; clicks can be `AtomicU64` inside an `Arc` if you want resolves to avoid the write lock. Return owned `String`s, never references into the maps.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
