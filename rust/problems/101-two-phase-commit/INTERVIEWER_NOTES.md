# Interviewer notes — 101 Two-Phase Commit

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: safety violation (split brain, lost or divergent committed data), liveness problem (no progress), missed failure case (crash, loss, duplicate, reorder, partition), non-determinism, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- A duplicated PREPARE after the participant voted YES must answer YES again and must not lock twice or conflict with itself.
- A late PREPARE arriving after the DECISION (ABORT) for that transaction must be refused, not lock the keys.
- A YES vote arriving after the coordinator already timed out and aborted.
- The coordinator crashes between logging the decision and sending it; or before logging it (then the restart must abort).
- Coordinator restarted on an empty log: an INQUIRE must get ABORT, `outcome` of the old id must be an error.
- A participant restarted from its log with PREPARED but no decision: locks and in-doubt set come back, and the INQUIRE loop resumes.
- Duplicate DECISION: apply once, acknowledge again.
- 50 transactions on the same hot key at 5 ms spacing: at most one active holder at a time and both participants end with the same final value.

## Likely interviewer questions
- Where is the commit point, and what can the coordinator do after it that it could not do before?
- Why must the participant log PREPARED before sending YES? What breaks if it logs after?
- Why can a participant not simply time out an in-doubt transaction? What is the cost of blocking?
- What exactly does presumed abort save, and why is it safe for the coordinator to forget aborted transactions?
- How do you make every handler idempotent? Which messages can be duplicated or reordered?
- What happens if the coordinator's disk loses the last record?

## Invariants
- Atomicity: for every transaction, all participants that applied it applied the same decision; nobody applies a COMMIT unless all voted YES.
- A participant that voted YES does not change its mind until it has learned the decision.
- A decision never changes once logged.
- Locks are held only while a transaction is prepared and undecided; after healing no participant is left in doubt.
- A transaction containing a `REJECT` value never commits.

## Common mistakes
- Sending the decision before logging it (a crash then loses a decision participants already acted on).
- Answering YES before the PREPARED record is durable.
- Releasing locks on timeout while still in doubt, or applying writes on a PREPARE.
- Not re-sending decisions / PREPAREs, so a lost message hangs a transaction forever.
- Restarting the coordinator without re-announcing decided transactions (participants block until their INQUIRE gets through - only works if INQUIRE is also implemented).
- Using the in-memory state after a restart (the new instance must start only from the log).
- Letting a late PREPARE lock keys for a transaction that has already been aborted.

## Over-engineering traps
- A generic transaction-manager framework with pluggable protocols before 2PC passes.
- Implementing Paxos Commit or 3PC because blocking 'looks wrong' - the tests require that 2PC blocks.
- A custom binary log format or a write-ahead-log library instead of a string record per event.

## Distributed-systems discussion
- Google Spanner runs 2PC across Paxos groups, so the coordinator and participants are replicated and the blocking window disappears in practice; MySQL XA, PostgreSQL `PREPARE TRANSACTION` and Java JTA expose the same protocol, with an operator-resolved in-doubt state.
- 2PC is CP and blocking: it needs all participants to commit and gives up availability when the coordinator and a participant fail together. Skeen's 3PC avoids blocking only under a no-partition assumption; FLP-style results say no asynchronous protocol both terminates and stays safe under all failures.
- Sagas (problem 102) trade isolation for availability: no locks across services, but intermediate states are visible and compensation can fail.
- Simplified here: one coordinator, string payloads, fixed timeouts, no read isolation, no heuristic decisions, perfect durable log, no log truncation / checkpointing.

## Trade-off questions to ask when the candidate is done
- 2PC vs saga: what does each give up, and which one for a bank transfer vs an order-fulfilment flow?
- How would you shorten the in-doubt window? (replicated coordinator, cooperative termination, early lock release)
- What is the lock granularity trade-off, and how would you avoid deadlock between two transactions locking keys in different orders?
- Presumed abort vs presumed commit vs presumed nothing: which log writes does each save?
- How do you test crash-recovery at every possible log point?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.
