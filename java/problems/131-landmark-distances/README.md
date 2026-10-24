# 131 · Shortest Distance and Nearest Landmarks (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 60 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

The routing core of a city-guide app. A city is a set of named locations joined by roads that have a length. Some locations are tagged as landmarks ("hospital", "cafe"). Users ask "how far is it from here to there?", "which way do I go?", "where are the three nearest cafes?" and "what can I reach within 2 km?". Roads open and close and landmarks come and go while people keep asking, and every answer must reflect the map as it is right now.

## Problem

Build an in-memory city map. Locations and roads are added and roads are removed over time; landmark tags are added and removed. Queries compute shortest distances, the actual route, the nearest landmarks of a kind and everything within a radius. Queries run concurrently with updates and must never observe a half-applied change. The map can be large: the tests use a 150 x 150 grid (22,500 locations, about 45,000 roads) and ask thousands of questions.

## Requirements

1. **Locations.** `addLocation(name)` adds a location. Names are case-sensitive and used exactly as given (`"a"` and `"A"` are different places). A null, empty or whitespace-only name, or a name that already exists, is an error.
2. **Roads are undirected.** `addRoad(a, b, distance)` joins two existing, different locations with a road of length `distance` (a `long`, must be > 0) that can be driven in both directions. A self-loop, a distance <= 0, or an unknown location is an error. Several roads may join the same pair (in either argument order); for routing the shortest one wins.
3. **Removing roads.** `removeRoad(a, b)` removes **every** road joining `a` and `b` (whichever order they were added in) and returns how many were removed. It returns 0 if there were none. Unknown locations are an error.
4. **Landmarks.** `tagLandmark(location, type)` marks a location as a landmark of a type; a location can carry several types. Types are trimmed and compared case-insensitively (`"  Cafe "` is `"cafe"`). A null or blank type is an error. Tagging the same type twice is harmless. `untagLandmark(location, type)` removes the tag and returns whether it was there.
5. **Distance.** `shortestDistance(a, b)` returns the smallest total length of any route from `a` to `b`, `0` if `a` equals `b`, and `-1` if there is no route.
6. **Route.** `shortestPath(a, b)` returns the location names along a shortest route, `a` first and `b` last. If `a` equals `b` the result is `[a]`; if there is no route the result is empty. When several routes have the same total length, return the one whose list of names is **lexicographically smallest**: compare the lists element by element from the first element, each pair of names with `String.compareTo`. (Both routes start at `a`, so the first difference decides.)
7. **Nearest landmarks.** `nearestLandmarks(source, type, k)` returns at most `k` landmarks of that type reachable from `source`, each formatted `"location:distance"`, ordered by distance ascending and then location name ascending (`String.compareTo`). If the source itself carries the type it counts, at distance 0. Unreachable landmarks are left out. A type that nobody carries gives an empty list. `k <= 0` is an error; so is a null/blank type.
8. **Radius.** `withinRadius(source, radius)` returns the names of all locations whose distance from `source` is `<= radius`, including `source` itself, sorted by name (`String.compareTo`). A negative radius is an error.
9. **Always current.** Every update (location, road added or removed, tag added or removed) is visible to the very next query. Nothing may answer from a stale view of the map.
10. **Large numbers.** Distances and totals are `long`; a route whose total exceeds the `int` range (for example three roads of 2,000,000,000) must be reported exactly.
11. **Failed calls change nothing.** A call that throws leaves the map exactly as it was.

## Constraints

- Thread-safe: many threads query while others update. A single query sees the map either before or after any given update, never a mixture, and no update is lost.
- Scale: on the 150 x 150 grid, 3,000 queries must finish well inside a generous bound (the test allows 90 seconds in total; a sensible solution needs a small fraction of that).

## Examples

