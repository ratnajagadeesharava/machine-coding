# Interviewer notes — 128 Local Search with Ranking

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Randomised comparison with a brute-force oracle (3,000 places, 400 searches with every filter combination): catches any single misread rule.
- Coordinates at `i32::MIN` / `i32::MAX`: `i32` arithmetic overflows (`(a - b).abs()` panics in debug builds, wraps in release) and `3 * distance` overflows `i32` far sooner.
- `k = i32::MAX`: `Vec::with_capacity(k)` or an eagerly sized heap blows up.
- Bonuses are computed from the **name**, even when the place matched through its category (`"Coffee Cart"` in category `coffee`, query `coffee` gets the prefix bonus; `"Beans"` in category `COFFEE` gets none).
- Equals bonus stacks: an exactly equal name gets `+50`, not `+25`.
- Blank query is the same as no query (no bonus either); a partial category (`india` vs `indian`) does not match through the category.
- Negative scores must still order correctly; tie-breaks apply to equal *scores* only, not equal ratings.
- Category case-insensitivity in the filter, in the query-equals-category rule and in `categories()` (merged, lower-cased, zero-count categories vanish after update/remove).
- A failed update must not remove or partially change the old place (validate before mutating, and do not do remove-then-add).
- Concurrent update flipping a place between two complete states while searches ask for mixes of the two (name of one, position of the other).
- 100,000 places x 10,000 searches: a full scan with allocation per candidate is slow; a scan that precomputes lower-cased name/category and keeps a bounded heap passes.

## Likely interviewer questions
- Why is the ordering a total order? What would break if two places could tie on all three keys?
- Where does the lower-casing happen, once per place or once per search per place? What does that cost at 50k places?
- How much memory does a search use? What is its complexity in places, candidates and `k`?
- How do you make an update atomic for readers without blocking them for long?
- When would you stop scanning everything and add an index? What does the index have to do on update?

## Invariants
- The id set, the per-category counts and any spatial index always describe the same places.
- A place is replaced as a unit; no reader sees a mixture of old and new attributes.
- Ranking is a pure function of (place, request); it does not depend on insertion order or on other candidates.
- Failed operations (duplicate add, unknown update/remove, invalid values) change nothing.

## Common mistakes
- `i32` arithmetic for distance or score; `abs()` on an `i32` difference.
- Sorting every candidate and taking `k`, or `BinaryHeap::with_capacity(k)`.
- Calling `to_lowercase()` on every name of every candidate on every search instead of once per place.
- Applying the name bonuses only when the text matched the name, or not at all when matched by category.
- Giving `startsWith` and `equals` as alternatives (+25) instead of stacking (+50).
- Update implemented as remove + add: a concurrent search can see the place missing; a failed update loses the place.
- Tie-break by insertion order, or by rating, instead of distance then id.
- Holding one global lock during the entire scan with writes also blocking readers forever (acceptable here, but should be a stated choice).
- Mixing up `radius` (inclusive) and `k`; returning `k` results from `nearest` when the filters were supposed to be ignored.

## Over-engineering traps
- A query parser, tokenizer, stemming or an inverted text index (the spec is a plain substring).
- A strategy hierarchy for ranking with one implementation, a plugin system for filters, or a specification pattern before the five filters work.
- Event sourcing / persistence of the catalogue.
- Building a quadtree/geohash first and only then discovering the correctness rules.

## Design and scaling discussion
- Patterns worth hearing: a scoring strategy (a small interface or a function) so personalised boosts and sponsored slots are additions; a filter pipeline or predicate composition; a bounded min-heap for top-k.
- Spatial index options: uniform grid buckets keyed by cell (simple, fine for evenly spread data, degrades with clusters), quadtree / k-d tree (adaptive), geohash prefix buckets (string keys, good for sharding). With a Manhattan radius, only cells intersecting the diamond need visiting; ring expansion gives `nearest`.
- Cell size trade-off: too small and a large radius touches many empty cells; too large and each cell holds too many places. Mention a fallback to a full scan when the radius covers most of the map.
- Concurrency options: an `RwLock`; copy-on-write immutable snapshot (cheap reads, expensive writes at 50k places unless structure is persistent); striped locks per cell with care for a place that moves between cells (lock ordering). The invariant to defend: a reader sees a place in exactly one cell, with one set of attributes.
- Sharding by region; the search fans out to the shards whose bounding boxes intersect the radius and merges per-shard top-k lists (the score is a pure function, so merge is just the same comparator).
- Text: the substring rule is O(name length) per candidate; mention trigram index or prefix tries only as the next step when the text filter dominates.

## Trade-off questions to ask when the candidate is done
- Why `i64` for the score even though ratings and bonuses are small?
- What is the worst case for your index, and what does a search cost there?
- A moving place: in which order do you update the id map and the index so no reader loses or duplicates it?
- If a search takes the lock for 20 ms, what happens to writers; what would you change first?
- How would you test the faster version against the simple one (hint: the hidden tests do it).

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test -p p128_local_search_ranking --features hidden`. They only use the same public contract.
