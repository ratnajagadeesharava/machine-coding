# Interviewer notes — 75 Bank Transfers Without Deadlock

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: race, deadlock, starvation, memory visibility, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- `from == to` must not try to lock the same non-reentrant lock twice (Rust deadlocks itself, Java does not with a reentrant lock but must still reject).
- Insufficient funds must leave both balances untouched.
- An account opened while `total()` is iterating.
- Overflowing the id ordering (`a - b` style comparators).

## Likely interviewer questions
- How do you prove the ordering rule prevents deadlock?
- What does `total()` cost, and can it block all transfers? Is that acceptable?
- Why is checking the balance *before* taking the lock a bug?
- Single global lock vs per-account locks: when is the global lock actually faster?
- What would you do if account ids are not comparable (e.g. remote accounts)?

## Invariants
- Sum of balances is constant across `transfer`.
- `balance >= 0` at all times.
- Lock acquisition order is a total order on accounts.

## Common mistakes
- Locking `from` then `to` (the classic deadlock).
- Comparing names or ids with subtraction (overflow) or using hash codes that can collide without a tiebreaker.
- Reading the balance, then locking, then writing (check-then-act race).
- `total()` summing without locks: it 'works' until it doesn't.
- Locking the two accounts but releasing the first before updating the second.

## Over-engineering traps
- Command objects, event sourcing, two-phase commit for an in-memory bank.
- A `TransferService` + `AccountRepository` + `LockManager` stack.

## Concurrency discussion
- Lock ordering is the cheapest correct answer. Global lock is correct and simple but serialises everything - quantify it.
- `total()` can lock everything in order (blocks transfers briefly) or use a stamp/versioned snapshot; discuss the cost of consistent reads.

## Trade-off questions to ask when the candidate is done
- Why lock ordering and not `tryLock`? When would you choose `tryLock`?
- What happens under high contention on one hot account? How would you shard a hot balance?
- How would you test deadlock freedom beyond running it many times? (lock-order checker, thread dump on timeout)
- Could this be lock-free? What happens to atomic two-account updates?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.
