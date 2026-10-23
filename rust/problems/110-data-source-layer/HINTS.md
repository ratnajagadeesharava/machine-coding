# Hints — 110 Runtime Data Source Access Layer (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Make a list of everything that is different between the three sources. For each difference, who should be the only one in your code that knows about it?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Treat "the current source" as one value that every call reads exactly once, then does all of its work on what it read. A switch only replaces that one value; it never touches records. Each source keeps its own records, so there is nothing to copy or clear.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: the entry point holds a name-to-source lookup and one pointer to the current source. A source owns its record store and its rules (key normalization, value check). `get`/`put`/`remove`/`size` validate the key (blank check first), take the current source once, and delegate. `switchSource` looks the name up, throws/returns an error if absent, and only then replaces the pointer. A refused value is checked before the store is modified.

**Rust:** `RwLock<Arc<Source>>` (or `ArcSwap`-style) lets a call clone the `Arc` and release the lock before working; give each source its own `RwLock<HashMap<String,String>>` or `Mutex`, and normalize the key inside the source before the map lookup.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
