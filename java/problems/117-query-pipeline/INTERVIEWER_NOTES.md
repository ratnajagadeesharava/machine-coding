# Interviewer notes — 117 Search Query Pipeline

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Banned word inside an OR group: the group shrinks, collapses to a plain word at one member, disappears at zero; banning the original word keeps the synonyms as a group.
- Ids that tie are ordered as text (`d10` before `d2`), not numerically.
- Highlight keeps the document's case (`Phone` stays `[Phone]`), wraps every occurrence, and also wraps synonym members.
- A correction is never corrected again; dictionary words one edit from each other are never touched.
- Step positions: `registerAfter(parse, a)` then `registerAfter(parse, b)` gives parse, b, a; `registerBefore(a, g)` goes between b and a.
- A step before `filter` is filtered, a step at the end is not.
- A failing step passes the previous query on unchanged and the later steps see exactly that.
- Every step, including parse, can be removed or disabled; blank input is still rejected before any step runs.
- Concurrent registrations/removals while many calls run: each call sees a consistent order (parse first, rank then highlight last).

## Likely interviewer questions
- Why do the built-in steps and a test-defined step implement the same interface? What would break if the built-ins were hard-coded in the runner?
- Where is the try/catch, and why there and not in each step?
- Why does rank need a parsed form of the query, and who produces it? Do filter and rank parse the OR group twice?
- How does `process` stay consistent while someone registers a step mid-call?
- What does the trace say for a step that returned the same text it received?
- Why are rank and highlight not in the same list as the query-phase steps here, and what would it take to make them regular steps?

## Design patterns and data structures to listen for
- Pipeline / Chain of Responsibility: a uniform step interface, an ordered list, one runner; open for extension (new step) and closed for modification (built-ins untouched).
- Strategy: each step is an interchangeable behaviour; spell/synonym/filter behaviours are configured data plus a strategy.
- Immutable snapshot (copy-on-write) of the step list for readers, replaced under a lock by editors; alternatively `CopyOnWriteArrayList`.
- A small value type for a term (a word or an OR group), shared by filter, rank and highlight.
- Possibly a decorator around steps for failure handling / timing (V2) and a builder for a pipeline (V5).
- Spell: candidate lookup by length buckets or deletion neighbourhoods to avoid scanning a large dictionary; a plain scan is fine here but should be named as such.

## Invariants
- Built-in order is parse, spell, synonyms, filter, rank, highlight until an editor changes it.
- A call runs against one snapshot of the step list; trace entries appear in execution order.
- A failed step never changes the query; a disabled or removed step never appears in the trace.
- Names are unique across built-in and registered steps; built-in names stay reserved.
- Registration never modifies a built-in step.

## Common mistakes
- Hard-coding built-in steps in the runner and bolting registered steps on separately, so ordering and failure handling differ.
- A try/catch only around registered steps, or catching Exception in the steps themselves.
- Recording a step in the trace whenever it ran, not only when it changed something.
- Trimming the query in the runner instead of parse (so a step before parse gets cleaned input).
- Counting duplicate query terms twice, or counting an OR group's members as separate terms.
- Treating a swap of two letters as one edit, or correcting a word that is already a dictionary word.
- Correcting when two dictionary words are candidates (picking the first), or correcting to a word at distance 2.
- Mutating the step list while `process` iterates (ConcurrentModificationException or a half-applied registration).
- Numeric ordering of document ids; case-sensitive matching or lower-casing the document in the output.
- Leaving empty parentheses or `( )` in the query when every member of a group is banned.

## Design and scaling discussion
- A real engine would parse once into a query tree and pass the tree (not text) between steps; the text form here is the observable contract.
- Spell correction at scale: BK-trees, symmetric-delete indexes, or a finite-state automaton; dictionaries of millions of words.
- Per-step metrics and timeouts (V2/V4) want a wrapper around steps, so the runner stays small.
- Ranking beyond counting terms (TF-IDF, field boosts) would replace the rank stage without touching the others.

## Trade-off questions to ask when the candidate is done
- Immutable list swap vs a read-write lock around a mutable list: what does each cost on the hot path?
- Should a failing built-in step abort the request in production? When would you rather fail fast?
- Is `name:failed` a good trace format? What would you use if names could contain a colon?
- How would you test a step that is slow or flaky without sleeping?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
