# Interviewer notes — 16 Digital Wallet

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- Transfer to the same wallet.
- Validation order: an invalid amount wins over an unknown wallet; a duplicate wins over everything.
- A refused operation does not burn its key.
- A transfer replayed with the same key moves money once.
- Statement is a snapshot, not a live view.

## Likely interviewer questions
- Where does the idempotency check happen relative to the balance check, and why must "check key + apply + record key" be one atomic step?
- How do you lock for a transfer? What is the deadlock risk and how do you remove it (ordering by id, single lock, ...)?
- Is the key store global or per wallet? What breaks if it is per wallet and the same key is replayed against another wallet?
- What do you store for a duplicate: just the key, or the original result too (so the retry can return the same answer)?
- How do you guarantee that the sum of all balances is invariant under any interleaving?

## Invariants
- Σ balances = Σ applied credits − Σ applied debits.
- No balance is ever negative.
- Statement entries of a wallet sum to its balance.
- A key appears in at most one applied operation.

## Common mistakes
- Check-then-act on the balance without holding the lock across both steps.
- Locking `from` then `to` in call order (classic deadlock on opposite transfers).
- Recording the key before the operation succeeds (burns the key on refusal) or after releasing the lock (two threads both apply).
- Debiting the source, then discovering the destination is unknown, and not rolling back.
- Per-wallet key sets (replay against another wallet applies twice).
- Treating `transfer(a, a, x)` as a no-op success.

## Over-engineering traps
- `Account`, `Ledger`, `Transaction`, `TransactionType`, `IdempotencyService`, `WalletRepository` interfaces and a command pattern in V1.
- Event sourcing before it is asked for.

## Concurrency discussion
- Global lock: trivially correct, serialises everything; fine until measured.
- Per-wallet locks acquired in a global order (e.g. lexicographic id) — no deadlock, parallel for disjoint pairs.
- The idempotency set needs its own atomic `putIfAbsent` or must be guarded by the same locks; reserving the key before the balance check is a trap.

## Trade-offs worth probing
- Lock striping vs per-wallet locks; lock ordering vs `tryLock` with back-off.
- Compare-and-swap on a balance works for credit/debit but not for a two-wallet transfer.

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.
