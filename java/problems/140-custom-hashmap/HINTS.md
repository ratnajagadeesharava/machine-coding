# Hints — 140 Custom HashMap (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Take a key, ask the hasher for a number, and turn that number into a slot of a fixed-size array. Two things can go wrong with that last step: the number can be negative, and several keys can land in the same slot. What must a slot be able to hold?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Let each slot hold a short list of entries that share it, and have every entry remember its own hash number so a resize can move it without asking the hasher again. For listing in insertion order, a table alone cannot help you: keep a second structure that threads all entries together in the order they arrived, and make sure removing an entry from it does not require searching.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: an array of buckets; each entry is a small node holding key, value, cached hash, a link to the next node in its bucket, and links to its neighbours in a doubly linked insertion-order list (with head and tail). The bucket index is the hash with its sign removed or masked and reduced by `capacity - 1` (capacity is a power of two). Insert: find the node; if present replace the value, otherwise append to the order list, push into the bucket and then check the growth rule with `long` arithmetic. Resize: walk the order list, recompute only the index from the cached hash, relink into a new array. Remove: unlink from the bucket and from the order list in O(1) once found. For atomicity one lock around every public operation is a valid first answer.

**Java:** `hash & (capacity - 1)` works for negative hashes because the mask clears the sign bit; do not use `hash % capacity`. Mixing the high bits into the low bits first (`h ^ (h >>> 16)`) helps weak hashers but is optional.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
