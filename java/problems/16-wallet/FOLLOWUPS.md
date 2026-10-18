# Follow-ups — 16 Digital Wallet

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Holds and two-phase payments
Reserve funds now, capture or release later (card authorisations). Available vs. ledger balance. What do idempotency keys mean for capture?

### V3 — Key expiry and conflicting replays
Keys expire after 24 hours; a replay with *different* parameters must be reported as a conflict instead of a plain duplicate. What do you store per key?

### V4 — Multiple currencies
Wallets per currency, conversion transfers with rates; how do you keep conservation checkable?

### V5 — Audit and reconciliation
Immutable ledger with double-entry postings; rebuild every balance from the ledger and prove it matches.
