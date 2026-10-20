# Interviewer notes — 86 Document Versioning with Merge

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Same change made twice by two authors: merged text equals latest, no new version, `latest` returned.
- Two insertions at the same place conflict, even with different text.
- Adjacent-line changes (line 2 on one side, line 3 on the other) merge cleanly.
- A stale edit against a base that is not 1 (base 2 with latest 3).
- An insertion before a line combined with a deletion of another line far away.
- Content with trailing newline: the last line is empty and must survive the round trip.
- Edit with `base == latest` and unchanged content creates no version; restore of the latest content creates none either.
- A failed merge must leave history untouched, even under 8 threads fighting for the same line.
- 200 versions of a 2,000-line document: diff and merge are fast.

## Likely interviewer questions
- Why is `edit` a compare-and-append? What goes wrong if you read latest, release the lock, and append?
- How do you decide two changes conflict? Is 'adjacent' a conflict in your rule, and why?
- What is a hunk? How do you apply two sets of hunks to the same base without offsets going stale?
- What does the diff do with moved lines? Is that a problem?
- Why store full copies of every version first? When would you switch to deltas?

## Invariants
- history length equals `latest`, and entry i starts with `i:`.
- A version's content never changes after it is created.
- A failed (conflicting or invalid) edit creates nothing.
- A merged version contains every non-conflicting change from both sides and nothing that neither side wrote.

## Common mistakes
- Reading `latest`, merging outside the lock, then appending: a concurrent edit is silently overwritten.
- Merging by comparing line numbers of the new text instead of base positions, so insertions shift everything.
- Treating adjacent changes as conflicts, or treating overlapping changes as clean.
- Creating a duplicate version when both authors made the identical change.
- Dropping the trailing empty line on split, or changing the number of lines on join.
- Putting additions before removals inside a changed region of the diff.
- A quadratic LCS table for 2,000 lines times 200 versions with no prefix/suffix trimming.

## Over-engineering traps
- A pluggable `MergeStrategy`/`DiffAlgorithm` hierarchy before one merge works.
- Word- or character-level diff, rename detection, or moved-line detection.
- Delta compression and persistence before the in-memory model passes.

## Design and scaling discussion
- Optimistic vs pessimistic concurrency: no document locks while editing, only compare-and-append at save time. This is how git push and database row versioning work.
- Three-way merge (diff3) needs a common ancestor; that is why every edit carries its base version.
- Line merge cannot understand meaning: two edits to different lines can still produce an inconsistent document; some tools add semantic merge or CRDTs.
- Storage: full snapshots are simple and fast to read; deltas save space but make reads and merges walk chains; periodic snapshots bound the cost.
- Operational transform and CRDTs avoid conflicts by merging at the keystroke level; the cost is more complex data and history.

## Trade-off questions to ask when the candidate is done
- Full snapshots vs deltas: read latency, storage and merge cost?
- Do you hold a lock while diffing a large document? How would you shorten the critical section while staying correct?
- Should a clean merge be authored by the stale editor or marked as a merge? What does history show?
- How would you test the merge: property tests (merge(base, a, a) == a, commutativity of non-overlapping changes)?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
