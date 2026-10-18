# Follow-ups — 15 Splitwise

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Groups and currencies
Expenses belong to groups; per-group balances; amounts in different currencies with conversion at the time of the expense.

### V3 — Edit and delete
Expenses can be edited or deleted; balances must be corrected without replaying everything. What do you store?

### V4 — Shares and itemized bills
Split by shares ("2 nights vs 1 night"), by itemized lines with tax and tip allocated proportionally.

### V5 — Minimal cash flow
`simplify()` should produce the *minimum* number of payments. (It is NP-hard in general — what do you do?)
