# Follow-ups — 87 Expense Groups (Splitwise)

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Unequal split types
Add percentage and share-based splits (`ann 50%, bob 30%, cy 20%`, or `2 shares / 1 share`) with the same rounding rule: totals always add up and the result is deterministic. Where does the split strategy plug in without touching balances?

### V3 — Multi-currency
Expenses are in different currencies with exchange rates at the date of the expense. Balances are kept per currency or converted? What happens to rounding when the rate changes, and what does `settleUp` mean now?

### V4 — Edit and delete expenses
Expenses become first-class records that can be edited or deleted, and `balances` must reflect that. Do you recompute balances from the ledger or keep them incrementally and apply inverse entries? What about settlements that depended on the old numbers?

### V5 — Simplified debts across groups
A user belongs to several groups. Compute a minimal set of payments across all groups the user shares with each friend. Why is the per-group optimum not the global optimum, and what changes in the algorithm's input?

### V6 — Scale and audit
A million groups, each small, with an append-only ledger and idempotent expense creation (retry-safe with a client-supplied request id). Where is the per-group lock, and how do you serve `balances` quickly without replaying the ledger?
