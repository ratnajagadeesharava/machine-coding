# 01 · Parking Lot (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 90 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

**What you are building.** A multi-storey car park needs the software behind its entrance and exit barriers. When a vehicle arrives, the system chooses a free parking spot for it and hands back a **ticket**. When the driver leaves, they present the ticket, the spot is freed and the system returns the **fee** for the time spent. Gate staff and a "spaces left" display also ask how many spots could currently take a given kind of vehicle. Design and build the system that does this, correctly even when several gates operate at the same time.

## Requirements

1. The car park has one or more floors, numbered from 0 (lowest). Every floor has the same number of **small**, **medium** and **large** spots (any of the three counts may be 0).
2. Vehicles are motorcycles, cars or trucks. A motorcycle fits any spot, a car fits a medium or large spot, a truck fits only a large spot.
3. When a vehicle enters, the system picks a spot for it and returns a ticket. If nothing fits, entry is refused and nothing changes.
4. Spots are allocated so that the **smallest sufficient spot size** is used first; among spots of that size, the **lowest floor** wins.
5. A vehicle (identified by plate) can be parked at most once at a time. Once it has exited, the same plate may enter again.
6. On exit the ticket is presented, the spot becomes available again, and the fee is returned.
7. Fees are charged per started hour with a minimum of one hour. Hourly rates: motorcycle 1000, car 2000, truck 5000 (smallest currency unit).
8. A ticket can be used to exit exactly once.
9. Callers can ask how many spots could currently accept a given vehicle type.

## Constraints

- Up to 100 floors and 10,000 spots; 100,000 enter/exit operations should complete in a few seconds.
- Time comes from the `Clock` handed to your factory; never read the wall clock directly.
- The first version may be single-threaded, but the contract tests include concurrent entry/exit — the end state must be consistent (no double allocation, no lost spots).

## Examples

Notation: `S/M/L` = small/medium/large spots per floor. Time starts at 0 unless stated.

**1. Spot choice and fee** (1 floor, 1 S, 1 M, 1 L)

```
enter(MOTORCYCLE, "M-1")  -> ticket T1   (goes into the small spot)
enter(CAR, "C-1")         -> ticket T2   (medium)
enter(CAR, "C-2")         -> ticket T3   (large; medium is taken)
enter(CAR, "C-3")         -> `None`      (no free spot a car fits)
available_spots(MOTORCYCLE) = 0
after 90 minutes: exit(T2) -> fee 4000   (2 started hours x 2000)
available_spots(CAR)        = 1           (the medium spot is free again)
```

**2. Lowest floor wins among equal sizes** (3 floors, 0 S, 1 M, 0 L per floor)

```
enter(CAR, "A") -> floor_of = 0
enter(CAR, "B") -> floor_of = 1
enter(CAR, "C") -> floor_of = 2
```

**3. Fee boundaries** (1 floor, 0 S, 2 M, 0 L; both cars enter at time 0)

| Exit after | Started hours | Fee for a car |
|---|---|---|
| 0 min | 1 (minimum) | 2000 |
| 60 min exactly | 1 | 2000 |
| 61 min | 2 | 4000 |

**4. Refusals** (1 floor, 0 S, 3 M, 0 L)

```
enter(CAR, "SAME") -> ticket         available_spots(CAR) = 2
enter(CAR, "SAME") -> `None`         (plate already parked; available_spots(CAR) still 2)
enter(HOVERCRAFT, "H-1") -> `None`   (unknown type)
exit("no-such-ticket") -> `None`
exit(ticket); exit(ticket) -> fee, then `None`   (second exit frees nothing)
```

## Edge cases

- A lot with no large spots refuses every truck and reports `available_spots(TRUCK) = 0`.
- A refused entry (full, duplicate plate, unknown type) must not use up a spot or change any later answer.
- A ticket that has been used for exit no longer has a floor: `floor_of` returns `None`.
- Under concurrent entry, with 10 large spots and 50 simultaneous truck arrivals exactly 10 tickets are issued, all distinct, and none shares a spot.

## Glossary

- **Ticket**: the opaque string returned by `enter`; it identifies one stay and is the only thing needed to exit.
- **Plate**: the vehicle's registration string; identifies a vehicle for the "parked at most once" rule.
- **Started hour**: any part of an hour counts as a whole hour, so 61 minutes is 2 hours.
- **Smallest currency unit**: fees are whole integers (e.g. cents), never fractions.

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- Vehicle types are the exact strings `MOTORCYCLE`, `CAR`, `TRUCK`. Any other type is refused on entry.
- `enter` returns a non-empty, unique ticket id, or `None` when: no fitting spot is free, the plate is already parked, or the type is unknown.
- `exit` returns the fee, or `None` when the ticket is unknown or already used.
- Fee = `max(1, ceil(elapsed / 1 hour)) × hourly_rate`. Exactly 60 minutes → 1 hour; 61 minutes → 2 hours; immediate exit → 1 hour.
- `available_spots(type)` = number of currently free spots that could accept that type (spots are counted per fitting size, not reserved). Unknown type → 0.
- `floor_of(ticket)` is the 0-based floor of the spot held by an active ticket; `None` for unknown/used tickets.
- Spot choice: smallest fitting size first, then lowest floor.
- `available_spots` counts a free spot once per type it could accept: with 1 free small, 1 free medium and 1 free large spot the answers are MOTORCYCLE 3, CAR 2, TRUCK 1. These are not additive totals.
- Elapsed time for the fee is `exit time - enter time` as read from the supplied `Clock`.
- Total spots = `floors x (small + medium + large)`; the plate rule applies across the whole lot, not per floor.

## Test entry point (Rust)

The tests drive your system through the thin, primitive-typed contract below (`src/contract.rs`). It is
**not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up
in `src/lib.rs`.

```rust
//! Thin test entry point. Your domain model lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

pub trait ParkingLotApi: Send + Sync {
    /// Parks the vehicle. Returns a ticket id, or `None` if refused.
    fn enter(&self, vehicle_type: &str, plate: &str) -> Option<String>;

    /// Releases the spot of the ticket and returns the fee, or `None` if the ticket is unknown/already used.
    fn exit(&self, ticket_id: &str) -> Option<u64>;

    /// Number of currently free spots that could accept this vehicle type.
    fn available_spots(&self, vehicle_type: &str) -> u32;

    /// 0-based floor of the spot held by an active ticket.
    fn floor_of(&self, ticket_id: &str) -> Option<u32>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(
    _floors: u32,
    _small_per_floor: u32,
    _medium_per_floor: u32,
    _large_per_floor: u32,
    _clock: Arc<dyn Clock>,
) -> Option<Arc<dyn ParkingLotApi>> {
    None
}
```

## Language notes

The trait is `&self` + `Send + Sync` because the concurrency tests call it from many threads. Decide whether one `Mutex`, per-pool locks, or atomics fit your design. `Clock::now_millis` is your time source.

## Running

```bash
cargo test -p p01_parking_lot
STRICT=1 cargo test -p p01_parking_lot   # fail instead of skip while unimplemented
cargo test -p p01_parking_lot --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p01_parking_lot --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.
