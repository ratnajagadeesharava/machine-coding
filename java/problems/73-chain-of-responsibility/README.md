# 73 · Chain of Responsibility (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — the `ExpenseRequest` record is provided; you write `ApprovalHandler` and the four handlers. |


## Problem

Expense claims need approval, and who can approve depends on the amount: the team lead handles small claims, the
manager bigger ones, the director the biggest. Some rules (no gambling) must stop a claim before anyone approves it.
The sender hands the claim to the first handler.

What you are building: a company's expense-approval workflow. An employee submits a claim (`ExpenseRequest(employee,
amountCents, category)`). A policy check can reject it outright; otherwise the claim travels up the management ladder
until someone with a large enough limit approves it. The result of handling a claim is a single text answer such as
`APPROVED:manager`. The order in which handlers are linked decides who answers, and a new kind of handler (for example
one that only logs requests) must be able to sit anywhere in a chain.

### Glossary

- **Cents**: all amounts are whole numbers of cents (`50000` = 500.00). "Up to N" means `amount <= N`.
- **Chain**: handlers linked with `setNext`; a claim is given to the first one.
- **Decision**: the text a handler returns when it settles a claim; a handler that does not settle it passes it on.

## Requirements

1. `ApprovalHandler` (abstract): `setNext(next)` links the next handler **and returns it** (so `a.setNext(b).setNext(c)` builds a chain); linking again replaces the previous next; `null` → `IllegalArgumentException`.
2. `String handle(request)`: asks `Optional<String> decide(request)`. A present decision is the answer and **the rest of the chain is not asked**. Empty → pass to the next handler. At the end of the chain with no decision (including a handler with no next at all): `"REJECTED:no approver for <amountCents>"`. `null` request → `IllegalArgumentException`.
3. Handlers (amounts in cents, limits inclusive):
   - `PolicyGuard`: `amount <= 0` or category `gambling` (any case) → `"REJECTED:policy violation"`; otherwise passes.
   - `TeamLead`: up to **50000** → `"APPROVED:team lead"`, except category `travel` (any case), which always goes up; otherwise passes.
   - `Manager`: up to **500000** → `"APPROVED:manager"`; otherwise passes.
   - `Director`: up to **5000000** → `"APPROVED:director"`; otherwise passes.
4. A handler written by the caller (a subclass of `ApprovalHandler` that supplies `decide`) can be placed anywhere in a chain, at the head, in the middle or at the end, alongside the four above.

## Examples

Chain `TeamLead -> Manager -> Director`, category `meals` unless stated:

| Claim | Answer |
|---|---|
| 50000 | `APPROVED:team lead` |
| 50001 | `APPROVED:manager` |
| 500000 | `APPROVED:manager` |
| 500001 | `APPROVED:director` |
| 5000000 | `APPROVED:director` |
| 5000001 | `REJECTED:no approver for 5000001` |
| 10000, category `Travel` | `APPROVED:manager` |

Order matters: a chain `Director -> TeamLead` answers a claim of 100 with `APPROVED:director`.

Guard: chain `PolicyGuard -> Audit -> TeamLead`, where `Audit` is a caller-written handler that records every claim it sees and never decides. A claim of 100 in category `Gambling`, or of amount `0` or `-5`, answers `REJECTED:policy violation` and `Audit` sees nothing; a claim of 100 `meals` answers `APPROVED:team lead` and `Audit` has seen exactly that one claim.

## Edge cases

- A lone `TeamLead` with no next handler answers 50001 with `REJECTED:no approver for 50001`.
- Linking `lead.setNext(x)` and later `lead.setNext(y)` leaves only `y` as the next handler.
- `lead.handle(null)` and `lead.setNext(null)` throw `IllegalArgumentException` and leave the handler working as before.
- Category matching ignores case (`Travel`, `GAMBLING`).

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
