# Interviewer notes — 112 Parking Lot with Allocation Strategies

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- `MOST_FREE_FLOOR` over three floors with 2, 4 and 3 spots, filled completely, then three spots freed (one per floor): the next three arrivals must go floor 0, 1, 2 (ties by floor), not "where the freed spot was".
- Switching back from `MOST_FREE_FLOOR` to `LOWEST_INDEX_SPOT` after holes appeared on several floors: holes are filled in global order.
- A lot that is over-subscribed (twice as many arrivals as spots), then fully emptied: counts return to the initial values, and a vehicle that was turned away earlier can park.
- A randomized workload (parks, unparks, strategy switches) compared with a brute-force model for both strategies: any tie-break difference shows up here.
- A big lot (60 floors, 6,000 two-wheeler spots) filled, half emptied and refilled under both strategies within a time budget: an O(spots) scan per `park` plus per-call sorting is too slow.
- 12 threads parking/unparking: no spot is ever held by two live tickets; counts return to the start.
- Strategy changes while 8 threads park: every spot ends up used exactly once.
- A rejected call (duplicate, bad type, blank registration) leaves no half-registered vehicle behind.

## Likely interviewer questions
- Where does the allocation rule live and what does it need to know to decide? Does it mutate anything?
- What is the global index of the first four-wheeler spot of floor 2? (Check they implemented the spec, not their own numbering.)
- What data structure answers "lowest free spot of this type" and "free count of this type per floor" fast, and who keeps them in sync?
- How is a ticket different from a registration number, and which one is the key of what?
- Why can the same registration park again after leaving but not twice at the same time? Where is that checked, and under which lock?
- What does `freeSpots` cost? What does `park` cost under each strategy?

## Invariants
- A spot is either free (in exactly one free structure of its type) or held by exactly one active ticket.
- A registration number maps to at most one active ticket, and every active ticket maps to exactly one registration.
- `freeSpots(type)` equals the sum of `freeSpotsOnFloor(f, type)`, and free + active tickets of a type is the capacity.
- Ticket ids are issued once; a used ticket is never active again.
- The strategy affects only which free spot is chosen, never what is already allocated.

## Common mistakes
- Numbering spots per floor (0.. on every floor) instead of globally, or interleaving the two types.
- Letting a two-wheeler take a four-wheeler spot when its own are full.
- `MOST_FREE_FLOOR` counting all free spots on a floor instead of the compatible ones.
- Tie-break by highest floor, or by the first floor encountered in a hash order.
- Choosing a floor and then finding it has no compatible spot (when the maximum is 0), instead of returning empty.
- Registering the registration number before the spot was found, leaving a ghost vehicle after an empty result.
- Reusing ticket ids (a counter that is reset, or an id derived from the spot index) so a stale ticket frees another vehicle's spot.
- Check-then-act on the duplicate registration check outside the allocation lock.
- Returning the freed spot to a list at the end instead of by index, so the next arrival does not get the lowest.
- Scanning every spot of every floor on each park (fine for the tests, not for the large-lot scenario).

## Over-engineering traps
- A strategy factory, a registry and a plugin loader for two strategies.
- A class per spot, per floor and per vehicle with inheritance, when a count and an index would do.
- Events, observers and persistence nobody asked for.

## Design and scaling discussion
- One lock around the whole lot is correct and simple; its cost is a serialized `park`. A lock per floor needs a floor choice that does not hold a global lock: `MOST_FREE_FLOOR` can pick on a slightly stale count and retry, but `LOWEST_INDEX_SPOT` is a global minimum and resists sharding.
- Sorted free sets per type and floor give "lowest free" in O(log n); a heap with lazy deletion is an alternative. The free count per floor is a counter, so `MOST_FREE_FLOOR` is O(floors).
- A strategy is naturally a function from "current free structures and the vehicle type" to a spot; a closed pair of strategies can be an enum, an open set a trait/interface. Make them say why.
- Tickets: a monotonically increasing counter is enough for uniqueness; a random UUID hides volume and avoids guessing.

## Trade-off questions to ask when the candidate is done
- Would you keep the strategy inside the lot or pass it per call? What changes for concurrent callers using different strategies?
- How would you support a spot going out of service (maintenance) without breaking the indices?
- Exact "lowest index" or "any of the lowest few" at 50 floors and many gates: what do you give up and what do you gain?
- What would you log or measure to notice that `MOST_FREE_FLOOR` is leaving the upper floors empty?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
