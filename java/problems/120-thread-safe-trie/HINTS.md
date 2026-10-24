# Hints — 120 Thread-Safe Trie (Java)

Try the problem for at least 25 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Forget threads for a moment. For the prefix `car`, which words do you ever need to look at, and what do they all have in common? Could the data be laid out so that finding them costs time proportional to the length of the prefix rather than to the number of words? Then ask the second question: for each of your seven operations, what is the single moment at which it "happens", and what must nobody else be able to observe before that moment?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

A prefix tree (trie) shares the characters of common prefixes, so every prefix is a place you can walk to. Keeping a small amount of bookkeeping on each node (is a word complete here, how many words are below) answers `size`, `countWithPrefix` and "can I remove this node now" without scanning. For thread-safety, start with the coarsest scheme that is clearly correct (one lock, or a read-write lock where readers share) and get all tests green; only then ask which operations really conflict and what a finer scheme would cost. Decide deliberately what `wordsWithPrefix` holds while it walks, because that is where "consistent snapshot" is won or lost.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: a normaliser/validator used by every public method before any state is touched; a node with `children` (ordered by character, so listing is a plain depth-first walk), a `terminal` flag and a `wordsBelow` counter; `insert` = walk and create nodes, flip `terminal`, increment counters along the path, all inside one critical section, returning whether the flag changed; `delete` = check terminal, then decrement counters along the path and detach any child whose counter reaches zero; `wordsWithPrefix` = walk to the prefix node, then in-order DFS that stops at `limit`. Alternatives to compare once it works: striping by first character, per-node locks, or immutable nodes with path copying and an atomically published root.

**Java:** `ReentrantReadWriteLock` or `StampedLock` give readers/writer separation; `TreeMap` or `ConcurrentSkipListMap` keep children ordered; `AtomicReference<Node>` publishes an immutable root. Normalise with `toLowerCase(Locale.ROOT)`. A `ConcurrentHashMap` child map makes single calls safe but does not make a multi-step insert or delete atomic.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
