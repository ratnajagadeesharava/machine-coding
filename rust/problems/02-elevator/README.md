# 02 · Elevator System (Rust)

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
2. **Hall calls**: a person on a floor requests to go `UP` or `DOWN` (`request_pickup(floor, direction)`). The system must decide which elevator serves it. The call is for "an elevator that will continue in that direction"; it carries no destination.
3. **Cabin requests**: a person inside a specific elevator requests a destination floor (`request_destination(elevator, floor)`); only that elevator may serve it.
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

| Call | floor_of(0) | doors_open(0) |
|---|---|---|
| `request_pickup(3, "UP")` -> `true` | 0 | false |
| `step()` | 1 | false |
| `step()` | 2 | false |
| `step()` | 3 | **true** (arrived, doors open in the same step) |
| `step()` | 3 | false (doors closed again, idle) |

**2. Serving on the way** (1 elevator): `request_destination(0, 5)` then `request_pickup(3, "UP")`. The elevator is going up, the hall call also wants up, so doors open at floor 3 and then at 5: **3, 5**.

**3. Opposite direction is not served on the way** (1 elevator): `request_destination(0, 5)` then `request_pickup(3, "DOWN")`. The elevator passes floor 3 without stopping, opens at 5, then returns: **5, 3**.

**4. Requests added mid-journey** (1 elevator): `request_destination(0, 5)`, two steps (now at floor 2). Then `request_destination(0, 3)` is ahead of the elevator -> stops are **3, 5**. Instead `request_destination(0, 1)` is behind it -> stops are **5, 1**.

**5. Choosing among elevators** (12 floors, 2 elevators). Elevator 0 gets `request_destination(0, 8)` and moves 3 steps (floor 3, still heading up).
- `request_pickup(5, "UP")`; two steps later elevator 0 is at floor 5 with doors open and elevator 1 is still at floor 0 (heading toward the call beats a farther idle elevator).
- Alternatively `request_pickup(2, "UP")`; two steps later elevator 1 is at floor 2 with doors open and elevator 0 is at floor 5 (elevator 0 already passed floor 2, so the idle elevator 1 is preferred).

**6. Rejections** (10 floors, 2 elevators): `request_pickup(10, "UP")`, `request_pickup(-1, "UP")`, `request_pickup(3, "SIDEWAYS")`, `request_pickup(0, "DOWN")`, `request_pickup(9, "UP")`, `request_destination(2, 4)`, `request_destination(0, 99)` all return `false`, and 20 further steps leave both elevators at floor 0 with doors closed.

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

- `request_pickup(floor, direction)`: direction is exactly `"UP"` or `"DOWN"`. Returns `true` if accepted, `false` if invalid.
- `request_destination(elevator, floor)`: elevators are numbered from 0. Returns `true` if accepted, `false` if invalid.
- `step()` advances every elevator by one tick as described in the requirements.
- `floor_of(elevator)` is the current floor. `doors_open(elevator)` is `true` only during the step in which the doors were opened.
- Observable ordering (single elevator): requests that lie ahead in the direction of travel are served before ones behind it; a hall call in the *opposite* direction to travel is not served on the way.
- With several elevators, a hall call results in exactly one elevator opening its doors for it; an elevator already heading toward the call is preferred over a farther idle one, and a nearby idle elevator is preferred over one that has already passed the floor.
- `floor_of(elevator)` and `doors_open(elevator)` are read-only; reading them never advances time.
- A rejected request leaves every elevator's position, doors and pending stops unchanged.
- Hall-call direction is compared case-sensitively against the exact strings `"UP"` and `"DOWN"`.

## Test entry point (Rust)

The tests drive your system through the thin, primitive-typed contract below (`src/contract.rs`). It is
**not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up
in `src/lib.rs`.

```rust
//! Thin test entry point. Your domain model lives behind it.

pub trait ElevatorApi {
    /// Hall call. `direction` is "UP" or "DOWN". Returns false for invalid requests.
    fn request_pickup(&mut self, floor: i32, direction: &str) -> bool;

    /// Cabin button inside the given elevator (0-based). Returns false for invalid requests.
    fn request_destination(&mut self, elevator: usize, floor: i32) -> bool;

    /// Advances the simulation by one tick.
    fn step(&mut self);

    fn floor_of(&self, elevator: usize) -> i32;

    /// True only during the step in which the doors were opened.
    fn doors_open(&self, elevator: usize) -> bool;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(_floors: u32, _elevators: u32) -> Option<Box<dyn ElevatorApi>> {
    None
}
```

## Language notes

Everything is `&mut self`; there is no shared mutable state to protect. Prefer enums with data for elevator state over trait objects unless behavior genuinely varies.

## Running

```bash
cargo test -p p02_elevator
STRICT=1 cargo test -p p02_elevator   # fail instead of skip while unimplemented
cargo test -p p02_elevator --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p02_elevator --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.