```
addLocation: A B C D E          (E has no roads)
addRoad(A,B,4) addRoad(A,C,1) addRoad(C,B,2) addRoad(B,D,5) addRoad(C,D,8)

shortestDistance(A, D)   -> 8        route A-C-B-D (A-B-D and A-C-D both cost 9)
shortestPath(A, D)       -> [A, C, B, D]
shortestDistance(A, E)   -> -1       shortestPath(A, E) -> []
shortestDistance(E, E)   -> 0        shortestPath(E, E) -> [E]
withinRadius(A, 3)       -> [A, B, C]
```

```
tagLandmark(D, "cafe")   -> nearestLandmarks(A, "cafe", 2) = [D:8]
tagLandmark(C, "Cafe ")  -> nearestLandmarks(A, "cafe", 2) = [C:1, D:8]
removeRoad(A, C)         -> returns 1; nearestLandmarks(A, "cafe", 2) = [C:6, D:9]
nearestLandmarks(A, "museum", 2) = []        nearestLandmarks(A, "cafe", 0) -> IllegalArgumentException
```

```
Tie-break: S-a-c-T and S-b-d-T cost 3 each, S-a-T costs 5
shortestPath(S, T) -> [S, a, c, T]    (equal cost: [S,a,c,T] < [S,b,d,T]; the cost-5 route never competes)
```

```
Parallel roads: addRoad(A,B,10) addRoad(B,A,7) addRoad(A,B,9)
shortestDistance(A, B) -> 7
removeRoad(A, B)       -> 3          shortestDistance(A, B) -> -1
```

### Edge cases

- `shortestDistance` and `shortestPath` between a location and itself never fail for an existing location (`0` / `[a]`), even if it has no roads.
- Equal-distance landmarks are ordered by name with plain `String.compareTo` (so `"Shop" < "shop0" < "shop1" < "shop10" < "shop2"`).
- Tag lookups use the normalised type: `nearestLandmarks(.., "CAFE", ..)` finds what was tagged `" cafe"`.
- `withinRadius(x, 0)` is `[x]`. The radius test is `<=`, so a location exactly at the radius is included.
- Removing the last road of a pair does not remove either location or their tags.
- Unknown locations are checked by every method that takes a location (including `removeRoad`, `tagLandmark`, `untagLandmark`).

## Glossary

- **Location**: a named point on the map.
- **Road**: an undirected connection with a positive length. There may be several between the same pair.
- **Route / path**: a sequence of locations where consecutive ones are joined by a road; its distance is the sum of the roads used (on each hop the shortest road between that pair).
- **Landmark**: a location tagged with a type such as `hospital`.
- **Reachable**: a route exists.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `IllegalArgumentException` for every error above (null/blank/duplicate names, unknown locations, self-loop, non-positive distance, blank type, `k <= 0`, negative radius). Lists are plain `List<String>`.
- Rust: `Result<_, MapError>` with `UnknownLocation`, `DuplicateLocation`, `InvalidArgument`.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract in `src/main/java/contract/CityMap.java`. It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
public interface CityMap {
    void addLocation(String name);
    void addRoad(String a, String b, long distance);
    int removeRoad(String a, String b);
    void tagLandmark(String location, String type);
    boolean untagLandmark(String location, String type);
    long shortestDistance(String a, String b);
    List<String> shortestPath(String a, String b);
    List<String> nearestLandmarks(String source, String type, int k);
    List<String> withinRadius(String source, long radius);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static CityMap create()
}
```

## What the tests check

Black-box tests that only call the contract above, so any class design behind it works. No clocks or randomness are involved.

- Name, road and type validation; unknown locations in every method; failed calls change nothing.
- Distances, unreachable (-1 / empty), self distance, parallel roads, removal count, immediate visibility of updates.
- Totals beyond the `int` range; route names; tie-breaking by name sequence; nearest landmarks ordering, `k`, source at distance 0, case/whitespace-insensitive types; radius boundaries.
- A 150 x 150 grid with 3,000 distance queries; queries consistent during concurrent updates; no lost concurrent updates.
- More scenarios (huge tie sets, grid updates, concurrent tag/road flipping) are in `interviewer/` (see `INTERVIEWER_NOTES.md`).

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/131-landmark-distances test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`Candidate`, `Concurrently`): `java/testkit`.
