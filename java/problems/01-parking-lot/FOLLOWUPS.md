# Follow-ups — 01 Parking Lot

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Multiple floors with different layouts
Floors no longer share the same spot mix; some floors are closed for maintenance (no new allocation, existing vehicles may leave). Allocation preference may now depend on more than size and floor index.

### V3 — Multiple gates
Several entry gates and exit booths operate independently; a gate can be assigned a *preferred floor*. How does ticketing and allocation scale across gates without a global bottleneck?

### V4 — Concurrent entry/exit
Hundreds of vehicles arrive at once. Prove (with tests) that no spot is double-allocated and that the garage never "loses" capacity. Where do you lock, and how coarse?

### V5 — Dynamic pricing
Rates depend on time of day, day of week, occupancy, and an optional validated discount. Rate rules change without redeploying and are applied to *past* intervals correctly (a stay that spans a peak boundary).

### V6 (stretch) — Reservations and EV charging
A spot can be pre-booked for a window, and some spots have chargers billed per kWh.
