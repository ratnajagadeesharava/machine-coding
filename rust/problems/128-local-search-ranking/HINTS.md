# Hints — 128 Local Search with Ranking (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Split one search into two questions: "which places are allowed at all?" and "in what order do the allowed ones come?". Write each as a small function of one place and the search request before you worry about speed.

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Treat a place as an immutable value and replace it as a whole on update, so a reader can never see half of it. For the top `k` you do not need to sort all candidates: keep a bounded collection of the best `k` seen so far. Make the scoring a separate piece from the filtering and give the comparator all three tie-break levels. Do the arithmetic in `i64`. If the full scan is too slow, think about how to look only at places whose coordinates could be within the radius.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `Place` record (lower-cased name and category precomputed once) in a map id -> place; `search` = filter (distance, category, rating, price, text) -> score -> bounded min-heap of size `k` ordered by (score desc, distance asc, id asc), drained into a sorted list. Reads and writes are guarded so that an update replaces one whole value; either an `RwLock` or a copy-on-write snapshot of the map works. For speed, bucket places into square grid cells (cell size around the typical radius), visit only the cells that overlap the Manhattan diamond of the query, and remove/re-insert a place in its cell on update. Keep `nearest` correct by expanding rings of cells until `k` places are found and no closer cell remains.

**Rust:** `RwLock<HashMap<String, Place>>` (or `Arc<Place>` values behind it); `BinaryHeap` with a wrapper type implementing `Ord` as "worse is greater" so `peek`/`pop` give the current worst of the best `k`; never `with_capacity(k)`; convert coordinates with `i64::from` before subtracting.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
