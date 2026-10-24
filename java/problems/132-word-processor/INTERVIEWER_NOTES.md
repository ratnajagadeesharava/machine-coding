# Interviewer notes — 132 Word Processor

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## What a strong answer looks like
- **Flyweight for formats**: a `Format` is an immutable value (equality by fields) obtained from a pool/interner, so a million characters share a handful of objects and `distinctFormatCount` is the number of live pool entries or a set over runs. Say it out loud: the format is intrinsic state, the position is extrinsic.
- **Run-length model**: the document body is a sequence of runs (length + format) rather than one entry per character. This makes the 1,000,000-character scale test trivial and makes `runs()` almost free. Splitting at a position, replacing formats in a span and re-merging neighbours are the three primitives everything else is built from.
- **Composite for the document**: document of paragraphs/lines of runs is a natural composite if the candidate wants alignment per paragraph; it is optional here, but it is the shape that survives tables (V3). Accept a flat text buffer plus a run list plus a line-alignment list if invariants are well kept.
- **Command** appears in V5 (undo/redo): each edit becomes an object that can apply and reverse itself. Do not ask for it in the base problem.
- A single lock around the whole document is a perfectly good answer; reads (`runs`, `text`) must be under the same lock as writes.

## Hidden edge cases (see `interviewer/`)
- A random sequence of 1,500 operations (insert, delete, format, replace, align) compared against a naive per-character model, five seeds.
- 20,000 characters formatted in alternating one-character spans (20,000 runs), then reset: must collapse to one run and one distinct format.
- Delete across several runs and lines: formats and alignments of the survivors.
- `replaceAll` whose replacement contains newlines, then one that removes them: alignment inheritance and merging.
- Every failing call leaves text, runs, line count and alignment untouched.
- Pattern longer than the text; replacement longer than the whole text; empty document searches.
- Replace in a million-character document with only a few matches (a per-match O(n) rebuild is fine, a per-character object model is slow and heavy).
- Concurrent inserts at both ends plus formatting plus readers: final length and line count are exact, runs describe the same length as the text.

## Likely interviewer questions
- What do you store per character? What does a million-character document of one format cost in your design?
- How do you keep `runs()` maximal? Do you merge eagerly after every edit or lazily when reporting? What does each cost?
- Why does position 0 differ from "the character before the position"? What did you do for an empty document and after deleting everything?
- Explain `insert` at the boundary between a bold run and a plain run, and inside a bold run.
- What is the complexity of `applyFormat` in characters, in runs, in lines?
- How do `find` (overlapping) and `replaceAll` (non-overlapping) differ, and why must replacements be applied from right to left, or with an offset, when their length differs from the match?
- What happens to alignments when a replacement contains a newline?
- What does a reader see if it calls `text()` and `runs()` while a writer runs? Is that one lock acquisition or two, and does it matter for the tests?

## Invariants
- The sum of the run lengths equals the text length, no run is empty, and no two adjacent runs have equal formats.
- Number of alignment entries equals the number of newlines plus one.
- A format value never changes after it is created (changes create or look up another value).
- A failed call has no visible effect.

## Common mistakes
- One `Format` object (or struct) per character: correct but memory-heavy and slow for the scale test; also makes `distinctFormatCount` an O(n) set build.
- Reporting runs without merging equal neighbours after a delete or a format change.
- Inserting at 0 with the format of the (new) first character, or reading `pos - 1` at 0 and crashing.
- Comparing formats by reference instead of by value (Java), so identical formats are never merged.
- `applyFormat` that applies the size and then throws on a bad colour: partial application.
- Letting `size="+5"` or `" 12"` through, or accepting `"TRUE"` for a flag.
- `find` that skips ahead after a match (non-overlapping) or `replaceAll` that allows overlapping matches (`"aaa"` replaced as two matches).
- Replacing left to right with stale indexes after the first replacement changed the length.
- Rescanning the replacement text (`"ab" -> "abab"` loops forever or double counts).
- Replacement taking the format of the preceding character rather than of the first character of the match.
- Counting words with `split(" ")` (empty strings, tabs, newlines) or treating `'\r'` as a word character.
- Alignment list not updated by `delete` and `replaceAll`, so later lines have the wrong alignment or the list goes out of bounds.
- Locking writers only: `runs()` throws a concurrent modification error or returns torn data.
- Reading `length()` then inserting at that position as if it were atomic (the tests insert at stale positions on purpose; it must still not fail or lose characters).

## Over-engineering traps
- A full rope, piece table or B-tree before a run list is shown to be too slow.
- A class per formatting attribute (`BoldDecorator`, `ItalicDecorator`) stacked on every character.
- An observer/event system, a layout engine, rendering, fonts metrics: none are asked for.
- A visitor hierarchy over document nodes to implement five queries.

## Design and scaling discussion
- Run list in an array or `ArrayList` gives O(runs) edits and O(log runs) lookup with prefix sums; a balanced tree or skip list keyed by offset gives O(log runs) everything. For real documents (long, many runs) a piece table or rope is the usual answer; the memory argument is the same: share formats, store ranges.
- The text and the formats can be separate structures (a text buffer and a run list that must stay the same length) or one structure of text chunks that each carry a format. The first has simpler search, the second simpler edits.
- Interning formats: `Map<FormatValue, FormatValue>` or a canonical-constructor cache; weak references if formats come and go; the pool is shared mutable state and needs the same lock or a concurrent map.
- Lines: store alignments in a parallel list indexed by line, or make each paragraph an object that owns its text and runs (composite). The second makes tables and headers easier.
- Concurrency: a single `ReentrantLock` or `synchronized` is enough here; a read-write lock helps read-heavy use; collaborative editing (operational transformation, CRDTs) is a different problem that changes the position model.

## Trade-off questions to ask when the candidate is done
- If documents were 100 MB, what changes? Which operations become too slow first?
- Would you store formats as a delta from the style (V2) or as full values? How does that affect `runs()` equality?
- How would you make `replaceAll` one atomic step for undo (V5)?
- How would you test the maximal-run invariant without writing the expected output by hand each time?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
