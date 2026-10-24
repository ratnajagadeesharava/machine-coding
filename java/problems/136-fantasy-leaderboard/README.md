# 136 · Fantasy Team Leaderboard (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 60 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

In a fantasy cricket game, players of a real match earn points as the match unfolds (runs, wickets, catches ...). Every
fan builds a **team** of 11 real players and picks a captain and a vice-captain who earn extra. Your system keeps every
team's score up to date after each scoring event and answers "who is winning?": a live leaderboard with ranks, where
many teams can share the same players and scores arrive from many threads at once.

## Problem

Build the scoring and ranking core for one match:

1. Register **players**, each with a role.
2. Fans create **teams** from registered players under roster rules.
3. **Scoring events** arrive for players (a four, a wicket, a catch ...). Each adds points to the player; every team that
   contains the player changes score immediately.
4. The **leaderboard** lists the best teams with their ranks; a team's rank can also be asked directly.
5. A scoring mistake can be **corrected** (an event reversed).

## Requirements

### Players and roles

- A player has an id, a name and exactly one role: `BATTER`, `BOWLER`, `ALL_ROUNDER` or `KEEPER` (exact, upper case).
- Ids and names must be non-blank; a player id can be registered only once.

### Teams

A team has an id, an owner, 11 players, a captain and a vice-captain. It is valid only if **all** of these hold:

- the team id and owner are non-blank, and no team with that id exists yet;
- exactly 11 player ids are given, all different, all registered;
- the captain and the vice-captain are both among the 11, and they are different players;
- at least one of the 11 is a `KEEPER`;
- at most six of the 11 are `BATTER`s (no other limit on any role).

The same player may be in any number of teams, as captain in one and vice-captain or ordinary in another. An invalid
request creates nothing and does not use up the team id. Teams are never removed or edited; their **creation order**
is the order in which the successful `createTeam` calls happened.

### Scoring events

| Event type | Points per occurrence |
|---|---:|
| `RUN` | 1 |
| `FOUR_BONUS` | 1 |
| `SIX_BONUS` | 2 |
| `WICKET` | 25 |
| `CATCH` | 8 |
| `STUMPING` | 12 |
| `MAIDEN` | 12 |
| `DUCK` | -2 |

- `recordEvent(player, type, count)` adds `count x points(type)` to that player's **match points**. `count` must be at
  least 1; the player and the event type must exist (type names are exact and case-sensitive). A player's match points
  may be negative.
- A player's match points are independent of teams. A team created after some events already happened scores them too:
  a team's score always reflects its players' current match points.
- `correctEvent(player, type, count)` reverses `count` earlier occurrences of that event type for that player
  (subtracts `count x points(type)`). It is an error to reverse more occurrences of a type than are currently recorded
  for that player (recorded minus already corrected, per player and per type). A failed correction changes nothing.
- Match points can grow large (counts up to 2,000,000,000 per call are valid); nothing may overflow or lose precision.

### Team score

A team's score is the sum over its 11 players of their match points, where the **captain's** points count **x2**, the
**vice-captain's** points count **x1.5** and everyone else's count x1.

To keep this exact, scores are expressed in **half points** (an integer): `scoreHalfPoints = 4 x captainPoints +
3 x viceCaptainPoints + 2 x (sum of the other nine players' points)`. A team whose captain has 50 points and nobody
else scored has `scoreHalfPoints = 200`, i.e. 100 points. Never use floating point. Scores can be negative.

An event is visible in **every** team that contains the player as soon as `recordEvent` (or `correctEvent`) returns, and
a reader never sees an event applied to some teams but not yet to others.

### Leaderboard and rank

- `leaderboard(k)` returns at most `k` strings `"rank:teamId:scoreHalfPoints"` (for example `"2:dream11:224"`), best
  first: score descending, and for equal scores the team created **earlier** first. `k = 0` gives an empty list; if there
  are fewer than `k` teams, all of them are returned. A negative `k` is an error.
- **Competition ranking**: a team's rank is `1 + (number of teams with a strictly higher score)`. Equal scores share a
  rank and the next rank is skipped: scores 200, 100, 100, 0 have ranks 1, 2, 2, 4. The rank is computed over all teams,
  not just the `k` returned, so a tied group cut by `k` still shows its shared rank.
- `rankOf(teamId)` returns that rank; `teamScore(teamId)` the half-point score; `playerPoints(playerId)` the whole-point
  match points. Unknown ids are errors.
- Every call to `leaderboard` is a consistent snapshot: correctly sorted, each team at most once, ranks consistent
  with the scores printed on the same lines, even while other threads record events and create teams. After all
  threads finish, the scores equal what the same operations would give if run one after another.

## Constraints

- The API is called from many threads at once and must be thread-safe.
- Scale the tests exercise: 50,000 teams, 200,000 scoring events and 1,000 `leaderboard(10)` calls interleaved with
  them must complete well inside a minute (the bound is generous).
- Everything in memory; no wall-clock time, randomness or I/O is involved in the behaviour.

## Examples

### 1. Captain, vice-captain and half points

```java
f.addPlayer("a1", "Asha", "KEEPER");      // ... a2..a11 registered too (4 BATTER, 3 BOWLER, 3 ALL_ROUNDER, ...)
f.createTeam("T", "ravi", List.of("a1", "a2", /* ... */ "a11"), "a1", "a2");   // captain a1, vice-captain a2

f.recordEvent("a1", "RUN", 50);       f.teamScore("T");   // 200   (50 points x2 = 100 points)
f.recordEvent("a2", "CATCH", 1);      f.teamScore("T");   // 224   (+ 8 x1.5 = 12 points = 24 half points)
f.recordEvent("a3", "WICKET", 1);     f.teamScore("T");   // 274   (+ 25 points = 50 half points, a3 is neither)
f.leaderboard(1);                     // ["1:T:274"]
```

