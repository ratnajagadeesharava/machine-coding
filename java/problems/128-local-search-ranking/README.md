# 128 · Local Search with Ranking (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 60 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

The "restaurants near me" box of a maps app. A catalogue holds places (cafes, pizzerias, pharmacies) with a name, a category, a position on an integer grid, a rating and a price level. A user standing at some grid point types an optional text, picks optional filters and gets back the best few places, **ranked** by a score that rewards good ratings and nearness and gives a small bonus to names that start with, or equal, what was typed. Owners add, edit and remove places while users search, and a search must never observe a place that is half edited.

## Problem

Build the in-memory search core for the catalogue. Places are added, updated and removed by id. A search takes the user position, a radius, optional text and category, a minimum rating, a maximum price level and a result limit `k`, and returns the ids of the top `k` places in ranking order. Two helper queries ignore ranking: `nearest` (the places closest to a point) and `categories` (how many places each category has).

## Requirements

**Places**

1. `addPlace(placeId, name, category, x, y, ratingTimes10, priceLevel)` adds a place. `x` and `y` are integer grid coordinates (any `int`, negative allowed). `ratingTimes10` is the rating multiplied by ten, an integer in `0..50` (so 4.5 stars is `45`). `priceLevel` is `1..4`.
2. A null or blank `placeId`, `name` or `category`, a rating outside `0..50` or a price level outside `1..4` is an error. Adding an id that already exists is an error. A rejected call changes nothing.
3. `updatePlace(...)` takes the same arguments and replaces **every** attribute of an existing place at once. An unknown id or invalid values are an error and change nothing.
4. `removePlace(placeId)` deletes a place; an unknown id is an error. A removed id can be added again.
5. Categories are case-insensitive: `Cafe`, `cafe` and `CAFE` are one category. `categories()` reports them in lower case. Place names keep their original text but are compared without regard to case.

**Search**

6. `search(query, userX, userY, radius, category, minRatingTimes10, maxPriceLevel, k)` returns at most `k` place ids, best first. `radius < 0` or `k <= 0` is an error.
7. **Distance** is the Manhattan distance `|x - userX| + |y - userY|`. Coordinates, radius and distance can be as large as the `int` range allows; distances may not fit in an `int`.
8. A place is a **candidate** only if it passes all of these filters:
   - `distance <= radius`;
   - `category` is null/blank (no filter) or equals the place's category ignoring case;
   - `rating >= minRatingTimes10` and `priceLevel <= maxPriceLevel` (any integers are allowed here; nothing is rejected);
   - `query` is null or blank (no text filter), or it is a case-insensitive substring of the place name, or it equals the place's category ignoring case (the whole category, not a part of it). The query is used exactly as given (it is not trimmed).
9. **Score** (an integer, computed exactly like this):

   ```
   score = ratingTimes10 * 10
           - 3 * distance
           + (nameStartsWithQuery ? 25 : 0)
           + (nameEqualsQuery     ? 25 : 0)
   ```

   `nameStartsWithQuery` is true when the query is present (non-blank) and the place name starts with it ignoring case; `nameEqualsQuery` when it equals the whole name ignoring case. An exactly equal name satisfies both, so it gets `+50`. The bonuses depend on the name only, not on how the place matched; with no query there is no bonus.
10. **Order:** higher score first; ties by smaller distance; remaining ties by `placeId` ascending (ordinary string order).
11. `nearest(userX, userY, k)` returns the `k` places with the smallest distance among **all** places (no other filter), ties by `placeId` ascending. `k <= 0` is an error. If there are fewer than `k` places it returns all of them.
12. `categories()` returns one `"category:count"` string per category that currently has at least one place, sorted by the category text ascending, e.g. `["bar:1", "cafe:2"]`.

## Constraints

- Many threads add, update and remove places while others search. The implementation must be thread-safe, and a search must never see a half-updated place: it sees a place entirely as it was before an update or entirely as it is after it. No add may be lost.
- **Sizes the tests use:** 50,000 places and 2,000 searches (a mix of text, category and radius from 300 to 700 on a 2,000 x 2,000 grid, `k = 10`) must finish within 30 seconds; the interviewer set goes up to 100,000 places and 10,000 searches. The tests do not require any particular approach; they only check the time bound.
- `k` can be as large as `Integer.MAX_VALUE`; the result simply holds every candidate.
- No wall-clock time and no randomness are involved in any result.

## Examples

