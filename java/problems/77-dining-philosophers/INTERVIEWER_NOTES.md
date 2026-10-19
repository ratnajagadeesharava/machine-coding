# Interviewer notes — 77 Dining Philosophers

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: race, deadlock, starvation, memory visibility, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Two seats: both philosophers share the same pair of forks.
- One philosopher far ahead of the others (greedy neighbours).
- A philosopher that never calls `dine` again must not block the others.
- `eat` throws while holding both forks.

## Likely interviewer questions
- Name the four Coffman conditions. Which one does your solution break?
- Why does the resource-ordering rule work for a circular table?
- Does your solution guarantee no starvation, or only deadlock freedom? What is the difference?
- What is the maximum parallelism your solution allows? Is it the theoretical `floor(n/2)`?
- How does the answer change with `tryLock` + back-off?

## Invariants
- Each fork is held by at most one philosopher.
- A philosopher eats only while holding both forks.
- Lock order is a total order (global fork numbering).

## Common mistakes
- Left-then-right for everybody (deadlock appears under load, not in a unit test of 10 meals).
- A global lock around `eat` (correct but zero concurrency).
- Releasing the first fork while waiting for the second ('polite' livelock).
- Forgetting to release in `finally` / on panic.
- Special-casing n=2 with a different code path that is untested.

## Over-engineering traps
- A full `Philosopher` thread class with its own loop - the tests drive the threads.
- Implementing a general resource-allocation graph.

## Concurrency discussion
- Ordering: simple, deadlock-free, can starve in theory. Arbitrator: simple, limits concurrency to n-1 diners but still prevents the cycle. Chandy-Misra: distributed, fair, more state.
- Fairness in practice depends on the lock (fair vs unfair locks); say what yours is.

## Trade-off questions to ask when the candidate is done
- Why this solution and not the arbitrator? What do you give up?
- What happens with 10,000 philosophers on 8 cores?
- How do you test for deadlock freedom? How do you test for starvation (it is a liveness property)?
- How would you build it lock-free?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.
