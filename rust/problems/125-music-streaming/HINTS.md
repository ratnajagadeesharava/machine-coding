# Hints — 125 Music Streaming and Search

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Which facts change when somebody plays a song, and who owns each of them: the song, the user, or the relation between the two? Write down every query (search, top, recommendations) and the one piece of data it reads.

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Search, top songs and rankings all order songs the same way: put that ordering in one place (a strategy you can reuse) and feed it different candidate sets. For finding candidates quickly, look at the words of each title and artist once, when the song is added, instead of when it is searched; a sorted structure over those words answers "starts with" questions. Recommendations are an aggregation of the user's own counters grouped by artist.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: a catalogue (songs by id, plus a sorted word index: lower-cased word -> song ids, plus songs by normalised artist); per-user state (play counts by song, a most-recent-first list capped at 10, a FIFO queue, playlists by id with a name index); per-song counters. Search: for every token take the union of song ids of all words with that prefix (a range scan of the sorted index), intersect across tokens starting with the smallest set, then sort with the shared comparator and cut at `k`. Plays can be published to listeners (observer) if you want counters, history and a future royalty counter to be separate subscribers, but a direct call is fine for this size.

**Java:** `TreeMap<String, Set<String>>` with `tailMap(token)` and `startsWith` to stop; `LongAdder`/`AtomicLong` for global counters or one lock for the whole write path; `Comparator.comparingLong(...).thenComparing(...)`.

**Rust:** `BTreeMap<String, BTreeSet<String>>` with `range(token..)` and `take_while(starts_with)`; a `Mutex`/`RwLock` around the state, or atomics for the counters and one mutex per user; `sort_by` with `then`/`then_with`.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
