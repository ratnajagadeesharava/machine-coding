# 14 · Ride Sharing (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 90 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building the matching and trip-lifecycle core of a ride-hailing service on a city laid out as an integer grid.
Drivers are parked at grid points. When a rider asks for a ride from a pickup point to a drop point, the service
assigns the **nearest free driver** (if one is close enough). The ride then moves through stages - matched, in progress,
completed - or is cancelled before the trip starts. When a ride completes, the rider is charged a fare based on the
distance and the driver is free again, now sitting at the drop point.

### Terms

- **Location** - an integer grid point `(x, y)`; coordinates can be any `int` (negative included).
- **Manhattan distance** - `|x1 - x2| + |y1 - y2|`: blocks walked along the grid, no diagonals. `(0,0)` to `(4,4)` is 8.
- **Available / busy driver** - a driver is *busy* from the moment they are matched until the ride is completed or cancelled; otherwise *available*. A new driver starts available.
- **Active ride** - a ride whose status is `MATCHED` or `IN_PROGRESS`.
- **Ride status** - `MATCHED`, `IN_PROGRESS`, `COMPLETED`, `CANCELLED`, or `UNKNOWN` (for an id the system has never issued).

## Requirements

1. The city is an integer grid. Drivers have a location and are either available or busy.
2. A rider **requests a ride** from a pickup point to a drop point. The system matches the **nearest available driver** (Manhattan distance to the pickup); ties are broken by the **smallest driver id** (plain string comparison, `"d3" < "d5" < "d9"`). Only drivers within distance **10** of the pickup (distance `<= 10`, so exactly 10 is allowed and 11 is not) are eligible; otherwise the request is refused. A successful request creates a ride in status `MATCHED`.
3. A rider can have **at most one active ride** (matched or in progress) at a time. A rider with a completed or cancelled ride may request again.
4. A matched driver is **busy** until the ride is completed or cancelled, and cannot be matched to another rider.
5. Ride lifecycle: `MATCHED → IN_PROGRESS → COMPLETED`, or `MATCHED → CANCELLED`. A ride that is in progress cannot be cancelled; a completed or cancelled ride cannot change. Any other transition (starting twice, completing a ride that was never started, ...) is refused and changes nothing.
6. On completion the fare is `500 + 100 × Manhattan(pickup, drop)`. The driver becomes available again **at the drop location**.
7. Cancelling a matched ride makes its driver available again at their previous location (the driver never moved).
8. Queries: the status of a ride, the driver of a ride, the current location of a driver.

## Constraints

- Up to 100,000 drivers and 100,000 ride requests; matching should not feel like a linear scan of all drivers in a production setting (a scan passes the tests; know the alternatives).
- Operations on unknown ids are refused/empty, never an exception.

## Examples

```
addDriver(d1, 0, 0); addDriver(d2, 3, 0)
requestRide(r1, pickup=(1,0), drop=(5,5)) -> ride1, driver d1      d1 is 1 away, d2 is 2 away; statusOf(ride1) = MATCHED
requestRide(r2, pickup=(1,0), drop=(2,2)) -> ride2, driver d2      d1 is busy
requestRide(r3, pickup=(1,0), drop=(2,2)) -> empty                 both drivers busy
requestRide(r1, pickup=(1,0), drop=(2,2)) -> empty                 r1 already has an active ride
completeRide(ride1)                       -> empty                 still MATCHED, not started
startRide(ride1)                          -> true                  status IN_PROGRESS
cancelRide(ride1)                         -> false                 in progress cannot be cancelled
completeRide(ride1)                       -> 1400                  500 + 100 x (|1-5| + |0-5|) = 500 + 100 x 9
                                                                   d1 is now at "5,5" and available again
```

Tie-break and distance: drivers `d9` at (2,0), `d3` at (0,2), `d5` at (1,1), pickup (0,0): all are distance 2, so `d3` wins (smallest id).
Manhattan, not straight-line: driver `far-diagonal` at (4,4) is distance 8 and `near-straight` at (7,0) is distance 7 from (0,0), so
`near-straight` is chosen even though `far-diagonal` is closer in a straight line.

Cancel: driver `d1` at (0,0), ride matched with pickup (1,0). `cancelRide` -> `true`, status `CANCELLED`, `driverLocation(d1)` is still `"0,0"`,
and `d1` can be matched again immediately.

Out of range: only drivers at (0,0) and (3,0); `requestRide(r, pickup=(50,50), ...)` -> empty.

## Edge cases

- Every transition from a final state fails: after `COMPLETED` or `CANCELLED`, `startRide`, `completeRide` and `cancelRide` return `false`/empty.
- `driverLocation` returns the string `"x,y"` with no spaces (for example `"8,8"`, `"-3,4"`); empty for an unknown driver.
- A driver's location changes only at completion (to the drop point), never at match or start.
- Unknown ride ids: `statusOf` is `"UNKNOWN"`, `driverOf` is empty, `startRide` and `cancelRide` are `false`, `completeRide` is empty.
- A rider can request a new ride right after their previous ride was cancelled or completed.

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

Locations are plain `int` coordinates in the contract.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/14-ride-sharing test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.
