# Interviewer notes — 06 ATM

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- 2 wrong PINs, 1 correct, 2 wrong → **not** locked; a third wrong in a row locks.
- A locked card stays locked even with the right PIN.
- Refused withdrawals (cash shortage, limit) leave balance, cash and session limit untouched.
- Limit boundary: exactly 500 allowed; 510 total refused.
- Fewest-notes under limited stock needs search/DP, not plain greedy.

## Likely interviewer questions
- Where is the PIN check, who owns the failure counter, and what does a "session" own?
- How do you make the withdrawal atomic? What if dispensing fails after the debit?
- How do you choose notes? Prove (or disprove) greedy correctness; what about bounded stock?
- What are the states of an ATM, and do they matter for your design?

## Invariants
- Account balance ≥ 0; cash per denomination ≥ 0.
- `Δ(account balance) = −(Σ dispensed notes)` on every successful withdrawal.
- Locked cards never get a session.
- A session's withdrawn total ≤ 500.

## Common mistakes
- Greedy note selection that returns "cannot pay" when a valid combination exists.
- Debiting the account before checking the machine can pay (leaving money lost on refusal).
- Counting failed withdrawals against the session limit.
- Not resetting the failure counter on success; locking at 4 instead of 3.
- Storing the PIN in plain text without comment (discuss hashing even if out of scope).

## Over-engineering traps
- A full ATM State pattern (`IdleState`, `HasCardState`, `PinEnteredState`...) before the contract needs it. Sessions as tokens already carry the state.
- `CashDispenser` chain-of-responsibility for 4 denominations.
- Interfaces for `Account`, `Card`, `Bank`, `Screen`, `Keypad`.

## Scalability / concurrency discussion
- Per-account locking, idempotency keys for withdrawals, two-phase withdrawal (reserve → dispense → commit/reverse).

## Trade-offs worth probing
- Bounded coin-change DP vs backtracking with pruning; denominations are tiny.
- Tokens vs session objects returned to the caller.

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.
