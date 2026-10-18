# 01 · Parking Lot (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 90 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

A multi-storey car park needs software that decides where arriving vehicles go, issues tickets, and charges
for the time spent when they leave. Design and build it.

## Requirements

1. The car park has one or more floors. Every floor has some number of **small**, **medium** and **large** spots.
2. Vehicles are motorcycles, cars or trucks. A motorcycle fits any spot, a car fits a medium or large spot, a truck fits only a large spot.
3. When a vehicle enters, the system picks a spot for it and returns a ticket. If nothing fits, entry is refused.
4. Spots are allocated so that the **smallest sufficient spot size** is used first; among spots of that size, the **lowest floor** wins.
5. A vehicle (identified by plate) can be parked at most once at a time.
6. On exit the ticket is presented, the spot becomes available again, and the fee is returned.
7. Fees are charged per started hour with a minimum of one hour. Hourly rates: motorcycle 1000, car 2000, truck 5000 (smallest currency unit).
8. A ticket can be used to exit exactly once.
9. Callers can ask how many spots could currently accept a given vehicle type.

## Constraints

- Up to 100 floors and 10,000 spots; 100,000 enter/exit operations should complete in a few seconds.
- Time comes from the `Clock` handed to your factory; never read the wall clock directly.
- The first version may be single-threaded, but the contract tests include concurrent entry/exit — the end state must be consistent (no double allocation, no lost spots).

## Examples

```
lot: 1 floor, 1 small, 1 medium, 1 large
enter(MOTORCYCLE, "M-1")  -> ticket T1   (goes into the small spot)
enter(CAR, "C-1")         -> ticket T2   (medium)
enter(CAR, "C-2")         -> ticket T3   (large; medium is taken)
enter(CAR, "C-3")         -> refused
after 90 minutes: exit(T2) -> fee 4000   (2 started hours × 2000)
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- Vehicle types are the exact strings `MOTORCYCLE`, `CAR`, `TRUCK`. Any other type is refused on entry.
- `enter` returns a non-empty, unique ticket id, or empty when: no fitting spot is free, the plate is already parked, or the type is unknown.
- `exit` returns the fee, or empty when the ticket is unknown or already used.
- Fee = `max(1, ceil(elapsed / 1 hour)) × hourlyRate`. Exactly 60 minutes → 1 hour; 61 minutes → 2 hours; immediate exit → 1 hour.
- `availableSpots(type)` = number of currently free spots that could accept that type (spots are counted per fitting size, not reserved). Unknown type → 0.
- `floorOf(ticket)` is the 0-based floor of the spot held by an active ticket; empty for unknown/used tickets.
- Spot choice: smallest fitting size first, then lowest floor.

## Test entry point (Java)

The tests drive your system through the thin, primitive-typed contract below. It is **not** your domain
model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in
`solution/Solution.java`.

```java
package contract;

import java.util.Optional;
import java.util.OptionalInt;

/** Thin test entry point. Your domain model lives behind it. Must be safe to call from multiple threads. */
public interface ParkingLotApi {

    /** Parks the vehicle. Returns a ticket id, or empty if refused. */
    Optional<String> enter(String vehicleType, String plate);

    /** Releases the spot of the ticket and returns the fee, or empty if the ticket is unknown/already used. */
    Optional<Long> exit(String ticketId);

    /** Number of currently free spots that could accept this vehicle type. */
    int availableSpots(String vehicleType);

    /** 0-based floor of the spot held by an active ticket. */
    OptionalInt floorOf(String ticketId);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    ParkingLotApi create(int floors, int smallPerFloor, int mediumPerFloor, int largePerFloor, Clock clock)
}
```

## Language notes

`Clock` is `java.time.Clock`; call `clock.millis()`. The contract methods may be called from many threads in the concurrency tests.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/01-parking-lot test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.
