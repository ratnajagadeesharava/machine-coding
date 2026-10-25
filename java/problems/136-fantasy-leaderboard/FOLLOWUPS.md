# Follow-ups — 136 Fantasy Team Leaderboard

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Contests with entry fees
A contest is a named leaderboard that a subset of teams joins by paying an entry fee, with its own prize pool (for example the top 10% share 80% of the pool, split by rank, ties splitting the prizes of the places they occupy). One team may be in several contests. How do contest boards relate to the global board, and what is paid out when the match is finalised and ties straddle a prize boundary? Keep money in integers.

### V3 — Tie-break by who scored first
Equal scores are broken by the time at which the team last reached that score (earlier wins) instead of creation order, using an injectable clock. A correction can lower a team and then another event raises it again. What do you store per team, and does `rankOf` still mean "1 + strictly higher"?

### V4 — Lineup locking and substitutions
Teams can be edited until the match starts; after the lock, a team may make up to two substitutions, each costing 4 points (8 half points), and the substituted player's earlier points no longer count for that team. What is the history of a team now? How do you keep scores exact when a player's points before and after a substitution matter, and what happens to events recorded concurrently with a substitution?

### V5 — Live updates for subscribers
Fans watch their team: a subscriber gets a notification whenever its team's rank or score changes, or whenever the top 10 changes. How do you deliver without calling subscribers under your lock, without flooding on every event, and without losing the final state for a slow subscriber? What do you do when 50,000 subscribers all care about the same event?