### 2. Competition ranking and the creation-order tie-break

```java
// teams created in this order: low, mid1, top, mid2 (no overlap in players); then
// top's player scores 100 RUN, mid1's and mid2's players score 50 RUN each (all as ordinary players: x2)
f.leaderboard(4);   // ["1:top:200", "2:mid1:100", "2:mid2:100", "4:low:0"]
f.rankOf("mid2");   // 2
f.rankOf("low");    // 4
f.leaderboard(2);   // ["1:top:200", "2:mid1:100"]   (mid1 was created before mid2)
```

### 3. Correcting a scoring mistake

```java
// team T holds a1 (captain) ... ; no event yet
f.recordEvent("a1", "WICKET", 2);     // a1 now has 50 points, T = 200
f.correctEvent("a1", "WICKET", 1);    // a1 has 25 points, T = 100
f.correctEvent("a1", "RUN", 1);       // IllegalArgumentException: no RUN recorded for a1
f.correctEvent("a1", "WICKET", 2);    // IllegalArgumentException: only 1 WICKET is left; nothing changes
f.playerPoints("a1");                 // 25
```

### 4. A team created after the events

```java
f.recordEvent("a1", "RUN", 10);       // before any team exists: a1 captain-to-be
f.recordEvent("a5", "RUN", 10);
f.createTeam("late", "o", squadOfA, "a1", "a2");
f.teamScore("late");                  // 60  (a1: 10 x2 = 40 half points, a5: 10 x1 = 20 half points)
```

## Edge cases

- Ten or twelve players, a repeated player, or an unregistered player: the team is not created.
- Captain missing from the 11, vice-captain missing, or the same player chosen for both: not created.
- No `KEEPER`, or seven `BATTER`s: not created; exactly six `BATTER`s and one `KEEPER` is fine.
- A rejected team does not consume its id: the same id can be used right away for a valid team.
- Vice-captain with an odd number of points gives an odd half-point score (1 point -> 3 half points).
- `DUCK` makes a player (and teams) negative; a team below zero ranks below teams at zero.
- Teams with identical rosters and captains always have identical scores, so they always share a rank.
- Correcting an event type that was never recorded for the player, or one more than is recorded, is an error; recording
  again after a correction makes room for another correction.
- Unknown player, event type `run` (wrong case), `count = 0` and negative counts are errors.
- Teams that have never scored are on the leaderboard with score 0.

## Glossary

- **Match points**: the points a player has earned so far from recorded events (minus corrections).
- **Half points**: the unit of team scores: 1 point = 2 half points, so x1.5 stays an integer.
- **Captain / vice-captain**: the two players of a team whose points count x2 and x1.5.
- **Competition ranking**: ties share the best rank, and the following rank is skipped (1, 2, 2, 4).
- **Creation order**: the order of successful `createTeam` calls; the tie-break on the leaderboard.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: every invalid input throws `IllegalArgumentException`: a null or blank id/name/owner, a role or event type that is not in the fixed lists (matching is exact and case-sensitive), `count < 1`, a negative `k`, an unknown player or team, a duplicate player id or team id, a roster that breaks a rule, a correction larger than the recorded count. A failed call changes nothing.
- Method names are those below; `recordEvent`/`correctEvent` take an `int count`, `teamScore`/`playerPoints` return `long`, `rankOf` returns `int`.
- Event names, role names and the points table above are exact. Events for players that are in no team are still recorded.
- Leaderboard lines are exactly `rank:teamId:scoreHalfPoints` with a plain decimal (possibly negative) integer score.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below. It is **not** your domain model — it is the doorway.
Create whatever types you like and wire them up behind it.

```java
package contract;

import java.util.List;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface FantasyLeaderboard {
    void addPlayer(String playerId, String name, String role);
    void createTeam(String teamId, String owner, List<String> playerIds, String captainId, String viceCaptainId);
    void recordEvent(String playerId, String eventType, int count);
    void correctEvent(String playerId, String eventType, int count);
    List<String> leaderboard(int k);          // "rank:teamId:scoreHalfPoints"
    int rankOf(String teamId);
    long teamScore(String teamId);            // half points
    long playerPoints(String playerId);       // whole points
}
```

Behaviour of each method is documented in the Javadoc of `src/main/java/contract/FantasyLeaderboard.java`.

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static FantasyLeaderboard create()
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any design behind it works. No test sleeps or relies on timing beyond the generous scale bound. More scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Every event type and the points table; captain x2, vice-captain x1.5 (odd half points), other teams unaffected.
- Immediate update of every team containing a player; teams created after events; negative scores.
- Ordering by score then creation order, competition ranks, `rankOf`, `leaderboard(k)` for k = 0, below and above the team count.
- `correctEvent`: reversal, per-type limits, failed corrections change nothing.
- Every roster rule and its boundary, duplicate team ids, blank and unknown inputs.
- Concurrency: 16 threads recording events while readers check that every snapshot is sorted, duplicate-free and shows identical teams equal; final totals equal the sequential result.
- Scale: 50,000 teams, 200,000 events, 1,000 `leaderboard(10)` calls.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/136-fantasy-leaderboard test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`Candidate`, `Concurrently`): `java/testkit`.
