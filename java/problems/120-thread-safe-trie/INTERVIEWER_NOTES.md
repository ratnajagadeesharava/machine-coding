# Interviewer notes — 120 Thread-Safe Trie

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, race condition, complexity problem, or wrong design.
2. Ask early: "What does thread-safe mean for each of these seven methods?" A candidate who answers "I put `synchronized` on everything" should be asked what that costs for 15 readers and one writer.
3. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
4. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- A randomised comparison with a `TreeSet` model, including upper-case and padded input: catches sloppy normalisation and wrong delete pruning.
- Delete everything in random order: the trie must report empty (`startsWith("")` false), accept new inserts, and not leak branches.
- 200,000 words inserted by 8 threads, then 20,000 listing queries: a global lock passes, but naive per-node locking with a lock-order bug or a recursive traversal that blows the stack does not.
- Deletes of `car` and `ca` racing with inserts of `card*` / `care*`: a delete that prunes the path without checking for children, or that races with an insert extending the same path, loses a sibling.
- Six words hammered by 8 threads: the net number of successful inserts minus deletes must equal the final size, which exposes lost updates in the size counter and in the "was it there" check.
- Rejected input inside concurrent work must not create nodes.

## Likely interviewer questions
- Which call is the linearisation point of `insert`? What if two threads both see "not present"?
- A reader walks `a -> p -> p` while a writer deletes `apple` and prunes the nodes: what does the reader see, and why is it safe?
- How does `wordsWithPrefix` get a snapshot? Does it hold a lock for the whole traversal? What does that do to writers when a million words match?
- How do you keep `size()` exact? A counter updated outside the structural lock can disagree with `contains` for a moment: is that acceptable for the contract here?
- Why did you pick this lock granularity? Where is the contention if all words start with the same letter?
- What is the memory cost of a node with a hash map of children versus a sorted array versus a fixed array of 37 slots?

## Invariants
- A word is present if and only if its terminal flag is set on the node reached by its characters.
- Every node below the root either ends a word or has at least one descendant that does (no dead branches, unless the design tolerates them knowingly).
- `size()` equals the number of terminal flags; `countWithPrefix(p)` equals the number of terminal nodes in the subtree of `p`.
- A rejected call changes nothing.
- A listing is sorted by character order and has no duplicates.

## Common mistakes
- `ConcurrentHashMap` children with a plain `boolean` terminal flag: individual reads are safe but `insert` of a word and `delete` of another word on the same path interleave and prune a live node.
- Check-then-act on the terminal flag without holding the node lock, so two racing inserts both return `true` and `size` goes wrong.
- A size counter updated with `count++` on a plain `int`, or updated before the structural change succeeds.
- `wordsWithPrefix` iterating live children maps, producing duplicates, skips, or a `ConcurrentModificationException`.
- Pruning on delete by walking up without re-checking that no concurrent insert added a child in the meantime.
- Hash-ordered children listed without sorting; sorting the final list but cutting it at `limit` before sorting.
- Recursion depth equal to word length (stack overflow at a few thousand characters in some setups, hence the 1,000 limit).
- Validating after mutating (a rejected insert leaves a half-built path).
- Treating `contains(" ")` as `false` instead of an error, or a blank prefix as an error.
- Locale-dependent lower-casing.

## Over-engineering traps
- A lock-free trie with CAS on every edge as a first step, without being able to explain safe memory reclamation and the delete race.
- A visitor / strategy / factory hierarchy for one data structure.
- Persistence, a background compaction thread, or an event bus for the core.
- Generic `Trie<T>` when the contract is strings only.

## Design and scaling discussion
**Patterns and structures an interviewer hopes to hear**
- Trie with a children map per node (hash map, sorted map, array indexed by the 37 symbols) and a terminal flag; a subtree word count per node makes `countWithPrefix` O(prefix length) and also makes pruning decisions local (count reaches zero, remove the node).
- Granularity options, from simplest: one lock for everything; a read-write lock (readers share, writers exclusive): correct and usually enough for this exercise; lock striping by first character (reads and writes take the stripe of the prefix; the empty prefix and `size()` need all stripes or an atomic counter); per-node locks with hand-over-hand locking (deadlock-free because order is root-to-leaf, but a lot of lock traffic and the root is hot); optimistic reads with versions or a `StampedLock`.
- Copy-on-write / persistent path copying: a writer copies the nodes on the path and publishes a new root with one volatile/atomic store; readers take the root once and traverse an immutable snapshot (this makes `wordsWithPrefix` a real snapshot for free); writers serialise on a mutex or retry with CAS on the root; cost is garbage per insert proportional to word length times node width.
- Snapshot semantics of `wordsWithPrefix`: under a lock it is trivially consistent; with fine-grained locking it is not, unless the traversal is bounded by `limit` and done under a lock covering the subtree, or the structure is immutable.
- Exact `size()`: an atomic counter incremented/decremented inside the same critical section that flips the terminal flag; or per-node counts.

**Readers that never wait (V4)**
- Copy-on-write root with an `AtomicReference` (Java) / `ArcSwap`-like pattern or `RwLock<Arc<Node>>` clone-then-release (Rust, std only); path copying cost; writer-writer serialisation; garbage and `Arc` reference-count traffic; epoch reclamation if raw lock-free is attempted.

**10M words in memory (V5)**
- Radix tree / compressed trie merges single-child chains; children as sorted small arrays with binary search or a bitmap plus packed child array; characters in `byte` arrays or a shared arena; node ids as `int` instead of references; deleting must also re-merge chains. Per-node locks become awkward once nodes split and merge, so locking moves to the subtree or stripe level.

**Wildcard (V2) / LCP (V3) / TTL (V6)**
- Wildcard is a DFS fan-out at every `.`; worst case touches the whole trie, so snapshot-under-lock hurts writers, which pushes towards copy-on-write. LCP walks down while a node has exactly one child and is not terminal, starting at the prefix node. TTL: store an expiry per terminal node, check lazily on read, purge on write or by a sweeper; counts per subtree complicate it (they must exclude expired words).

## Trade-off questions to ask when the candidate is done
- Read-write lock versus copy-on-write: what workload ratio makes each win?
- What would you measure to decide on the lock granularity? What would you expect for 90 percent reads, and for 90 percent writes?
- Is `size()` allowed to be slightly stale in a real product? What did the contract here force?
- How would the design change if words were Unicode?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
