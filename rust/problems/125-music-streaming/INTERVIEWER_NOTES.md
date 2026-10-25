# Interviewer notes — 125 Music Streaming and Search

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- 100,000 songs and hundreds of queries: a per-query scan that lower-cases and splits every title is too slow; words must be prepared at `addSong` time. Note that a query with a very common token (`"song"`) matches everything, so intersecting from the rarest token matters.
- A token that matches the same word as another token (`rock rock`, `r r r`) and a token matched by the artist only. Duplicate hits must not appear twice in the result.
- Title tie-break is case-insensitive (`apple` before `Banana`), song id is the last key. A mistake here shows only when titles differ in case or are equal.
- Search re-ranks live: the counters are read at query time, not frozen when the song was added.
- History: 11 distinct songs with every song played twice; replaying a dropped song re-adds it; replaying an in-history song never drops anything.
- Recommendations rank by the user's artist count first, so a hugely popular song of a weakly played artist loses against an unplayed song of a strongly played artist. Songs played through `playNext` count; queued-but-not-popped do not. Artist `" queen "` is the same artist as `Queen`.
- Two users with the same playlist name, and a user reading another user's playlist id: unknown playlist, not a leak.
- Mixed concurrent traffic: exact totals (plays + pops), an exact playlist, exactly one winner for racing duplicates.
- A rejected duplicate `addToPlaylist` and a failed `play` (unknown song after a valid user) change nothing.

## Likely interviewer questions
- Where does each counter live (song, user, user-song pair)? What does `play` touch, and how do you keep those updates atomic together?
- How is the title ordering defined and where is it implemented? Is the same comparator used by search and top songs?
- What is the complexity of `search`, `topSongs` and `recommendations`? What would you change if there were 50 million songs or 10,000 plays per second?
- Why is a playlist unknown to other users rather than "forbidden"?
- Is the history a list, a linked structure or a map plus order? What does move-to-front cost?
- What does `playNext` do when the queued song was meanwhile removed from the catalogue (it cannot be in this problem; what would you do if songs could be deleted)?

## Invariants
- Global count of a song equals the sum of all users' counts of that song.
- A user's history has no duplicates, is at most 10 long and only contains songs the user played.
- A playlist has no duplicate songs and belongs to exactly one user; (user, name) is unique among live playlists.
- Every queued song is played exactly once, in FIFO order.
- Search results are exactly the matching songs in ranking order.

## Common mistakes
- Treating the query as one string (`title.contains(query)` or "starts with the whole query").
- Substring instead of word-prefix matching; matching only the title; requiring all tokens in the title or all in the artist.
- Ranking by title case-sensitively (uppercase letters sort first), forgetting the id as the last key, or sorting by play count ascending.
- History kept as a list with duplicates, or truncated before removing the existing entry, giving 9 distinct songs.
- Counting a song when it is enqueued, or popping without counting.
- Recommendations: including songs already played, summing global plays per artist instead of the user's, ranking by global popularity first, treating `Queen` and `queen ` as different artists.
- Check-then-act races (`contains` then `put`) for `addSong`, `registerUser`, `createPlaylist`; a non-atomic `count++` on a hot song; two locks taken in different order.
- Returning the internal playlist/history list to the caller.

## Over-engineering traps
- A full text-search engine (stemming, fuzzy matching, scoring) before prefix matching works.
- An event bus with threads for plays before the counters are right.
- A class per ranking rule and a factory for them when there are exactly two orderings.
- Persisting to a database; modelling albums, genres and audio formats that no requirement mentions.

## Design and scaling discussion
- Patterns an interviewer hopes to hear, when justified: **observer** for play events (history, counters, royalty and trending subscribers), **strategy** for ranking (search/top vs recommendations share a comparator abstraction), a **sorted prefix index** (trie or ordered map) for search, **LRU-like move-to-front** for the history, a repository per aggregate (songs, users, playlists).
- Search index: a trie with song-id postings, an ordered map over words, or an inverted index with prefix expansion. Trade-off: memory vs query latency, and updating rankings (counters change constantly, so rank at query time or keep a top-N per prefix refreshed asynchronously).
- Counters: a single lock is fine at this scale; `LongAdder`/sharded counters or per-user locks scale the write path. Global top-N under heavy writes: a periodically rebuilt ranking or a bucketed structure instead of sorting all songs per call.
- Recommendations: per-user artist counts maintained incrementally on play (O(1)) vs computed on demand (O(distinct songs played)); later collaborative filtering is a different feature.
- Injected sequence instead of a wall clock keeps everything deterministic; a real service would add timestamps, sessions and a skip/listen threshold for what counts as a play.

## Trade-off questions to ask when the candidate is done
- One global lock vs lock per user vs lock per song: what invariants cross those boundaries (global count = sum of user counts)?
- Would you keep the word index in the same structure as the songs? What happens to it when songs can be edited or deleted?
- How would you test "search is fast" without a flaky timing assertion?
- What would you cache, and how would a cached top-10 stay correct after a play?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test -p p125_music_streaming --features hidden`. They only use the same public contract.
