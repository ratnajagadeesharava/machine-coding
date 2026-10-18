# Interviewer notes — 14 Ride Sharing

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- Eligibility is `≤ 10`: 10 matches, 11 does not.
- A rider whose ride is completed or cancelled may request again; one whose ride is active may not.
- Driver ties resolve to the smallest id (not insertion order).
- Negative coordinates; pickup == drop (fare 500).
- A busy driver is excluded even if they are the nearest.
- After completion the driver is matched based on the **drop** location.

## Likely interviewer questions
- What are the entities? `Driver`, `Rider`, `Ride`, `Location`? Which of them have identity and state?
- Where does the ride lifecycle live? How do you make illegal transitions impossible (enum + checks, state objects, type-state)?
- How do you find the nearest available driver efficiently? Grid buckets? Spatial index? What does a location update cost?
- How would you make matching atomic when two riders compete (concurrent V5)?
- Why is fare computed on completion, and how would that change with upfront pricing?

## Invariants
- A driver is part of at most one active ride.
- A rider has at most one active ride.
- `status` only moves along allowed edges; terminal states are final.
- A driver's location changes only on completion (in this version).

## Common mistakes
- Not releasing the driver when a matched ride is cancelled.
- Marking the driver available at the pickup (rather than drop) after completion.
- Distance with `Math.sqrt` / Euclidean instead of Manhattan.
- Tie-breaking by hash-map iteration order.
- Allowing `cancel` while in progress.
- Matching a driver that is within range but busy.

## Over-engineering traps
- `RideState` pattern with four classes before the lifecycle justifies it (an enum + a `transition` function can be enough).
- `MatchingStrategy` interface with a single implementation; `LocationService`; observers for every state change.
- Modelling `Rider` as a class that holds nothing but an id.

## Scalability / concurrency discussion
- Spatial grid with cell → set of available drivers; search expanding rings; per-cell locks; two-phase "reserve driver" with timeout.
- Idempotent ride requests and driver state in a CAS-able store.

## Trade-offs worth probing
- Scan vs grid index: complexity vs maintenance cost on every location update.
- Enum lifecycle vs state objects.

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.
