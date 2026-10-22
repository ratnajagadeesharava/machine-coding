# Interviewer notes — 115 Autocomplete / Typeahead Service

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.
4. If this is a follow-up round on an earlier prefix-search or autocomplete exercise, let them reuse that code and ask what they would change.

## Hidden edge cases (see `interviewer/`)
- A low-ranked term is bumped past the leaders: a per-node top-k that is only updated when a term is already inside it goes stale.
- A thousand terms with the same count: the alphabetical tie-break must hold at every prefix and for every k.
- Recency: bulk records and `recordFor` also refresh recency; equal clock instants tie alphabetical.
- Rejected calls (`count = 0`, blank user, blank term) must not create the term, a user entry or a node.
- A model-based check against brute force on random operations with upper-case input.
- Long terms (2,000 characters) and accented letters ("ÉCOLE" equals "école").
- Hot term updated by 8 threads while 4 threads read the same prefix; per-user counts exact under concurrency.
- 200,000 terms and 20,000 queries.

## Likely interviewer questions
- Walk me through `suggest("ap", 5)`: which nodes do you touch, how many results do you look at?
- You store the top-k per node: what is k, and what does a request with a bigger k than you stored do?
- When a term's count changes, which nodes change? What is the cost of that, and when is it done (write time or read time)?
- How does a reader avoid seeing a half-updated top-k list?
- Why alphabetical as the final tie-break? What breaks if it is missing (flaky tests, unstable UI, cache keys)?
- How is the personalised query different from the global one? Do you copy the trie per user?
- Why is `setRanking` cheap or expensive in your design? Do you keep a ranking per rule?

## Invariants
- Count of a term equals the sum of all accepted records of it, regardless of interleaving.
- For every prefix, the answer equals "all terms with that prefix, sorted by (rank desc, term asc), first k".
- A user's count never exceeds the global count of the same term; a personal record also counts globally.
- A rejected call changes nothing.

## Common mistakes
- Case or whitespace handled for terms but not for prefixes (or the other way round).
- Scanning every term on each query, or sorting every descendant of the prefix node on each query.
- Top-k lists per node that are never re-sorted after a count update, or that only insert when the term is already present.
- Forgetting the alphabetical tie-break, or applying it ascending for one rule and descending for another.
- `HashMap` or a plain trie child map mutated under a read lock; `count++` on a plain `long`.
- Treating the recency tie as insertion order instead of alphabetical when two records share a clock instant.
- Creating the term before validating the count, leaving a ghost term behind.
- Personalised path that ignores the active ranking when user counts tie.
- Locale-dependent lower-casing (Turkish dotless i).

## Over-engineering traps
- Persistence, a database or a message bus for the core.
- A strategy class per ranking rule plus a factory before a second rule is needed (acceptable at V4, not before).
- A ternary search tree / FST / double-array trie as a first step when a plain trie or sorted structure passes.
- A per-user copy of the whole structure.

## Design and scaling discussion
**Patterns and structures an interviewer hopes to hear**
- A trie (or a sorted map with range scans) keyed by prefix; top-k cached per node versus a heap over the matching subtree: write-time versus read-time cost.
- Strategy for the ranking rule (a comparator or score function over a term record: count, last-recorded time), so a new rule is a new class and no existing code changes (V4). The personalised mode composes a user score in front of the active rule (decorator / composite comparator).
- Immutable snapshot of a node's ranked list swapped atomically, or per-node locks / striping; atomic counters for counts. Lock-free counting with a read-path that tolerates slightly stale top-k is a valid answer if it is stated, but the visible-immediately requirement means the next call must see the new count.
- Per-user history as a separate map of small counters keyed by (user, term); re-rank the global candidates with the user's score at read time.

**Updating counts without locking everything (V2)**
- Contention is on the nodes along the path of the term, and mostly at the root and the first levels. Hand-over-hand or per-node locks hurt at the top; striping by first characters, or a copy-on-write list per node, is better; or make counts atomic and recompute top-k lazily with a version stamp.
- A write only needs to change the top-k of ancestors where the term enters, leaves or moves inside the list; most writes to a cold term touch nothing.
- Batching writes through a queue and applying them in the background gives throughput but breaks "visible immediately" unless reads also consult the pending buffer.

**10M terms in memory (V3)**
- Naive: a node object with a hash map of children and a list of strings costs several hundred bytes per node; ten million terms of 20 characters is tens of gigabytes.
- Ideas: compress single-child chains (radix tree), store keys as byte arrays or in a shared character arena, sort children into arrays instead of maps, intern terms once and keep top-k as `int` ids into a dictionary, 32-bit counts with saturation, drop top-k on deep nodes (compute from subtree), shard by first characters, frozen read-only segments rebuilt periodically plus a small mutable delta, FST for the static part.
- The trade is freshness versus memory; ask how they would measure it (heap dump, a synthetic 10M load, bytes per term).

**Adding a ranking rule (V4)**
- A rule is a function from a term record to a score (or a comparator). Rules that depend on time alone (decay) cannot be precomputed in a per-node list that is only updated on writes; they need a read-time re-rank of a larger candidate set, or a score with a reference time that is monotone with the order (store `log(count) + t/halflife`).

## Trade-off questions to ask when the candidate is done
- Precompute top-k on write or compute on read? Which is better for a read-heavy search box, and how does the answer change with a write-heavy firehose?
- What staleness would a product tolerate for suggestions? What changes if a stale suggestion is allowed for 5 seconds?
- Would you cache hot prefixes in front of the structure? How do you invalidate them on a write?
- Should personalised counts be capped or decayed? Where is the privacy boundary of a per-user history?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.
