# Follow-ups — 06 ATM

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Multiple accounts per card, transfers
A card can access checking and savings; the customer can transfer between them within the session.

### V3 — Hardware faults
The cash dispenser can jam after the account was debited. Model the failure and make the books consistent (reversal, audit trail).

### V4 — Daily limits and receipts
The 500 limit becomes a rolling daily limit across sessions and ATMs; every operation produces an auditable receipt.

### V5 — Concurrent ATMs
Two ATMs withdraw from the same account at the same instant. Who wins, and what is the unit of locking?
