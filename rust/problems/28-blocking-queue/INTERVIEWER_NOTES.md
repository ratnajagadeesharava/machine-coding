# Interviewer notes — 28 Bounded Blocking Buffer

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- Capacity 1: strict ping-pong hand-off, no lost wake-ups.
- `size()` must never be observed above capacity while producers race.
- Poll woken by a put must return promptly, long before its timeout.
- `put` on a full, closed buffer fails immediately (it must not wait for space).
- Several consumers draining a closed buffer get every item exactly once.

## Likely interviewer questions
- One lock and two conditions (not-full, not-empty) versus one condition with `signalAll`? Why is a single condition with `signal` a bug?
- Why `while` instead of `if` around `await`? What are spurious wake-ups?
- How do you implement the timeout without drifting (deadline versus remaining time)?
- What exactly happens to a blocked producer when `close` is called? Who removes it from the wait set?
- Can you do it with semaphores? What does that make awkward (close, size)?
- What do you do on interrupt: restore the flag, rethrow, leave the item in the queue?

## Invariants
- `0 <= size <= capacity` at all times.
- Items leave in the order they entered; each leaves exactly once.
- After `close`, the stored count only decreases.
- No thread waits while the condition it waits for already holds and nobody will signal it again (no lost wake-up).

## Common mistakes
- `if` instead of `while` around the wait (spurious wake-up or stolen item breaks the invariant).
- `signal()` on a shared condition: a consumer wakes another consumer and everyone sleeps forever.
- Forgetting to wake waiters in `close`, or waking them but not re-checking `closed` after waking.
- Re-computing the timeout from scratch after each wake-up (never times out under contention) or ignoring the remaining time.
- Returning "no more items" while items are still stored after `close`.
- Letting a blocked producer add its item after `close` because the check happened before the wait.
- Busy waiting or `Thread.sleep` polling.

## Over-engineering traps
- Node-based linked queue plus custom lock striping before the simple version works.
- Producer/Consumer/Channel class hierarchy; a `QueueState` strategy hierarchy for open/closed.
- Generic observer/metrics layers.

## Concurrency discussion
- Monitor with two condition variables is the classic answer; correctness first. Explain how you avoid lost wake-ups and why `signalAll` on close is required.
- Two-lock queue (head lock and tail lock plus atomic count) improves throughput; harder to get `close` and capacity right.
- Semaphores (permits for free slots and for items) give blocking for free but close/size need extra care.

## Trade-offs worth probing
- Throughput of one lock versus two locks versus lock-free.
- Fairness (FIFO of waiters) versus throughput.
- Failing producers on close versus letting them finish their put.

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.
