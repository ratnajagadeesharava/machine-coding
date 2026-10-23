# 02 · Elevator System (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

**What you are building.** The control brain for a bank of elevators in an office building. People press a **hall button** on a floor (up or down arrow) or a **cabin button** inside an elevator (a destination floor). Your controller decides which elevator answers each hall call and in what order each elevator visits its stops. Time is simulated: nothing moves until the caller invokes `step()`, and each `step()` is one tick in which every elevator does exactly one thing (open its doors, move one floor, or stay put). The controller must be deterministic and must never leave a request unserved.

## Requirements

1. The building has `F` floors (numbered `0..F-1`) and `E` elevators. All elevators start idle at floor 0 with doors closed.
2. **Hall calls**: a person on a floor requests to go `UP` or `DOWN` (`requestPickup(floor, direction)`). The system must decide which elevator serves it. The call is for "an elevator that will continue in that direction"; it carries no destination.
3. **Cabin requests**: a person inside a specific elevator requests a destination floor (`requestDestination(elevator, floor)`); only that elevator may serve it.
4. Time advances in discrete `step()`s. In one step each elevator does exactly one thing: open its doors at a floor where it has a stop, **or** move one floor toward its next stop, **or** stay idle.
5. When an elevator arrives at a floor with a stop it opens its doors **in that same step**; doors stay open for that step only and are closed again in the next step.
6. An elevator keeps travelling in its current direction while there are stops it can serve ahead; it serves cabin requests on the way and hall calls whose direction matches its travel direction. It reverses only when nothing remains ahead.
7. A hall call must be served by **one** elevator (not by all of them), and every request must eventually be served (no starvation).
8. Invalid requests (floor out of range, i.e. below 0 or at/above `F`; a direction other than exactly `UP` or `DOWN`; an elevator index outside `0..E-1`; `DOWN` at floor 0; `UP` at the top floor `F-1`) are rejected (return `false`) without side effects. A valid request returns `true`. Duplicate requests (same floor and direction, or same cabin and floor) are harmless: the floor is still served once.

## Constraints

- Up to 100 floors and 16 elevators; `step()` should be cheap enough to simulate 100,000 steps in a second or two.
- A request for the floor an idle elevator is already on opens its doors on the next step without moving.
- No threads are required — the simulation is deterministic and single-threaded.

## Examples

**1. One step at a time** (10 floors, 1 elevator, starts at floor 0)

| Call | floorOf(0) | doorsOpen(0) |
|---|---|---|
| `requestPickup(3, "UP")` -> `true` | 0 | false |
| `step()` | 1 | false |
| `step()` | 2 | false |
| `step()` | 3 | **true** (arrived, doors open in the same step) |
| `step()` | 3 | false (doors closed again, idle) |

**2. Serving on the way** (1 elevator): `requestDestination(0, 5)` then `requestPickup(3, "UP")`. The elevator is going up, the hall call also wants up, so doors open at floor 3 and then at 5: **3, 5**.

**3. Opposite direction is not served on the way** (1 elevator): `requestDestination(0, 5)` then `requestPickup(3, "DOWN")`. The elevator passes floor 3 without stopping, opens at 5, then returns: **5, 3**.

**4. Requests added mid-journey** (1 elevator): `requestDestination(0, 5)`, two steps (now at floor 2). Then `requestDestination(0, 3)` is ahead of the elevator -> stops are **3, 5**. Instead `requestDestination(0, 1)` is behind it -> stops are **5, 1**.

**5. Choosing among elevators** (12 floors, 2 elevators). Elevator 0 gets `requestDestination(0, 8)` and moves 3 steps (floor 3, still heading up).
- `requestPickup(5, "UP")`; two steps later elevator 0 is at floor 5 with doors open and elevator 1 is still at floor 0 (heading toward the call beats a farther idle elevator).
- Alternatively `requestPickup(2, "UP")`; two steps later elevator 1 is at floor 2 with doors open and elevator 0 is at floor 5 (elevator 0 already passed floor 2, so the idle elevator 1 is preferred).

**6. Rejections** (10 floors, 2 elevators): `requestPickup(10, "UP")`, `requestPickup(-1, "UP")`, `requestPickup(3, "SIDEWAYS")`, `requestPickup(0, "DOWN")`, `requestPickup(9, "UP")`, `requestDestination(2, 4)`, `requestDestination(0, 99)` all return `false`, and 20 further steps leave both elevators at floor 0 with doors closed.

## Edge cases

- A request for the floor an idle elevator is already on: it does not move; doors open in the next `step()`.
- With no requests, any number of steps leaves every elevator at floor 0, doors closed.
- The same floor requested several times (duplicate hall calls, plus a cabin request for it) opens the doors there once.
- A hall call is served by exactly one elevator: after it is answered, exactly one elevator has its doors open at that floor, the others are elsewhere.
- Under a long random mix of valid requests, every hall-call floor is eventually served by some elevator and every cabin-request floor by the elevator it was pressed in.

## Glossary

- **Hall call / pickup**: the up or down button on a floor.
- **Cabin request / destination**: a floor button pressed inside a particular elevator.
- **Stop**: a floor at which an elevator has a pending request and will open its doors.
- **Step / tick**: one unit of simulated time, advanced by `step()`.
- **Idle**: an elevator with no pending stops; it stays where it is.

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `requestPickup(floor, direction)`: direction is exactly `"UP"` or `"DOWN"`. Returns `true` if accepted, `false` if invalid.
- `requestDestination(elevator, floor)`: elevators are numbered from 0. Returns `true` if accepted, `false` if invalid.
- `step()` advances every elevator by one tick as described in the requirements.
- `floorOf(elevator)` is the current floor. `doorsOpen(elevator)` is `true` only during the step in which the doors were opened.
- Observable ordering (single elevator): requests that lie ahead in the direction of travel are served before ones behind it; a hall call in the *opposite* direction to travel is not served on the way.
- With several elevators, a hall call results in exactly one elevator opening its doors for it; an elevator already heading toward the call is preferred over a farther idle one, and a nearby idle elevator is preferred over one that has already passed the floor.
- `floorOf(elevator)` and `doorsOpen(elevator)` are read-only; reading them never advances time.
- A rejected request leaves every elevator's position, doors and pending stops unchanged.
- Hall-call direction is compared case-sensitively against the exact strings `"UP"` and `"DOWN"`.

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
