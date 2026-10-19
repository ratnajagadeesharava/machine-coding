# Follow-ups — 75 Bank Transfers Without Deadlock

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Deposit, withdraw, history
Add `deposit`/`withdraw` and a per-account transaction history. Which lock protects the history, and does `total()` still hold?

### V3 — Millions of accounts
A lock object per account is fine for 1,000 accounts. What do you do for 50 million? (lock striping, hash of the account id) What does striping do to your lock-ordering argument?

### V4 — tryLock and backoff
Replace lock ordering by `tryLock` with back-off. When is that better, and what new problem (livelock) appears?

### V5 — Optimistic / lock-free
Use versioned balances and compare-and-swap. What happens to the atomicity of a *two*-account update?
