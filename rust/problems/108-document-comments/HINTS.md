# Hints — 108 Document Comment Service (Rust)

Try the problem for at least 10 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Which checks can you do before looking at any state, and what must be true about the id counter if a call is refused?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

The set of documents never changes after creation, so only the contents of each document need protection. Think about which two things must happen together for a comment to be ordered consistently with its id, and how a reader gets a copy that cannot be affected by later writers.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: a read-only lookup from document id to a small holder with its own ordered list of formatted comments; one service-wide counter. A write validates, finds the holder, and then takes the id and appends inside one critical section on that holder. A read copies the list inside the same protection and returns the copy.

**Rust:** build a `HashMap<String, Mutex<Vec<String>>>` once (it is never modified afterwards, so it needs no lock) plus an `AtomicU64`; take the id while holding the document's mutex, push, and clone the `Vec` for reads. Do not return a reference into the `Vec`.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
