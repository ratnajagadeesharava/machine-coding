# 114 · Elevator Request Feasibility (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

A building has one lift. Passengers ask for rides ("floor 3 to floor 8, two of us") and the controller must answer **immediately** with *accepted* or *rejected*. An accepted request is a **promise**: the lift will carry that group, it will never be overloaded, and nobody who was already promised a ride will be made to stop more often than the building's comfort limit allows because of a newcomer. A rejection must leave the controller exactly as it was. You are building that decision-maker and the lift that keeps the promises: the lift moves in a fixed, deterministic circuit, so "will this plan work?" has exactly one answer.

## Problem

The building has floors `0..floors-1`, a lift with a `capacity` in persons, and a comfort limit `maxIntermediateStops`: **no rider may sit through more than that many intermediate stops** between boarding and getting off. The lift follows a fixed circuit (below), so given the riders already promised and a new request, you can work out exactly which floors it will stop at, who is inside at every moment and how many stops each rider will sit through. Accept the request only if that future is acceptable for everybody, the newcomer included. The lift can also be driven one stop at a time, so promises must hold as it moves and as more requests arrive in between.

## Requirements

1. The controller is created with `floors >= 2` (floors are `0..floors-1`), `capacity >= 1` persons and `maxIntermediateStops >= 0`. The lift starts empty and idle at floor 0, heading up.
2. `request(requestId, fromFloor, toFloor, persons)` answers accepted or rejected at once. A request is one group of `persons` that travels together, boards at `fromFloor` and leaves at `toFloor`. The ride is **upward** if `toFloor > fromFloor`, otherwise **downward**.
3. **The circuit.** The lift travels up from floor 0 to floor `floors-1`, then down to floor 0, then up again, and so on, forever. It is heading **up** at floors `0..floors-2` on the way up and heading **down** at floors `floors-1..1` on the way down; so floor 0 is only ever visited heading up and the top floor only ever heading down.
4. **Stops.** A *stop* is the lift opening its doors at a floor. While following the circuit the lift stops at a floor only if (a) somebody onboard has that floor as destination, or (b) somebody is waiting at that floor whose ride goes in the direction the lift is heading at that moment. Otherwise it passes without stopping. Floors passed without a stop are not counted anywhere.
5. **What happens at a stop**, in this order: everybody onboard whose destination is this floor gets off; then every waiting rider at this floor whose ride matches the heading gets on, all of them (nobody is left behind at a stop). The same floor can be a stop twice in a row only if new requests are accepted in between (see edge cases).
6. **Intermediate stops** of a rider = the number of stops the lift makes **after** the stop where they boarded and **before** the stop where they get off, whoever caused those stops. Stops already made while the rider is onboard count.
7. A request is **feasible** if, when the lift follows the circuit from its present position with every promised rider plus the new one, (a) the number of persons inside never exceeds `capacity` (measured after riders have got off and the waiting ones have got on at each stop) and (b) every rider, including the new one, has at most `maxIntermediateStops` intermediate stops. Only feasible requests are accepted. A request with `persons > capacity` can never be feasible and is rejected.
8. A rejected request changes nothing: not the plan, not the position, not the load, not the status of any request, and its id is not taken.
9. `step()` makes the next stop of the plan (the lift moves there, riders get off and on) and returns its floor. It returns "nothing" and does nothing when no accepted rider is waiting or onboard.
10. `currentFloor()`, `onboard()` (total persons inside), `plan()` (floors of all upcoming stops, in order, up to the delivery of the last accepted rider) and `status(requestId)` (`WAITING`, `ONBOARD`, `DELIVERED`, or `UNKNOWN` for an id that was never accepted) report the present state. Calling `step()` repeatedly returns exactly what `plan()` returned, as long as nothing is accepted in between.
11. Invalid input is an error and changes nothing (see the behavioral contract).

## Constraints

