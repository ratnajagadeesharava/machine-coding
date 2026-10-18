# Interviewer notes — 03 Library Management

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- 23h59m late → fine 0; exactly 24 h late → 50.
- Returning one of three loans lets the member borrow again.
- Adding more copies of an existing ISBN makes a held/unavailable title available again.
- Reserving an unknown ISBN or by an unknown member is refused.

## Likely interviewer questions
- What is a "copy"? Is there a `BookCopy` object, or just a counter? What changes when you need per-copy barcodes?
- Where does the reservation queue live? What owns "held" state?
- How do you keep `availableCopies` correct when copies are on loan *and* held?
- How would loan limits per membership type be added?

## Invariants
- `onLoan + held + free = total copies` for every ISBN.
- A member never has two active loans of one ISBN, or more than the limit.
- Held copies ≤ queue length; a queue entry exists at most once per (member, ISBN).
- Failed operations leave no trace.

## Common mistakes
- Fine computed with rounding up (late by 1 minute → fined).
- Forgetting that returning a copy must hand it to the *queue* rather than the free pool.
- Letting the head-of-queue member be blocked because the copy counter is zero (counter excludes held copies).
- Allowing a reservation while a copy is free.

## Over-engineering traps
- `Book`, `BookItem`, `Catalog`, `Librarian`, `Account`, `Notification`... all in V1.
- Interface `FineStrategy` with one rule (the follow-ups justify a policy, V1 doesn't).
- Singleton `Library`.

## Scalability discussion
- Inverted index / trie for search; a database with indexes for ISBN, member, due date; time-based jobs for overdue detection.

## Trade-offs worth probing
- Counters vs per-copy entities; when does the second one pay for itself?
- Computing fines at return time vs a periodic accrual job.

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.
