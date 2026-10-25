# Interviewer notes — 127 Browser History Manager

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Visiting a url equal to a page *ahead* (not the current page) must still truncate and append; only equality with the *current* page suppresses a new entry.
- Visiting the current url keeps the pages ahead (a reload), unlike every other visit.
- `a, b, a` is three pages; a naive "skip if the url is anywhere in the trail" is wrong.
- Entries with the same timestamp must keep recording order; sorting by time alone is unstable.
- `mostVisited` counts only retained entries: eviction and `deleteFromHistory` lower counts, and the url tie-break only applies when the latest *times* are equal (not merely when the entries are adjacent).
- Capacity one, clear-then-visit, delete-then-refill all keep the capacity invariant.
- `back(Integer.MAX_VALUE)` / `forward(i32::MAX)` must not overflow (`pos - steps` in `int`, or `usize` underflow in Rust).
- 300,000 visits with capacity 50,000: an `ArrayList.remove(0)` eviction makes this quadratic.
- Tabs closed while other threads browse: ids stay unique, closed ids are unknown, history keeps their entries.

## Likely interviewer questions
- Why does the trail live in the tab and the log in the manager? Who owns the invariant "capacity is never exceeded"?
- Why is `visit` of the same url special, and what does Chrome actually do? (It treats it like a reload.)
- How do you make eviction O(1)? What does `deleteFromHistory` cost, and could it be cheaper?
- What is the complexity of `search` and `mostVisited`, and when would you maintain an index instead?
- Your lock: one global lock, or one per tab plus one for the log? What is the lock order, and does the extra concurrency pay?
- What should happen if the clock goes backwards? (Recording order versus time order.)

## Invariants
- A tab always has at least one page (`about:blank` initially) and `0 <= position < pages`.
- The global log never exceeds the capacity and is ordered by recording order.
- A visit updates the trail and the log as one atomic step; a failed call changes nothing.
- Tab ids are strictly increasing and never reused.

## Common mistakes
- Replacing the whole trail with a linked list of nodes and then having no cheap way to clamp `steps`.
- Recording back/forward in the global history.
- Recording `about:blank` when a tab opens.
- Letting `visit(sameUrl)` truncate the forward pages, or letting it be a no-op for the global log.
- Deduplicating `search` results by lower-cased url (the spec keeps differently-cased urls distinct).
- Sorting `mostVisited` by count only, or by the latest recording position instead of the latest time.
- Throwing when steps exceed the available pages instead of clamping.
- Reusing tab ids from `size() + 1` (collides after a close).
- Evicting by time instead of by recording order; evicting before validating the url.

## Over-engineering traps
- A command/undo stack or memento objects for navigation before reopen-closed-tab is asked.
- An LRU cache class for the history capacity (it is a FIFO of visits, not an access-ordered cache), or an event bus for "page visited".
- A trie or inverted index for `search` before measuring a need.

## Design and scaling discussion
- Patterns an interviewer hopes to hear, and only if they fit: a memento (tab state snapshots) or command objects when V2 (reopen closed tab) arrives; a deque used as a bounded FIFO for the log; an LRU-style structure only in the V6 variant where recency of use matters; a strategy for ranking (count versus frecency) in V6; a policy object for "record or not" in V4 (incognito).
- Per-url counters (map url to count and latest time) turn `mostVisited` into a sort over distinct urls; a heap or ordered set gives top-k without a full sort at the price of keeping it consistent under delete and eviction.
- A real browser stores history in a database with an index on url and time; the in-memory log with a capacity is the cache of the recent part.
- Locking: the whole manager is tiny and every operation is a few list operations; one lock wins. Sharding by tab would need a lock around the shared log anyway.

## Trade-off questions to ask when the candidate is done
- Would you store the trail as a list and an index, or as two stacks (back and forward)? What does each make cheap, and how do you implement "go back 5"?
- Single log with filtering on delete versus a per-url index of entries: which operations get faster, which get more fragile?
- Return copies or views from `history()`? What can a caller do with a live view?
- Where would a persistence layer hook in without leaking into the tab logic?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
