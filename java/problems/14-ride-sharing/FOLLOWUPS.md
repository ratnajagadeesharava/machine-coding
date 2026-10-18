# Follow-ups — 14 Ride Sharing

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Vehicle types and surge pricing
Economy / XL / premium vehicles; the fare multiplier rises with demand-to-supply ratio in the pickup zone. Prices must be quoted before the ride and honoured.

### V3 — Driver acceptance
Matched drivers may decline within 10 seconds; the request cascades to the next nearest driver. What does "matched" mean now?

### V4 — Ride pooling
Two riders heading in the same direction share a car; fares are split; the route can change as riders join.

### V5 — Scale
Millions of drivers moving constantly. How do you index locations (grid cells, geohash, quadtree) and keep matching consistent when two riders compete for one driver?
