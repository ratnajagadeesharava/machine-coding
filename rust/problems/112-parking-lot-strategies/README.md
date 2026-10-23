# 112 · Parking Lot with Allocation Strategies (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 75 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

A multi-storey car park. Every floor has a fixed number of spots for two-wheelers and a fixed number for four-wheelers. A driver arrives, the lot picks a spot and hands out a ticket; later the ticket is handed back and the spot becomes available again. Operators can also ask where a vehicle is and how many spots are free. The interesting part is **where** a vehicle goes: the lot supports two allocation strategies that give different, fully predictable answers, and the operator can switch between them while the lot is in use.

## Problem

Build the core of a parking lot for a mall with several floors. Each floor has its own count of two-wheeler spots and four-wheeler spots (a floor may have none of one kind). A two-wheeler only ever uses a two-wheeler spot and a four-wheeler only a four-wheeler spot. When a vehicle arrives, a strategy decides which free spot it gets. The lot starts with one strategy and the operator may change it at any time; the change affects only vehicles that arrive afterwards. Every choice must be deterministic: given the same history, the same spot is chosen, every time.

## Requirements

1. The lot is created with `n >= 1` floors, the number of two-wheeler spots and the number of four-wheeler spots on each floor (each `>= 0`), and a starting strategy. Anything else is an error.
2. **Spot numbering (global).** Floors are numbered `0..n-1`. Spots are numbered globally from `0`, floor by floor: floor 0 owns the first block of indices, floor 1 the next block, and so on. Inside a floor, the two-wheeler spots come first and the four-wheeler spots next, each in consecutive indices. A spot is reported as `"floor:spot"` using the floor number and the global spot index.
3. `park(vehicleType, registrationNumber)` allocates a free spot of the vehicle's own type, chosen by the current strategy, and returns a new ticket id. If no compatible spot is free it returns empty and changes nothing.
4. A registration number that is currently parked cannot park again: that is an error, even if the lot is full or the second attempt uses the other vehicle type. Once the vehicle has left, the same registration number may park again and gets a new ticket.
5. Ticket ids are non-blank, unique among all tickets ever issued by the lot and never reused, even after the ticket was used.
6. `unpark(ticketId)` frees the spot and returns true. An unknown ticket, or a ticket that was already used, returns false and changes nothing. The freed spot is available to the very next `park`.
7. `findByRegistration` and `findByTicket` return the `"floor:spot"` of a vehicle that is currently parked, or empty. After `unpark`, both return empty for that vehicle.
8. `freeSpots(vehicleType)` returns the free spots of that type in the whole lot; `freeSpotsOnFloor(floor, vehicleType)` on one floor.
9. Strategies:
   - `LOWEST_INDEX_SPOT`: the compatible free spot with the lowest global index in the whole lot.
   - `MOST_FREE_FLOOR`: the floor that currently has the most free spots **of the vehicle's type** (ties: lowest floor number); on that floor, the compatible free spot with the lowest index.
10. `setStrategy` changes the strategy for later `park` calls only; parked vehicles do not move. An unknown strategy is an error and leaves the current one in place.
11. Invalid input is an error and changes nothing: an unknown vehicle type, a null/blank registration number or ticket id, a floor that does not exist.

## Constraints

- Many threads park and unpark at once. The lot must never give one spot to two live tickets, never lose a freed spot, and never let two threads park the same registration number.
- A rejected or empty `park` must leave no trace (no half-registered vehicle, no consumed spot).
- Allocation is the hot path; keep it reasonable for lots with thousands of spots and dozens of floors (hidden: 6,000 spots per type).
- No clock and no randomness: the same sequence of calls always yields the same spots.

## Examples

Lot used in the examples: floor 0 has 2 two-wheeler and 1 four-wheeler spots, floor 1 has 1 two-wheeler and 2 four-wheeler spots. Global indices: floor 0 = `0,1` (two-wheeler), `2` (four-wheeler); floor 1 = `3` (two-wheeler), `4,5` (four-wheeler).

```
LOWEST_INDEX_SPOT
park(TWO_WHEELER, "A")   -> ticket, findByTicket -> "0:0"
park(TWO_WHEELER, "B")   -> "0:1"
park(TWO_WHEELER, "C")   -> "1:3"
park(TWO_WHEELER, "D")   -> empty             (no two-wheeler spot left, four-wheeler spots are not used)
unpark(ticket of A)      -> true
park(TWO_WHEELER, "D")   -> "0:0"             (the freed spot is the lowest again)

MOST_FREE_FLOOR
park(FOUR_WHEELER, "A")  -> "1:4"             (floor 1 has 2 free four-wheeler spots, floor 0 has 1)
park(FOUR_WHEELER, "B")  -> "0:2"             (1 vs 1: tie, lowest floor wins)
park(FOUR_WHEELER, "C")  -> "1:5"
park(FOUR_WHEELER, "D")  -> empty

Mixed
park(FOUR_WHEELER, "A")  -> "0:2" under LOWEST_INDEX_SPOT
setStrategy(MOST_FREE_FLOOR)
park(FOUR_WHEELER, "B")  -> "1:4"             (A stays at 0:2; floor 0 has 0 free, floor 1 has 2)
park(FOUR_WHEELER, "A")  -> error             (A is still parked)

Numbering with uneven floors: two-wheeler counts [0,3,1], four-wheeler counts [2,0,2]
floor 0 = 0,1 (four) | floor 1 = 2,3,4 (two) | floor 2 = 5 (two), 6,7 (four)
```