- The controller is used from many threads at once (several clients requesting, one thread stepping). Every `request` is judged against, and recorded into, a consistent state: two concurrent requests must not both be accepted if together they break a promise or the capacity.
- The decision must be taken from the future the circuit produces, not from a counter you adjust per request: the tests include cases where one extra stop breaks somebody else, where a stop is shared, and where a rider has already sat through part of their stops.
- No clock, no randomness: the same sequence of calls gives the same answers in every run.

## Examples

All examples use a 10-floor building (floors 0..9) and `capacity = 4`.

```
maxIntermediateStops = 0
request(r1, 0, 5, 1)  -> accepted      plan() = [0, 5]
request(r2, 0, 7, 1)  -> rejected      (the lift would stop at 5 for r1; r2 would sit through that stop)
request(r3, 5, 8, 1)  -> accepted      plan() = [0, 5, 8]     (r1 leaves at 5, r3 boards at 5)

maxIntermediateStops = 1
request(r1, 0, 9, 1)  -> accepted      plan() = [0, 9]
request(r2, 3, 5, 1)  -> rejected      (stops 3 and 5 would both fall inside r1's ride: 2 > 1)
request(r3, 0, 5, 1)  -> accepted      plan() = [0, 5, 9]     (only stop 5 is new for r1)
request(r4, 0, 3, 1)  -> rejected      (r1 would sit through 3 and 5)

maxIntermediateStops = 5
request(r1, 0, 8, 3)  -> accepted
request(r2, 3, 6, 2)  -> rejected      (3 + 2 persons between floors 3 and 6 > 4)
request(r3, 3, 6, 1)  -> accepted
request(r5, 8, 9, 4)  -> accepted      (r1 gets off at 8 before r5 gets on)

maxIntermediateStops = 5
request(down, 6, 2, 1) -> accepted     plan() = [6, 2]       (up to 9 without stopping, then down: boards at 6)
step() -> 6                            currentFloor() = 6, heading down
request(up, 3, 5, 1)  -> accepted      plan() = [2, 3, 5]    (down to 2, turn at 0, board the upward rider at 3)
```

### Worked traces

```
maxIntermediateStops = 1
request(r1, 0, 9, 1); request(r2, 0, 4, 1)           both accepted, plan() = [0, 4, 9]   (r1 sits through 4: 1 <= 1)
step() -> 0     onboard() = 2
step() -> 4     r2 DELIVERED, r1 has now sat through 1 stop
request(r3, 6, 9, 1) -> rejected    (stop 6 would be r1's second intermediate stop)
-- compare: a fresh lift with r1 = 0->9 only: request(r3, 6, 9, 1) -> accepted, plan() = [0, 6, 9]

maxIntermediateStops = 0
request(r1, 0, 9, 1); step() -> 0
request(r2, 0, 9, 1) -> rejected    (a stop at 0 would open the doors again while r1 is inside)

maxIntermediateStops = 0
request(r1, 0, 9, 1); request(r2, 9, 0, 1)   plan() = [0, 9, 0]   (floor 9 is one stop: r1 off, r2 on)

request(r1, 0, 5, 1); step(); step()         lift at 5, r1 DELIVERED, lift heading up
request(r2, 5, 8, 1); request(r3, 5, 2, 1)   plan() = [5, 8, 5, 2]
   (r2 boards at 5 right now, heading up; r3 needs the way down, so the lift goes up to the top, turns and stops at 5 again)

`request("r", 4, 4, 1)` or `persons = 0` -> `IllegalArgumentException`
```

### Edge cases

