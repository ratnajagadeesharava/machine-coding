# Interviewer notes — 136 Fantasy Team Leaderboard

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- A correction that brings two teams back to the same score: the tie is restored and creation order decides again.
- A three-way tie followed by a lower team: ranks 1,1,1,4; the tied group cut by `k` keeps rank 1.
- Counts near the int limit (2e9 x 25 points): needs 64-bit points and 64-bit half-points; the `count * points` product must be computed in 64 bits.
- 16 threads creating a team with the same id: exactly one wins and the others fail cleanly.
- Snapshots while teams are being created and events recorded concurrently: never duplicated, never unsorted, tied teams share the "1:" rank.
- `rankOf` agrees with the `leaderboard` line of the same team.
- Record/correct interleavings on one player: the correction allowance never goes negative and no update is lost.

## Likely interviewer questions
- Where is a player's point total stored, and why is a team's score not recomputed from scratch on every read?
- What is the work for one event on a player in 5,000 teams? Can you avoid touching all of them on write, and what does that cost on read?
- Why half points instead of doubles? What does x1.5 do to the invariants if you wrote it as `points * 1.5`?
- How do you make "an event is visible in every team or in none" true for readers? What is the lock scope?
- What does `rankOf` cost in your design, and what if it were called 1,000 times a second?
- A team created after events happened: how does your design get its first score right?

## Invariants
- A team's score always equals 4 x captain + 3 x vice + 2 x others using the players' current points.
- A player's per-type counts never go negative; the player's points equal the sum of count x points per type.
- The ordered view contains every team exactly once, ordered by (score desc, creation sequence asc).
- Failed operations change nothing (team id, counters, positions).

## Common mistakes
- Updating the score field of a team while it is still inside a sorted collection (the order breaks silently), or reinserting without removing the old entry (duplicates in the leaderboard).
- Using `double` for x1.5, or integer-dividing the vice-captain's points by 2 and losing the half.
- Computing the team score only from events seen after creation (a team created late starts at 0).
- Forgetting that the tie-break is creation order, so two teams with equal scores have a stable, deterministic order; sorting by team id instead.
- Rank by position in the list (1,2,3,4) instead of competition ranking, or dense ranking (1,2,2,3).
- Computing rank only among the returned `k` entries rather than among all teams.
- Applying an event to teams one at a time under per-team locks so a reader sees half of the update.
- Rejecting a duplicate team id after having registered the team's players in per-player team lists (leaving ghost entries).
- Clearing the correction counter on a failed correction, or checking it against the player's total points instead of the per-type count.
- `Collections.sort` of all 50,000 teams on every `leaderboard(10)`.

## Over-engineering traps
- An event bus with subscribers and a thread pool before the synchronous core works.
- A class per event type / role with a strategy interface for a lookup table of eight constants.
- Persisting events in a database, or an append-only log with replay, as part of the core.
- Lock-free structures with hand-rolled snapshot isolation when one lock meets the requirement.

## Design and scaling discussion
- Write cost vs read cost: pushing each event into every containing team (O(teams-of-player x log n)) gives O(k) leaderboard reads. The alternative is to store only player points and compute a team lazily or recompute the board on demand (O(n) per read, but O(1) per event); a hybrid recomputes a dirty subset. Ask which workload (many events, few reads, or the reverse) each favours.
- Ordered structure: balanced tree / skip list keyed by `(-score, seq)`; an order-statistic tree or a Fenwick tree over compressed scores gives `rankOf` in O(log n). A sorted array is O(n) per move.
- Snapshot options: one big lock (simple, event latency grows with the number of containing teams), read-write lock, or copy-on-write published views (readers never block, cost shifts to writers). Sharding by team loses the global order and needs a merge of per-shard top-k (which is fine for `leaderboard(k)`, exact for ranks only with counts).
- Hot players: a star player in 40% of teams makes every event touch 20,000 teams. Batching events per tick, or storing only the player total and applying the multiplier at read time for the top candidates, are the standard answers.
- Real systems use Redis sorted sets for exactly this (`ZADD`, `ZREVRANGE`, `ZREVRANK`) with a score encoding that folds in the tie-break; discuss the precision limits of that encoding and why half points help.
- Observer: this is the producer side of the pattern. Scoring feeds team scores (update the interested teams); a live leaderboard push to subscribers is the next observer layer (see V5).

## Trade-off questions to ask when the candidate is done
- Eager per-team updates vs lazy derivation: what would make you switch?
- Would you keep per-event history to allow undo of a specific event, rather than "N of type T"? What does that cost?
- How would you shard 5 million teams across machines and still return a global top 10 and an exact `rankOf`?
- How do you test the snapshot guarantee deterministically instead of by hoping a race shows up?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.
