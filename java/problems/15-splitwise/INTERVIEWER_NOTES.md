# Interviewer notes — 15 Splitwise

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- Remainder allocation order (ascending name) for both equal and percent splits.
- Payer not among participants; payer is the only participant (no debt).
- Settling more than owed flips the direction.
- Large number of small expenses: balances stay exact.
- `simplify` on already-zero or two-person situations.

## Likely interviewer questions
- What do you store: raw expenses, a pairwise debt matrix, per-person net balances — or several? Which queries does each serve?
- How do you handle rounding without losing a cent?
- What is the algorithm for `simplify`, and what is its complexity? Is greedy optimal?
- How would you edit/delete an expense (V3) in each storage choice?
- Where do split types live — an enum + `match`, a class per split, a function table? Which one makes adding "by shares" cheapest?

## Invariants
- Σ of all balances = 0 at all times.
- For every expense, Σ shares = amount.
- `owed(a,b) > 0` ⇒ `owed(b,a) == 0`.
- Refused operations change no state.

## Common mistakes
- Floating point for percentages (lost cents).
- Remainder given to the payer or dropped.
- Keeping directed debts without netting (two-way debt reported).
- Mutating balances before validating the split (leaves partial state).
- Mixing "owes" and "is owed" signs.

## Over-engineering traps
- `Expense` hierarchy with `EqualExpense`, `ExactExpense`, `PercentExpense`, `ExpenseFactory`, `SplitStrategy`, `User`, `Group`, `BalanceSheet`, `Settlement` services... A balance ledger plus a split computation per type is the core.
- Observers/notifications in V1.

## Scalability discussion
- Incremental maintenance of net balances (O(participants) per expense) vs storing a log and deriving.
- `simplify`: greedy matching of max creditor with max debtor via heaps — O(n log n), good but not minimal.

## Trade-offs worth probing
- Net balances alone cannot answer `owed(a, b)` pairwise; a pairwise map can. Which does the product need?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.
