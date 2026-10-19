# Interviewer notes — 90 Food Delivery Dispatch

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Stock exactly equal to the quantity ordered; one line short among several (nothing reserved).
- Ready order when a free partner is farther than a busy one: only free partners compete.
- Equal-distance partners, with ids that sort differently from insertion order (`pb` added before `pa`).
- Cancel of a READY order that holds a partner while another order waits: the waiter must get the partner at once.
- Cancel of a waiting order in the middle of the queue; cancel twice; cancel after pickup.
- `pickUp` on a READY order without a partner; `deliver` before `pickUp`.
- Many concurrent cancels and orders: the final stock must equal initial minus the surviving orders.
- Re-adding an item that already has stock (price replaced, stock added).

## Likely interviewer questions
- How do you prevent two orders from being given the same partner when they become READY at the same moment?
- Where does re-dispatch happen, and which events trigger it? What happens if you forget one of them?
- How do you reserve stock for several items atomically? What is the critical section?
- Why is the queue FIFO by `markReady` time and not by `placeOrder` time? Is that a business decision?
- How do you pick the nearest free partner efficiently if there are 100k partners?

## Invariants
- stock >= 0 and stock + reserved by live (non-cancelled) orders == total stock ever added.
- Each partner has at most one active order; an active order has at most one partner.
- An order waits only if no partner was free when it became READY (or when last re-dispatched).
- Order status only follows the allowed transitions; DELIVERED and CANCELLED are terminal.

## Common mistakes
- Decrementing stock line by line and forgetting to roll back on failure.
- Dispatching only in `markReady`, so freed partners never pick up the queue.
- Choosing the nearest partner among all partners, not only the free ones.
- Partner tie-break by insertion order or hash order instead of by id.
- Cancel that restores stock but leaves the partner busy, or leaves the order in the waiting queue.
- Reading `partnerOf` outside the lock so a test sees a half-finished assignment.

## Over-engineering traps
- A strategy-pattern zoo for partner selection when there is exactly one rule.
- A State pattern class per order state with a visitor for what is an enum and a switch.
- Event bus / observer notifications to customers that nobody asked for.

## Design and scaling discussion
- A single lock is simple and correct; per-restaurant locks for stock plus a dispatcher lock for partners scale better but need a lock order (restaurant before dispatcher) and care for cancel.
- Dispatch in production is an optimisation problem (batching, ETAs, partner ratings), run as a periodic matching round rather than greedily per order.
- State changes (order, stock, assignment) belong in one transaction; notifications and payments go through an outbox or saga so retries are idempotent.
- Spatial index for partners (geohash/grid) plus a lock per cell; the free-partner set changes frequently, so keep it separate from slow-changing partner data.

## Trade-off questions to ask when the candidate is done
- Greedy nearest-free assignment versus waiting for a closer partner who is about to finish: when is each better?
- Pessimistic locking of stock versus a conditional update (`stock >= q`) in a database.
- Is FIFO fair for all restaurants? What about priority orders or an SLA deadline?
- How would you test that no partner is double-assigned?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
