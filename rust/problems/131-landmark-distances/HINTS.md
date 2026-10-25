# Hints — 131 Shortest Distance and Nearest Landmarks (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Four of the five queries ask the same underlying question: "how far is every location from this one?" Which parts of that question change from query to query, and which part is shared? Also ask yourself what makes "just count roads" wrong here.

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Distances with positive road lengths are a classic single-source shortest-path problem, and a priority queue keyed by distance settles locations in increasing order of distance. That ordering is useful for more than the distance itself: you can stop as soon as the answer is known (target reached, enough landmarks collected, radius exceeded). For the equal-length route tie-break, think about running the search from the destination so that each location knows its remaining distance, then walk forward from the start choosing among the hops that stay on a shortest route.

For concurrency, queries only read and updates are rare compared with queries: a read/write lock (or a copy-on-write graph behind an `Arc`) gives every query one consistent graph without blocking other queries.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: locations get dense integer ids; adjacency is a list per location of (neighbour, shortest road, road count) so parallel roads collapse on insert and `removeRoad` can report the count. A tag index maps normalised type to the set of tagged locations. Queries run Dijkstra with a binary heap and `i64` distances (store -1 or `i64::MAX` as "unreached", never a 32-bit type). `nearestLandmarks` pops locations in distance order and collects tagged ones; because the heap order among equal distances is not name order, either collect everything up to the k-th distance and sort, or break heap ties by name. `shortestPath`: Dijkstra from the destination, then from the start repeatedly pick the smallest-named neighbour `v` with `w(u,v) + dist(v) == dist(u)`. Do not cache results across updates unless you can say exactly what each update invalidates.

**Rust:** `RwLock<Graph>` (queries take the read guard), `BinaryHeap<Reverse<(i64, usize)>>`, `Vec<i64>` for distances. Allocating a fresh `Vec` of size V per query is fine for 22,500 locations; a `HashMap<String, i64>` per query is what makes the grid test slow.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
