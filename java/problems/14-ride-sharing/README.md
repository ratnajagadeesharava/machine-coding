# 14 · Ride Sharing (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 90 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Build the matching and trip-lifecycle core of a ride-hailing service on a city grid: riders request a ride, the
nearest available driver is matched, the ride progresses through its stages, and a fare is computed on completion.

## Requirements

1. The city is an integer grid. Drivers have a location and are either available or busy.
2. A rider **requests a ride** from a pickup point to a drop point. The system matches the **nearest available driver** (Manhattan distance to the pickup); ties are broken by the **smallest driver id**. Only drivers within distance **10** of the pickup are eligible; otherwise the request is refused.
3. A rider can have **at most one active ride** (matched or in progress) at a time.
4. A matched driver is **busy** until the ride is completed or cancelled.
5. Ride lifecycle: `MATCHED → IN_PROGRESS → COMPLETED`, or `MATCHED → CANCELLED`. A ride that is in progress cannot be cancelled; a completed or cancelled ride cannot change.
6. On completion the fare is `500 + 100 × Manhattan(pickup, drop)`. The driver becomes available again **at the drop location**.
7. Cancelling a matched ride makes its driver available again at their previous location.
8. Queries: the status of a ride, the driver of a ride, the current location of a driver.

## Constraints

- Up to 100,000 drivers and 100,000 ride requests; matching should not feel like a linear scan of all drivers in a production setting (a scan passes the tests; know the alternatives).
- Operations on unknown ids are refused/empty, never an exception.

## Examples

```
addDriver(d1, 0, 0); addDriver(d2, 3, 0)
requestRide(r1, pickup=(1,0), drop=(5,5)) → ride1, driver d1 (distance 1 < 2)
requestRide(r2, pickup=(1,0), drop=(2,2)) → ride2, driver d2 (d1 is busy)
startRide(ride1); completeRide(ride1) → 500 + 100×(4+5) = 1400    d1 is now at (5,5), available
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `requestRide` returns a ride id, or empty if no eligible driver exists, the rider already has an active ride, or an input is invalid.
- `statusOf(ride)` is one of `MATCHED`, `IN_PROGRESS`, `COMPLETED`, `CANCELLED`, or `UNKNOWN`.
- `startRide` returns `true` only from `MATCHED`. `completeRide` returns the fare only from `IN_PROGRESS`. `cancelRide` returns `true` only from `MATCHED`.
- `driverLocation(id)` is `"x,y"` (e.g. `"5,5"`); empty for unknown drivers.
- Distance is Manhattan: `|x1 − x2| + |y1 − y2|`; eligibility is `distance ≤ 10`.

## Test entry point (Java)

The tests drive your system through the thin, primitive-typed contract below. It is **not** your domain
model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in
`solution/Solution.java`.

```java
package contract;

import java.util.Optional;

/** Thin test entry point. Your domain model lives behind it. */
public interface RideSharingApi {

    void addDriver(String driverId, int x, int y);

    /** Matches the nearest available driver within distance 10 and returns the ride id. */
    Optional<String> requestRide(String riderId, int pickupX, int pickupY, int dropX, int dropY);

    Optional<String> driverOf(String rideId);

    /** MATCHED, IN_PROGRESS, COMPLETED, CANCELLED or UNKNOWN. */
    String statusOf(String rideId);

    boolean startRide(String rideId);

    /** Returns the fare. */
    Optional<Long> completeRide(String rideId);

    boolean cancelRide(String rideId);

    /** "x,y" or empty. */
    Optional<String> driverLocation(String driverId);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    RideSharingApi create()
}
```

## Language notes

Locations are plain `int` coordinates in the contract; you may introduce your own `Location` type.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/14-ride-sharing test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.
