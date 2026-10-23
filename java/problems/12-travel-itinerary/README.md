# 12 · Travel Itinerary Planner (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 90 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building the planning core of a travel app. A user creates a **trip** (a range of days and a total budget) and
fills it with **activities** - a museum visit at 09:00, lunch at 13:00 - each at a location, on a day, with a start
time, an end time and a price. The planner must refuse plans that are physically or financially impossible: two things
at the same time, not enough time to travel between two places, or spending more than the budget.

### Terms

- **Trip** - id, first day and last day (both inclusive, plain integers), budget (money, whole units).
- **Activity** - id (unique within its trip), location (a string such as `"Hotel"`), day, start and end time, cost.
- **Minute-of-day** - a time written as minutes since midnight: `540` = 09:00, `720` = 12:00, `1440` = 24:00 (end of day).
- **Overlap** - two activities on the same day whose time ranges intersect. Ranges are half-open, so one ending at `600` and one starting at `600` do **not** overlap ("touching" is fine).
- **Consecutive** - two activities on the same day with no other activity between them in time order.
- **Travel time** - minutes needed to get from one location to another, configured with `setTravelMinutes`. It is symmetric (A to B equals B to A); a pair that was never configured, or two identical locations, needs 0 minutes.

## Requirements

1. A **trip** has an id, a start day and an end day (inclusive, integer day numbers) and a **budget**.
2. An **activity** belongs to a trip and has an id (unique within the trip, across all its days), a location, a day, a start and end time (minutes since midnight, `0 ≤ start < end ≤ 1440`) and a cost (≥ 0).
3. Adding an activity is refused (returns `false`, nothing changes) when:
   - the trip is unknown, the day is outside the trip (`day < startDay` or `day > endDay`), the time range is invalid or the cost is negative, or the id already exists in the trip;
   - it **overlaps** another activity on the same day (touching end-to-start is fine);
   - the **travel time** between it and the activity just before it or just after it (on the same day) is too short - see below;
   - adding its cost would take the **total cost** above the budget (spending exactly the budget is allowed).
4. **Travel time**: the planner knows travel minutes between pairs of locations (symmetric; unspecified pairs and identical locations need 0 minutes). Between two *consecutive* activities of the same day at different locations, the gap between the first's end and the second's start must be at least that travel time. A gap exactly equal to the travel time is allowed.
5. Removing an activity frees its time and budget. A removal is refused if it would leave two consecutive activities violating the travel-time rule.
6. The agenda of a day is the list of activity ids ordered by start time.
7. The total cost of a trip is the sum of its activities' costs.

## Constraints

- Up to 1,000 trips of up to 60 days with up to 30 activities per day; operations must be fast without re-sorting the whole day on every query.
- Travel times can be set at any moment; they apply to *future* additions (existing plans are not re-validated).

## Examples

Setup: `createTrip("T", 10, 12, budget=500)`, `setTravelMinutes("Hotel", "Museum", 30)`.

```
addActivity("T","a1","Hotel", 10, 540, 600, 0)   -> true    09:00-10:00
addActivity("T","a2","Museum",10, 620, 700, 50)  -> false   gap 20 < 30 minutes of travel
addActivity("T","a2","Museum",10, 630, 700, 50)  -> true    gap 30 == 30 is enough
agenda("T", 10)  = [a1, a2]                      totalCost("T") = 50
```

Symmetry and both neighbours (Hotel activities `early` 480-540 and `late` 700-760 on day 10):

```
addActivity("T","m1","Museum",10, 560,650,0) -> false   only 20 min after early
addActivity("T","m2","Museum",10, 570,680,0) -> false   only 20 min before late
addActivity("T","m3","Museum",10, 570,670,0) -> true    30 after early, 30 before late
```

Budget 500: add cost 300 -> true; add cost 201 -> false (501 > 500); add cost 200 -> true (total exactly 500);
add cost 1 -> false. Removing the 300 activity makes `totalCost` 200 again.

Refused removal (travel times: Hotel-Museum 30, everything else 0): day has `Hotel 540-600`, `Beach 600-610`, `Museum 610-700`.
`removeActivity` of the Beach activity returns `false`, because Hotel would then be directly followed by Museum with a
gap of 10 < 30. Nothing changes.

## Edge cases

- Invalid times: `start == end`, `start > end`, `start < 0` and `end > 1440` are all refused. Day `startDay - 1` and `endDay + 1` are refused.
- Touching activities at the same location (`480-600` then `600-660`) are allowed, as is an activity ending exactly when another begins.
- A duplicate id is refused even when it is on a different day of the same trip; the same id may be used in a different trip. Trips are fully independent (separate budgets and agendas).
- `agenda` of an unknown trip or an empty day is an empty list; `totalCost` of an unknown trip is `0`; `removeActivity` of an unknown trip or id is `false`.
- Changing travel minutes later never invalidates activities already in the plan.

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `addActivity` returns `true` only when the activity was added; every refusal leaves the plan unchanged.
- `removeActivity` returns `true` when removed; `false` when the trip/activity is unknown, or when removing would leave consecutive activities violating the travel-time rule (nothing changes then).
- `agenda(trip, day)` is empty for unknown trips and days without activities. `totalCost(trip)` is 0 for unknown trips.
- Day numbers are inclusive for trips (`startDay ≤ day ≤ endDay`).

## Test entry point (Java)

The tests drive your system through the thin, primitive-typed contract below. It is **not** your domain
model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in
`solution/Solution.java`.

```java
package contract;

import java.util.List;

/** Thin test entry point. Your domain model lives behind it. */
public interface ItineraryApi {

    void createTrip(String tripId, int startDay, int endDay, long budget);

    /** Symmetric travel time between two locations; applies to future additions. */
    void setTravelMinutes(String locationA, String locationB, int minutes);

    boolean addActivity(String tripId, String activityId, String location,
                        int day, int startMinute, int endMinute, long cost);

    boolean removeActivity(String tripId, String activityId);

    /** Activity ids of the day ordered by start time. */
    List<String> agenda(String tripId, int day);

    long totalCost(String tripId);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    ItineraryApi create()
}
```

## Language notes

Minutes are `int` (0..1440), money is `long`. The `agenda` returns ids only.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/12-travel-itinerary test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.
