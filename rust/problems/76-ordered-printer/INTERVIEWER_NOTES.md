# Interviewer notes — 76 Ordered Printer (Turn Taking)

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: race, deadlock, starvation, memory visibility, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Single worker: a ring of one must not deadlock itself.
- Worker arrives long before its turn (reverse start order).
- Action throws on every call.
- The same thread acts for several workers (the tests never do this, but the primitive must not assume thread identity).

## Likely interviewer questions
- Why `while` and not `if` around the wait? What is a spurious wake-up?
- `signal` vs `signalAll` here: with `signal` which thread can get woken, and why can the system hang?
- Is it OK to run the action while holding the lock? What does that do to throughput and to actions that block?
- One condition for everybody or one per worker? What does each cost under 100 workers?
- What guarantees that the next action sees the previous action's writes (happens-before)?

## Invariants
- `0 <= turn < workers` always; `turn` changes exactly once per executed (or failed) action.
- At most one action is running at any instant.
- The sequence of executed actions is exactly `0,1,..,n-1,0,1,...`.

## Common mistakes
- `if` instead of `while` around the wait.
- `notify` / `signal` waking a worker that is not next, and nobody waking the right one.
- Handing the turn over before the action ran, or not at all when the action throws.
- Busy-waiting on a non-volatile / non-atomic field (works by luck, then hangs after JIT).
- Updating `turn` outside the lock, losing the wake-up.

## Over-engineering traps
- A `Worker` class hierarchy or a `TurnStrategy` for a one-method primitive.
- Queues of waiting threads when a single integer suffices.

## Concurrency discussion
- Condition variable on one lock is the textbook answer. `signalAll` costs O(N) wake-ups per turn; per-worker semaphores or `LockSupport.park/unpark` make it O(1) but make exception safety and ownership harder.
- Memory visibility comes for free from the lock hand-off; with per-worker semaphores it comes from the semaphore release/acquire pair.

## Trade-off questions to ask when the candidate is done
- Why this lock, and why one condition? What would per-worker conditions change?
- What happens with 10,000 workers? How would you avoid the thundering herd?
- How would you test it? (How do you prove the absence of a lost wake-up?) What would make a hang visible instead of silent?
- Make it lock-free: what do you spin on, and why is that a bad idea for a turn-taking primitive?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.
