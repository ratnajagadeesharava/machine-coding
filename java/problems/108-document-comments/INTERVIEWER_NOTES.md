# Interviewer notes — 108 Document Comment Service

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Ids are one sequence for the whole service: add to d1, d2, d1 gives "1", "2", "3".
- Rejected adds (blank reviewer, blank text, unknown document) racing with good adds must not leave holes in the id sequence.
- Readers running during writers must always see a prefix of the final history: no gaps, no half-written entry.
- A list returned earlier must not change when new comments arrive, and mutating it must not alter the service.
- Text containing `: `, newlines, leading/trailing spaces or non-ASCII is stored verbatim; only all-blank text is rejected.
- 50,000 comments on one document keep their order and every call stays cheap.
- Unknown document read after a failed add still fails: a failed add must not create the document.
- Blank arguments beat an unknown document in the error choice.

## Likely interviewer questions
- Where is the id counter, and what makes "take an id" and "append to the document" one step?
- Why is a per-document lock (or a single lock) enough, and what would a global lock cost?
- What does `comments` copy, and how long does it hold the lock while copying?
- Why is the document set fixed, and what would adding documents at runtime change in your locking?
- What would you return if the document has millions of comments?

## Invariants
- Ids are unique, start at 1, are never reused and are assigned only to accepted comments.
- Within a document, list order equals id order.
- A reader sees the document as of one moment (a prefix of the history).
- A rejected call has no side effect.

## Common mistakes
- Taking the id before validating, so a rejected call burns an id.
- Taking the id from an atomic counter and appending under a different lock: ids and list order disagree under contention.
- Returning the internal list (or a live view), or an unmodifiable wrapper that still changes when comments are added.
- Iterating a plain `ArrayList` while another thread appends, or copying it without holding the lock.
- Checking for the document and adding to it in two separate steps over a mutable map.
- Trimming the text, or formatting as `reviewer:text`, or lower-casing ids.
- Using `isEmpty()` / `trim().isEmpty()` only on the text and forgetting the reviewer or the document id.
- Letting the caller's document set leak into the service (mutating the set after `create`).

## Over-engineering traps
- A `Comment`/`Reviewer`/`Document` hierarchy with visitors and an event bus before the core works.
- A repository interface and a factory for a single in-memory map.
- Persistence, pagination or notifications that nobody asked for.

## Design and scaling discussion
- Read-mostly per document: copy-on-write snapshots or a read-write lock give cheap reads; the cost is a copy per write or a write lock during copying.
- The global id sequence is the only shared hot spot; per-document locks plus one atomic counter are fine if the id is taken while the document lock is held.
- If ids only had to be unique (not gap-free in order), a per-document counter or a UUID removes the cross-document coordination.
- For huge histories, return pages or a cursor instead of a full copy.
- Persistence: an append-only log per document, replayed on start, maps naturally onto this model.

## Trade-off questions to ask when the candidate is done
- Single lock vs lock per document vs immutable snapshots: what is the throughput and the complexity of each?
- How would you test the ordering guarantee without sleeping or relying on timing?
- If replies (V2) arrive, is the id still the key, and what does `comments` return?
- What happens if two reviewers comment at the same instant: is there a "correct" order, or only a consistent one?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
