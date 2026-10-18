# Interviewer notes — 01 Parking Lot

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- Re-entering with the same plate **after** exit is allowed.
- `floorOf` on a used ticket is empty.
- A motorcycle spills into medium (then large) when smaller spots are full; a car spills into large.
- Fee for exact boundaries (60 min, 61 min, 0 min) and for long stays (25 h).
- A refused entry must not consume a spot or leave the plate "parked".

## Likely interviewer questions
- Why does your allocation structure not scan all spots? What is the cost of `enter`?
- Where does "this spot is taken" live — in the spot, in the floor, in a central index? What breaks if two places hold it?
- How would you add a fourth vehicle type / a new spot size? What code changes?
- What exactly is a "ticket"? Is it an object, an ID, a record in a map? What does exit do with it?
- How do you make fee calculation testable without sleeping?

## Invariants
- A spot is held by at most one active ticket; every active ticket holds exactly one spot.
- `sum(free spots) + sum(active tickets) = total spots`.
- A plate maps to at most one active ticket.
- Refused operations change nothing.

## Common mistakes
- Allocating first, then discovering the plate is a duplicate (and leaking the spot).
- `ceil` bugs at exact hour boundaries; integer division on millis.
- Computing availability by iterating every spot on every query.
- Reading `System.currentTimeMillis()` / `Instant::now` directly.
- Check-then-act on a boolean `isFree` flag with no atomicity.

## Over-engineering traps
- `VehicleFactory`, `SpotFactory`, `ParkingStrategy` interface with one implementation, an `Observer` for "spot freed" nobody uses.
- Deep `Vehicle` hierarchy when vehicle type is only a lookup of (fits sizes, rate).
- A `Gate` class in V1 — wait for V3 to need it.

## Scalability / concurrency discussion
- Per-(size,floor) free pools with a priority structure; lock striping per pool vs one global lock. Compound operations (plate uniqueness + allocation) need a single atomic decision.
- Ticket store: `ConcurrentHashMap` / sharded map. Idempotent exit.
- Sharding by floor across processes; what is the source of truth for plates?

## Trade-offs worth probing
- Pre-computed free lists vs scanning; memory vs speed.
- Enum with data vs polymorphic vehicle classes.
- Pricing as data (table of rates) vs code (strategy) — and when the answer flips.

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.
