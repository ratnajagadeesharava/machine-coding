# Follow-ups — 112 Parking Lot with Allocation Strategies

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Fees
Charge on exit: the first hour costs a flat amount, every further started hour costs less, per vehicle type, with a daily maximum. `unpark` now returns the amount. Where does the clock come from so that the rule is testable, and what happens to a lost ticket?

### V3 — Reservations
A driver can reserve a spot of a type for a time window before arriving. A reserved spot must not be given to other vehicles during its window but is free outside it. How does that interact with each strategy, and what does `freeSpots` mean now?

### V4 — A new strategy
Add `NEAREST_TO_EXIT` (each floor has a distance to an exit) and a strategy that fills floors one at a time (`FILL_FLOOR_FIRST`). How many places change? Can the operator register a strategy at runtime, and who decides the tie-breaks?

### V5 — Vehicle classes and spot sharing
Add a third type (bus, three spots long) and allow a two-wheeler to use a free four-wheeler spot as a fallback when its own spots are full (never the other way round). What is the compatibility rule, how is it stated in code, and how do the strategies and the free-spot counts change?

### V6 — Multiple entrances and scale
Several gates call `park` at the same time, and the lot has 50 floors with thousands of spots. Where is the contention? What do you shard, and what do you give up (for example exact "lowest index")?
