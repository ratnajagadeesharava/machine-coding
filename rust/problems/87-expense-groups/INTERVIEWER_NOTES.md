# Interviewer notes — 87 Expense Groups (Splitwise)

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- 100 split between three people in scrambled input order: the remainder goes by name, not by position.
- The payer is not a participant, or is one of several participants.
- Overpaying in `settle`: the balances cross zero and flip sign.
- A balance becoming 0 after an expense removes the entry from `balances`.
- `removeMember` of a member who is owed money or owes money returns false and keeps them.
- Rejected calls (unknown member in the middle of the list, duplicates) must not apply the part of the expense that was valid.
- Greedy trap: balances {+2, +3, -4, -5, +4} need 3 payments, matching largest debtor with largest creditor needs 4.
- Five independent pairs: exactly 5 payments; 20 people with random balances finish in seconds.
- Everybody square, a single member, or a group with just one debtor.

## Likely interviewer questions
- Why one balance per person and not a matrix of pairwise debts? What do you lose?
- How do you split 100 among 3, 10 among 4, 1 among 3? Is the result independent of the order of participants?
- How do you make an expense with several affected members atomic with respect to concurrent `balances`?
- Why is greedy settlement not optimal? What is the true minimum, and why is the problem hard in general?
- How big can the zero-sum search get and how do you keep it under 10 seconds for 20 people?

## Invariants
- The sum of all balances in a group is always zero.
- `totalSpent` equals the sum of the amounts of all successful expenses and ignores settlements.
- A rejected operation changes no balance, no member and no total.
- Applying the payments returned by `settleUp` makes all balances zero, with each payment amount positive.

## Common mistakes
- Distributing the remainder to the first participants in input order rather than ascending names.
- Dividing with floating point or rounding each share separately so the sum differs from the amount.
- Updating some balances, then discovering an invalid participant and leaving a half-applied expense.
- Counting settlements in `totalSpent`, or forgetting to remove zero balances from the map.
- Returning the payer's share from the payer's own entry twice when the payer is also a participant.
- Greedy 'biggest debtor pays biggest creditor' for `settleUp`, which is not minimal.
- Exploring all permutations or `3^n` subset pairs for 20 people, which is too slow.

## Over-engineering traps
- A `Splitter` strategy hierarchy, an `Expense` entity with ids and a ledger for the tests that only need net balances.
- Floating-point `BigDecimal` money and multi-currency now.
- A full graph model of debts with edge simplification instead of net balances.

## Design and scaling discussion
- Net balances are a sufficient statistic for settling, but an audit trail needs the ledger of expenses; derive balances from the ledger or keep both consistent.
- Minimising the number of transactions is NP-hard in general (it is equivalent to partitioning into the maximum number of zero-sum subsets); a bitmask DP is exact for about 20 people, and greedy heuristics are good enough for large groups.
- Using integer minor units avoids rounding drift; a deterministic remainder rule keeps all clients and replicas in agreement.
- Concurrency: one lock per group is natural, since expenses only touch one group; cross-group views read several groups and must define their consistency.
- Idempotency keys make a retry of 'add expense' safe under at-least-once delivery.

## Trade-off questions to ask when the candidate is done
- Incremental balances vs balances recomputed from the ledger on each read: speed vs the ability to edit and audit.
- Do you want the minimal number of payments or the minimal total amount moved? (The total amount is fixed; the number is not.)
- How would you give an approximate `settleUp` for groups of 500 people and what guarantee would you state?
- What would you test with property tests: zero-sum, applying `settleUp` yields zero, the count is at most `n-1`?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.
