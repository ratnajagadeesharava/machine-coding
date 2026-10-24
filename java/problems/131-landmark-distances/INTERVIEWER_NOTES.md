# Interviewer notes — 131 Shortest Distance and Nearest Landmarks

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- 150 x 150 grid with unit roads: a huge number of equal routes; the name-sequence tie-break must still be exact and fast (go right along the first row, then down, with zero-padded names).
- A lexicographically smaller first hop that leads to a longer route must lose (length first, names second).
- A 1,000-way tie on landmark distance: ordering by name with plain string comparison (upper case before lower case, `shop10` before `shop2`).
- Removing and re-adding roads on a grid: wall off a column, open one gate, check detours; no stale answers.
- Two components joined later by a new road: `-1` / empty must turn into real answers immediately.
- Totals above the `int` range inside `nearestLandmarks` strings and `withinRadius`.
- Queries interleaved with an adder that only ever shortens a route (monotone observation) and with tag/road flip-flopping.
- Many threads growing the map by adding locations and roads while querying it.

## Likely interviewer questions
- Why is BFS wrong here? What if all roads had length 1, and what changes if some had length 0?
- What is the complexity of one query on V locations and E roads? How early can you stop for each of the four query kinds?
- How do you produce the lexicographically smallest route among equal-length ones, and why is "relax with a name tie-break in the forward search" error-prone?
- How do parallel roads show up in your structure, and what does `removeRoad` have to do about them?
- How do you guarantee that a query never sees half of an update (and what are the two ways to implement that)?
- What do you do about `Long.MAX_VALUE + w` overflow when a distance is "infinite"?

## Invariants
- Every stored road length is positive; each unordered pair has one effective (smallest) length plus a count of roads.
- The graph is symmetric: whatever is stored for `(a, b)` is also stored for `(b, a)`.
- The tag index and the per-location tags agree (a tag exists in both or in neither).
- A failed call (unknown location, invalid argument) leaves the graph and tag index untouched, including the half-way point of `addRoad` (do not insert one direction and then fail).

## Common mistakes
- BFS or Floyd-Warshall (22,500^3 operations) or recomputing all-pairs on each update.
- `int` for distances or totals; `Integer.MAX_VALUE` as infinity; `dist + w` overflow on an infinite entry.
- Cutting the search at the first time the target is *pushed* instead of when it is *popped*.
- Breaking path ties inside the forward Dijkstra by "first predecessor found" or "smallest predecessor name": this does not give the lexicographically smallest whole sequence. (Smallest predecessor is the smallest *last* hop; the requirement is the smallest *first* differing hop.)
- Treating a tie in `nearestLandmarks` as arbitrary heap order, or cutting off after exactly k pops while a later landmark at the same distance has a smaller name.
- Forgetting that `removeRoad(a, b)` must also remove roads added as `(b, a)`, and that parallel roads all go.
- Normalising the type in `tag` but not in `nearest` (or the reverse), or lower-casing location names.
- A cache that is not cleared by `removeRoad` / `untagLandmark`, or that is cleared before the update is applied (readers refill it with the old graph).
- A `synchronized` method for everything: correct but serialises all queries (each is milliseconds on the grid).
- Iterating the adjacency while another thread inserts (ConcurrentModificationException), or locking each location separately and reading a mixed graph.

## Over-engineering traps
- A generic graph library with interfaces for `Edge`, `Vertex`, `Algorithm`, strategy objects per query before one search works.
- Persisting the graph, or an event bus for map changes.
- Precomputing all-pairs distances or landmark tables on every update (a legitimate follow-up discussion, not a core need).

## Design and scaling discussion
- Single-source shortest paths with non-negative weights: Dijkstra with a binary heap, O((V + E) log V); stopping on pop of the target; for nearest-k, multi-target early exit when k landmarks plus their ties are settled; for radius, stop when the popped distance exceeds the radius. The "multi-source" idea: all landmarks of a type as sources gives nearest landmark of every location in one pass (useful if the question is reversed or cached per type).
- Reverse search to build the lexicographically smallest route: after the search from the destination, every forward step only has to choose among neighbours that keep `w + dist(v) == dist(u)`; greedy smallest name is then globally correct because all feasible continuations have the same remaining length.
- Concurrency: read/write lock vs copy-on-write snapshot. Updates here rebuild or mutate a graph of 45,000 roads; a lock is simpler, a persistent/immutable structure gives lock-free reads at a higher update cost.
- Caching (V5): invalidate everything on any update (simple, correct) versus per-source results with a version counter checked at read time. Discuss the race where a query started before an update finishes after it and stores a stale result (store with the version it read).
- A* (V2) needs an admissible heuristic: straight-line distance is only valid if no road is shorter than it; contraction hierarchies / landmark-based bounds (ALT) for a real road network.
- One-way roads (V6): the reverse search for the route needs a reversed adjacency; "within radius" and "nearest" become direction-dependent.

## Trade-off questions to ask when the candidate is done
- Adjacency in a map keyed by name versus dense integer ids: cost on the grid, and where the name-to-id mapping lives.
- Would you keep one global lock or shard by region? What does a query that crosses shards see?
- If the same few sources are queried constantly, what would you cache and how would the cache know when to die?
- How would you test the tie-break rule without an oracle you wrote with the same idea? (Brute-force enumeration on small random graphs.)

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
