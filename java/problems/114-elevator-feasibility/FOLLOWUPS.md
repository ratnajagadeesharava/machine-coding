# Follow-ups — 114 Elevator Request Feasibility

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Cancellations
A rider who is still waiting can cancel. The plan shrinks, so stops disappear and other riders' stop counts drop. What exactly do you recompute, and can a cancellation ever make an already promised ride infeasible? (Think about what happens to the *capacity* check at a stop that no longer exists.)

### V3 — Per-rider limits and priorities
Each request carries its own `maxIntermediateStops`, and a VIP request may be accepted even if it breaks nothing but would be rejected for lack of seats when a non-VIP promise is waiting: the VIP is allowed to bump a *waiting* (never an onboard) non-VIP rider. Which state do you need to roll back if the bump does not make the VIP feasible after all?

### V4 — Two lifts
There are two lifts with the same circuit but different start floors. A request is accepted if either lift can take it; which one do you choose, and does a later request change your earlier choice? Keep promises: an accepted rider is never moved to the other lift unless the move keeps every promise.

### V5 — Smarter lift
Instead of the fixed circuit the lift may reverse where it likes (it turns at the farthest pending floor). Which parts of your feasibility check survive unchanged, and what does the plan model need to become? How do you keep the plan deterministic and testable?

### V6 — Scale
2,000 floors, 5,000 pending riders and 1,000 requests per second. Where does your per-request simulation spend its time? Can you evaluate a new request incrementally, touching only the part of the plan it affects, while keeping the result identical to a full re-simulation?
