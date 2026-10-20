# 73 · Chain of Responsibility (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — the `ExpenseRequest` record is provided; you write `ApprovalHandler` and the four handlers. |


## Problem

Expense claims need approval, and who can approve depends on the amount: the team lead handles small claims, the
manager bigger ones, the director the biggest. Some rules (no gambling) must stop a claim before anyone approves it.
The sender hands the claim to the first handler.

## Requirements

1. `ApprovalHandler` (abstract): `setNext(next)` links the next handler **and returns it** (so `a.setNext(b).setNext(c)` builds a chain); linking again replaces the previous next; `null` → `IllegalArgumentException`.
2. `String handle(request)`: asks `Optional<String> decide(request)`. A present decision is the answer and **the rest of the chain is not asked**. Empty → pass to the next handler. At the end of the chain with no decision: `"REJECTED:no approver for <amountCents>"`. `null` request → `IllegalArgumentException`.
3. Handlers (amounts in cents, limits inclusive):
   - `PolicyGuard`: `amount <= 0` or category `gambling` (any case) → `"REJECTED:policy violation"`; otherwise passes.
   - `TeamLead`: up to **50000** → `"APPROVED:team lead"`, except category `travel` (any case), which always goes up; otherwise passes.
   - `Manager`: up to **500000** → `"APPROVED:manager"`; otherwise passes.
   - `Director`: up to **5000000** → `"APPROVED:director"`; otherwise passes.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
