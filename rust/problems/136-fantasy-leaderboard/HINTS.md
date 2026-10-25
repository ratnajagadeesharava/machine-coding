# Hints — 136 Fantasy Team Leaderboard (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Where does a "point" physically live: in a player, or in a team? Which of the two is touched by one event, and how many of the other are affected by it?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

An event changes one player and many teams, but only by a fixed amount per team (its multiplier for that player). Keep team scores incrementally up to date instead of recomputing them from 11 players. For the board you need something that supports "move this team to its new position" and "give me the first k" without sorting all 50,000 teams each time, and the sort key has the creation sequence in it, so no two teams ever compare equal. Decide what a reader must be guaranteed about a snapshot, then choose the lock that gives exactly that.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `Player { role, points, perTypeCounts, teams that contain it with their multiplier }` and `Team { seq, roster, captain, vice, scoreHalf }`. `recordEvent` computes `delta = count * points`, then for each containing team does `scoreHalf += delta * multiplier` (4, 3 or 2) and repositions the team in an ordered structure keyed by `(-scoreHalf, seq)`. `leaderboard(k)` walks the first k entries; ranks inside a walk come from comparing neighbours (the first of a tie group has rank = its 1-based position). `rankOf` needs "how many teams have a strictly higher score": an ordered structure with subtree sizes, a Fenwick/segment tree over compressed scores, or a scan (acceptable only if you can argue the call count).

A single lock around a mutation (all affected teams moved) plus the same lock for reads gives the "event is all-or-nothing in any snapshot" guarantee. A read-write lock, or immutable snapshots published after each event, keeps readers cheap.

**Rust:** a `BTreeSet<(Reverse<i64>, u64)>` (or `BTreeMap`) works the same way: remove the old key before changing the score, insert the new one after; `RwLock` fits the read/write split, and keep `Arc<Team>`-style sharing out of the hot path.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
