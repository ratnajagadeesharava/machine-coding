# 12 · Travel Itinerary Planner (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 90 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

A travel app lets people plan trips: a date range, a budget, and a day-by-day schedule of activities in different
places. Build the planning core. It must refuse plans that are physically or financially impossible.

## Requirements

1. A **trip** has an id, a start day and an end day (inclusive, integer day numbers) and a **budget**.
2. An **activity** belongs to a trip and has an id (unique within the trip), a location, a day, a start and end time (minutes since midnight, `0 ≤ start < end ≤ 1440`) and a cost (≥ 0).
3. Adding an activity is refused when:
   - the trip is unknown, the day is outside the trip, the time range is invalid or the cost is negative, or the id already exists in the trip;
   - it **overlaps** another activity on the same day (touching end-to-start is fine);
   - the **travel time** between it and the activity just before it or just after it (on the same day) is too short — see below;
   - adding its cost would take the **total cost** above the budget.
4. **Travel time**: the planner knows travel minutes between pairs of locations (symmetric; unspecified pairs and identical locations need 0 minutes). Between two *consecutive* activities of the same day at different locations, the gap between the first's end and the second's start must be at least that travel time.
5. Removing an activity frees its time and budget. A removal is refused if it would leave two consecutive activities violating the travel-time rule.
6. The agenda of a day is the list of activity ids ordered by start time.
7. The total cost of a trip is the sum of its activities' costs.

## Constraints

- Up to 1,000 trips of up to 60 days with up to 30 activities per day; operations must be fast without re-sorting the whole day on every query.
- Travel times can be set at any moment; they apply to *future* additions (existing plans are not re-validated).

## Examples

```
createTrip("T", 10, 12, budget=500)
setTravelMinutes("Hotel", "Museum", 30)
addActivity("T","a1","Hotel", 10, 540, 600, 0)   → true   (09:00–10:00)
addActivity("T","a2","Museum",10, 620, 700, 50)  → false  (gap 20 < 30)
addActivity("T","a2","Museum",10, 630, 700, 50)  → true
agenda("T", 10) = [a1, a2]
```

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