```
addPlace("c1", "Coffee",          "cafe",   3, 0, 40, 2)
addPlace("c2", "Coffee House",    "cafe",   1, 0, 40, 2)
addPlace("c3", "Iced Coffee Bar", "cafe",   0, 2, 45, 3)
addPlace("c4", "Bean Cart",       "coffee", 5, 0, 30, 1)
addPlace("p1", "Pizza",           "pizza",  1, 1, 50, 2)

search("coffee", 0,0, radius 100, category null, minRating 0, maxPrice 4, k 10)
  c3: 450 - 3*2        = 444   (name contains "coffee": no bonus)
  c1: 400 - 3*3 + 25+25 = 441  (name equals the query)
  c2: 400 - 3*1 + 25    = 422  (name starts with the query)
  c4: 300 - 3*5         = 285  (category equals the query, no name bonus)
  p1 does not match the text
  -> [c3, c1, c2, c4]

search("coffee", 0,0, 100, null, minRating 40, maxPrice 2, 10)  -> [c1, c2]   (c3 costs 3, c4 is rated 3.0)
search(null,     0,0, 100, "CAFE", 0, 4, 2)                     -> [c3, c2]   (scores 444, 397, 391; k = 2; category ignores case)
search("coffee", 0,0, radius 3, null, 0, 4, 10)                 -> [c3, c1, c2]   (c4 is at distance 5)
nearest(0, 0, 3)                                                -> [c2, c3, p1]   (distances 1, 2, 2; the tie goes to "c3" < "p1")
categories()                                                    -> ["cafe:3", "coffee:1", "pizza:1"]
```

Equal scores:

```
addPlace("far",   "Far",    "cafe", 10, 0, 43, 2)    score 430 - 30 = 400, distance 10
addPlace("close", "Close",  "cafe",  0, 0, 40, 2)    score 400,            distance 0
addPlace("b", "Twin B", "cafe", 0, 5, 41, 2)         score 410 - 15 = 395, distance 5
addPlace("a", "Twin A", "cafe", 5, 0, 41, 2)         score 395,            distance 5
search(null, 0,0, 100, null, 0, 4, 10) -> [close, far, a, b]
```

Update and remove:

```
addPlace("a", "Old Name", "cafe", 50, 50, 10, 1)
addPlace("b", "Rival",    "cafe",  1,  0, 30, 2)
search(null, 0,0, 200, null, 0, 4, 10)  -> [b, a]
updatePlace("a", "New Diner", "diner", 0, 1, 50, 3)
search(null, 0,0, 200, null, 0, 4, 10)  -> [a, b]
search("old", ...)                       -> []
categories()                             -> ["cafe:1", "diner:1"]
removePlace("a"); categories()           -> ["cafe:1"]
```

### Edge cases

- A place at exactly `radius` is included; `radius = 0` returns only places at the user's own position.
- Scores can be negative (a far, poorly rated place); ordering still follows the rules above.
- Rating filter is inclusive (`>=`), price filter is inclusive (`<=`).
- A blank query (`""`, `"   "`) is the same as no query: no text filter and no bonus.
- A query that is a *part* of the category (`"india"` for `indian`) does not match through the category; it can still match through the name.
- A failed `updatePlace` (unknown id, invalid values) leaves the old place untouched.
- Fewer matching places than `k` is fine; nothing matching returns an empty list.
- `nearest` ignores the text, category, rating and price filters and the radius.
- Coordinates near `Integer.MIN_VALUE` / `MAX_VALUE`: the distance between two places can exceed `Integer.MAX_VALUE`.

## Glossary

- **Place**: one catalogue entry: id, name, category, grid position, rating (times ten), price level.
- **Manhattan distance**: `|dx| + |dy|`, the number of grid steps with no diagonals.
- **Candidate**: a place that passes every filter of a search.
- **Score**: the integer a candidate is ranked by (requirement 9).
- **ratingTimes10**: the star rating multiplied by ten, so that all arithmetic stays in integers.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `IllegalArgumentException` for every error listed above (blank/null ids, names and categories, rating/price out of range, duplicate or unknown id, `k <= 0`, `radius < 0`, `nearest` with `k <= 0`).
- Rust: `Result<_, SearchError>` with `DuplicatePlace`, `UnknownPlace` and `InvalidArgument`.
- Results are lists of place ids, best first. The returned list may be modified by the caller without affecting the system.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.List;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface PlaceSearch {

    void addPlace(String placeId, String name, String category, int x, int y, int ratingTimes10, int priceLevel);

    void updatePlace(String placeId, String name, String category, int x, int y, int ratingTimes10, int priceLevel);

    void removePlace(String placeId);

    List<String> search(
            String query, int userX, int userY, int radius, String category,
            int minRatingTimes10, int maxPriceLevel, int k);

    List<String> nearest(int userX, int userY, int k);

    List<String> categories();
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static PlaceSearch create()
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. More scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- The score formula, the bonuses (prefix, stacked equality, name only), and the ordering with tie-breaks by distance and id.
- Every filter (radius boundary, category ignoring case, rating, price, text by name and by category, blank queries).
- Add / update / remove / re-add, error cases and "failed call changes nothing", `nearest`, `categories`.
- Concurrency: a place flipped between two complete states is never seen as a mixture; 16 threads adding 250 places each lose none.
- Scale: 50,000 places and 2,000 searches within 30 seconds.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/128-local-search-ranking test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`Candidate`, `Concurrently`): `java/testkit`.
