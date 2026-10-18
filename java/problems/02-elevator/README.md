# 02 · Elevator System (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Build the control logic for a bank of elevators serving a building. The system is driven by a discrete simulation:
people press buttons (outside on a floor, or inside an elevator) and the controller advances time one **step** at a time.

## Requirements

1. The building has `F` floors (numbered `0..F-1`) and `E` elevators. All elevators start idle at floor 0 with doors closed.
2. **Hall calls**: a person on a floor requests to go `UP` or `DOWN`. The system must decide which elevator serves it.
3. **Cabin requests**: a person inside an elevator requests a destination floor.
4. Time advances in discrete `step()`s. In one step each elevator does exactly one thing: open its doors at a floor where it has a stop, **or** move one floor toward its next stop, **or** stay idle.
5. When an elevator arrives at a floor with a stop it opens its doors **in that same step**; doors stay open for that step only and are closed again in the next step.
6. An elevator keeps travelling in its current direction while there are stops it can serve ahead; it serves cabin requests on the way and hall calls whose direction matches its travel direction. It reverses only when nothing remains ahead.
7. A hall call must be served by **one** elevator (not by all of them), and every request must eventually be served (no starvation).
8. Invalid requests (floor out of range, unknown direction, unknown elevator, `DOWN` at floor 0, `UP` at the top floor) are rejected without side effects. Duplicate requests are harmless.

## Constraints

- Up to 100 floors and 16 elevators; `step()` should be cheap enough to simulate 100,000 steps in a second or two.
- A request for the floor an idle elevator is already on opens its doors on the next step without moving.
- No threads are required — the simulation is deterministic and single-threaded.

## Examples

```
1 elevator, 10 floors. requestPickup(3, UP).
step → floor 1 | step → floor 2 | step → floor 3, doors OPEN | step → doors closed, idle

1 elevator. requestDestination(0, 5) and requestPickup(3, UP):   doors open at 3, then at 5
1 elevator. requestDestination(0, 5) and requestPickup(3, DOWN): doors open at 5, then at 3
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `requestPickup(floor, direction)`: direction is exactly `"UP"` or `"DOWN"`. Returns `true` if accepted, `false` if invalid.
- `requestDestination(elevator, floor)`: elevators are numbered from 0. Returns `true` if accepted, `false` if invalid.
- `step()` advances every elevator by one tick as described in the requirements.
- `floorOf(elevator)` is the current floor. `doorsOpen(elevator)` is `true` only during the step in which the doors were opened.
- Observable ordering (single elevator): requests that lie ahead in the direction of travel are served before ones behind it; a hall call in the *opposite* direction to travel is not served on the way.
- With several elevators, a hall call results in exactly one elevator opening its doors for it; an elevator already heading toward the call is preferred over a farther idle one, and a nearby idle elevator is preferred over one that has already passed the floor.

## Test entry point (Java)

The tests drive your system through the thin, primitive-typed contract below. It is **not** your domain
model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in
`solution/Solution.java`.

```java
package contract;

/** Thin test entry point. Your domain model lives behind it. */
public interface ElevatorApi {

    /** Hall call. direction is "UP" or "DOWN". Returns false for invalid requests. */
    boolean requestPickup(int floor, String direction);

    /** Cabin button inside the given elevator (0-based). Returns false for invalid requests. */
    boolean requestDestination(int elevator, int floor);

    /** Advances the simulation by one tick. */
    void step();

    int floorOf(int elevator);

    /** True only during the step in which the doors were opened. */
    boolean doorsOpen(int elevator);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    ElevatorApi create(int floors, int elevators)
}
```

## Language notes

`step()` is a pure simulation tick. Think about what the controller owns versus what each car owns before you write a class.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/02-elevator test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.
