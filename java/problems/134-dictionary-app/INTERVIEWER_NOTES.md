# Interviewer notes — 134 Dictionary App

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Suggestions: insertion, deletion and substitution all count; two edits and swaps do not; the first `k` are taken in alphabetical order, not insertion order; a known word returns nothing.
- Word of the day with `Long.MIN_VALUE` / `Long.MAX_VALUE`: `Math.abs` or `%` on a negative gives a wrong or negative index. The answer changes when words are added or removed.
- `updateMeaning` must not behave like `addWord` for a different part of speech, and the old text becomes free again afterwards.
- Search must match inside one meaning, not across the boundary between two meanings of the same word, and must list a word once.
- Removing the last meaning must remove the word and its synonym links (a re-added word has no synonyms).
- Adding a synonym to a removed word fails; linking `(a, b)` and `(b, a)` from different threads must not deadlock.
- Casing and padding: `" APPLE\t"` is the same word everywhere, including in `addSynonym`.
- A word is accepted only if it has valid characters after trimming; an invalid `addWord` must not leave an empty word behind (check the order of validation and creation).

## Likely interviewer questions
- Which structure answers `prefix` without scanning every word? What about `wordOfTheDay` by index: what does it cost, and how would you make it cheaper?
- Where is the normalisation done, and why are reads normalised too?
- How do you store a symmetric link: two sets, or one set of unordered pairs? What does removal cost in each?
- How do you find words within edit distance 1 of an unknown word without comparing it with every word?
- How do you make `addWord` for the same word from 16 threads lose nothing while different words proceed in parallel?

## Invariants
- A word exists only while it has at least one entry.
- A word's entries are in insertion order, with no two equal `(part of speech, text)` pairs.
- Synonym links are symmetric, never self links, and only between existing words.
- Every sorted answer uses the same order (plain character code).
- `size()` and `meaningCount()` equal what a full walk would count.

## Common mistakes
- Creating the word before validating the part of speech or meaning, leaving an empty word behind.
- `String.toLowerCase()` without a locale, or accepting non-ASCII letters that lower-case to ASCII ones.
- Using a `HashMap` and sorting on every query, or sorting with a locale-aware collator so `'` and `-` order differently from the contract.
- Storing links on one side only, so `synonyms` is not symmetric, or forgetting to clean the other side on removal.
- Deadlock from per-word locks taken in argument order in `addSynonym`.
- Returning the internal entry list, so a later edit changes a list the caller already holds.
- Treating a transposition as distance 1, or treating equal length strings differently from the other cases.
- `index` checks that allow `size` itself, or negative indices accepted by a list implementation that wraps.
- Duplicate check in `updateMeaning` that compares the entry with itself.

## Over-engineering traps
- A BK-tree or n-gram index before the simple scan works (it is a good V4 discussion, not a requirement).
- A class hierarchy per part of speech, or a visitor over entries.
- A persistence layer or event log in the core.

## Design and scaling discussion
- Sorted map (red-black tree) keyed by word gives `prefix` as a range scan and the sorted order for free; a trie gives the same with a different cost profile. `wordOfTheDay` by index needs an order-statistic structure or a rebuilt sorted array if the cost matters.
- Concurrency: a single read-write lock is the honest baseline; sharding by word works for entries but synonyms and `size` cross shards. Copy-on-write snapshots suit read-mostly dictionaries.
- Suggestions: scanning is O(words x length); for distance 1, generating all edits of the input and probing a hash set is O(26 x length^2) hash probes and independent of dictionary size, but returns candidates unsorted; a BK-tree or a deletion-neighbourhood index handles distance 2+ (V4).
- Search by meaning text: a scan is fine for a small dictionary; an inverted index over tokens trades memory for speed but then substring matching is no longer exact.

## Trade-off questions to ask when the candidate is done
- Is it right that removing the last meaning removes the word? What would a word with no meanings mean for `size()`, `prefix` and `wordOfTheDay`?
- Where would you put the "part of speech" rule if a new type (say `PRONOUN`) arrived?
- What are the costs of storing synonyms on the word versus in a separate graph structure?
- How would you test the thread-safety claim without a flaky test?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