### Edge cases

- A two-wheeler never takes a four-wheeler spot (and the other way round), even if the lot is otherwise empty.
- `MOST_FREE_FLOOR` counts only free spots of the arriving vehicle's type; free spots of the other type do not influence the choice.
- A floor with zero spots of a type can never be chosen for that type, even when all other floors are full.
- A registration number is compared exactly (case-sensitive, no trimming). `"ka-01"` and `"KA-01"` are different vehicles.
- A vehicle turned away (empty result) is not registered: it may try again later and no error is raised for it.
- A duplicate registration is reported as a duplicate even when the lot is full.
- A ticket id that was used cannot release a spot that has since been given to another vehicle.
- Lot with no spots of one type is valid; parking that type always returns empty.

## Glossary

- **Spot**: one parking place of a fixed type on one floor, identified by its global index.
- **Ticket**: the receipt returned by `park`; it is the only handle for `unpark`.
- **Strategy**: the rule that picks the spot for the next arriving vehicle.
- **Compatible spot**: a spot of the same type as the vehicle.
- **Global index**: the lot-wide spot number defined in Requirement 2.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Vehicle types are the strings `TWO_WHEELER` and `FOUR_WHEELER`; strategies are `LOWEST_INDEX_SPOT` and `MOST_FREE_FLOOR`.
- Spot descriptions are `"floor:spot"` with the global index, for example `"1:4"`.
- Rust: `Result<_, ParkingError>` with `InvalidArgument` (unknown vehicle type or strategy, blank registration number or ticket id, a floor that does not exist, invalid lot configuration in `create`) and `AlreadyParked` (registration number currently parked).
- `park` returns `Result<Option<String>, _>` (the ticket); `unpark` returns `Result<bool, _>`; `find_*` return `Result<Option<String>, _>`; counts are `usize`.
- Ticket ids are opaque non-blank strings; the tests never rely on their format, only on uniqueness and never being reused.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

#[derive(Debug, PartialEq, Eq)]
pub enum ParkingError {
    /// unknown vehicle type / strategy, blank registration or ticket id, floor out of range,
    /// invalid lot configuration
    InvalidArgument,
    /// a vehicle with this registration number is currently parked
    AlreadyParked,
}

pub trait ParkingLot: Send + Sync {
    /// Ticket id of the new parking, `Ok(None)` if no compatible spot is free.
    fn park(&self, vehicle_type: &str, registration: &str) -> Result<Option<String>, ParkingError>;
    /// `Ok(true)` if the ticket was active; `Ok(false)` for an unknown or already used ticket.
    fn unpark(&self, ticket_id: &str) -> Result<bool, ParkingError>;
    /// "floor:spot" of a parked vehicle.
    fn find_by_registration(&self, registration: &str) -> Result<Option<String>, ParkingError>;
    /// "floor:spot" of an active ticket.
    fn find_by_ticket(&self, ticket_id: &str) -> Result<Option<String>, ParkingError>;
    fn free_spots(&self, vehicle_type: &str) -> Result<usize, ParkingError>;
    fn free_spots_on_floor(&self, floor: usize, vehicle_type: &str) -> Result<usize, ParkingError>;
    /// Strategy for later park calls; parked vehicles do not move.
    fn set_strategy(&self, strategy: &str) -> Result<(), ParkingError>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(
    two_wheeler_spots: &[usize],
    four_wheeler_spots: &[usize],
    strategy: &str,
) -> Option<Result<Arc<dyn ParkingLot>, ParkingError>> { ... }
```

`two_wheeler_spots[f]` and `four_wheeler_spots[f]` are the spot counts on floor `f`. Return `None` until implemented; an invalid configuration is `Some(Err(InvalidArgument))`.

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Nothing depends on the wall clock or on sleeping. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Placement under both strategies, including exact `floor:spot` answers, tie-breaking and freed-spot reuse.
- Type separation, a full lot, a lot with no spots of one type, the global numbering with uneven floors.
- Duplicate registrations, re-parking after leaving, unknown and already used tickets, unique ticket ids.
- Free-spot counts in total and per floor; switching strategies.
- Invalid input and invalid lot configuration.
- Concurrency: 64 threads parking into 30 spots (exactly 30 succeed, no spot shared), one registration parked by 16 threads (exactly one wins), parking and leaving churn that must restore the counts.
- More (a randomized cross-check against a brute-force model, large lots, strategy switches under load) in `interviewer/`.

## Running

```bash
cargo test -p p112_parking_lot_strategies
STRICT=1 cargo test -p p112_parking_lot_strategies   # fail instead of skip while unimplemented
cargo test -p p112_parking_lot_strategies --features hidden   # interviewer scenarios
cargo play run 112              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`run_concurrently`, `sut!`): `rust/support`.