- The ride direction must match the lift's heading at the floor where the rider waits. An upward rider at floor 3 when the lift is heading down is picked up only after the lift has turned at the bottom.
- A rider waiting at the floor where the lift is standing is picked up by the next stop (a stop at the same floor again) if the lift's current heading matches their ride. That stop is a stop like any other: it counts as an intermediate stop for riders who are onboard.
- At the same stop riders get off **before** others get on, so a full lift can swap passengers there.
- Persons are counted, not riders: a group of 3 and a group of 2 do not fit together in a lift of capacity 4.
- A request whose groups never overlap in time with anyone else is trivially feasible even when the lift is far away: stops caused by travelling to it count only for riders who are onboard at that time.
- `persons > capacity` is a normal rejection, not an error. A request for the floor you are already on (`from == to`) is an error.
- An id is taken while its rider is waiting or onboard. A rejected request does not take its id.
- `step()` with nothing to do returns "nothing" and leaves `currentFloor()` unchanged.
- Feasibility is judged against **every** promise at once: a request may be fine for the newcomer and for most riders, and still be rejected because of one rider far away in the circuit.

## Glossary

- **Circuit**: the lift's fixed path, floor 0 up to the top, then down to floor 0, repeated.
- **Heading**: the direction the lift is moving at a floor on the circuit (up, or down).
- **Stop**: the lift opening its doors at a floor because someone leaves or boards there.
- **Plan**: the ordered stops the lift will make until every accepted rider has been delivered.
- **Intermediate stops (of a rider)**: stops strictly between the rider's boarding stop and their exit stop.
- **Promise**: an accepted request; later requests may not break it.
- **Load**: persons inside the lift at a moment.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `IllegalArgumentException` for a null/blank `requestId`, an id that belongs to a rider that is still waiting or onboard, a floor outside `0..floors-1`, `fromFloor == toFloor`, and `persons <= 0`. `step()` returns `-1` when there is nothing to do.
- `request` returns accepted/rejected for valid input; `persons > capacity` is rejected, not an error.
- `plan()` is the exact sequence of floors `step()` will return, as long as nothing else is accepted; it is empty when idle.
- Capacity is checked at every stop after riders have got off and the waiting ones have got on; a rejected request leaves plan, position, load and statuses untouched.
- Stops already made count towards a rider who is onboard.
- `status` is `"WAITING"`, `"ONBOARD"`, `"DELIVERED"` or `"UNKNOWN"`.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below. It is **not** your domain model: it is the doorway. Create whatever you like and wire it up in the entry point.

```java
package contract;

import java.util.List;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface LiftController {

    /**
     * Asks to be carried from fromFloor to toFloor. true = accepted (the promise is recorded);
     * false = rejected (the state is exactly as before the call). Accepted only if, with the new request added,
     * the load never exceeds the capacity and every rider (the new one and all earlier ones) reaches their
     * destination with at most the configured number of intermediate stops. persons > capacity is rejected (false).
     * IllegalArgumentException for bad input (see above).
     */
    boolean request(String requestId, int fromFloor, int toFloor, int persons);

    /** Moves to the next stop, lets riders off and on. Returns that floor, or -1 (and does nothing) if idle. */
    int step();

    int currentFloor();

    /** Total persons currently inside the lift. */
    int onboard();

    /** Floors of all upcoming stops in the order they will be made, until every accepted rider is delivered. */
    List<Integer> plan();

    /** "WAITING", "ONBOARD", "DELIVERED" or "UNKNOWN". */
    String status(String requestId);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static LiftController create(int floors, int capacity, int maxIntermediateStops)
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any design behind it works. There is no clock. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Idle lift, single ride, a downward ride on the downward sweep, request lifecycle, `step()` returning exactly `plan()`.
- Overlapping routes: shared stops, a stop that breaks someone else's limit, a stop that breaks the newcomer's own limit, limit 0.
- Capacity: pickup overload, mid-route overload, swapping passengers at one stop, a group bigger than the lift.
- Rejection leaves no trace, stops already made count, direction handling (turning at the bottom/top, pickup at the current floor).
- Invalid input and duplicate ids; concurrency: 8 threads racing for the last seats, 160 concurrent requests none lost.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/114-elevator-feasibility test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`Candidate`, `Concurrently`): `java/testkit`.
