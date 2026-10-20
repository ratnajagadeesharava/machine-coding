# 73 · Chain of Responsibility (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — `ExpenseRequest` and the `Decider` trait are provided; you write the four handlers and `ApprovalChain`. |


## Problem

Expense claims need approval, and who can approve depends on the amount: the team lead handles small claims, the
manager bigger ones, the director the biggest. Some rules (no gambling) must stop a claim before anyone approves it.
The sender hands the claim to the chain.

What you are building: a company's expense-approval workflow. An employee submits a claim (`ExpenseRequest { employee,
amount_cents, category }`). A policy check can reject it outright; otherwise the claim travels up the management ladder
until someone with a large enough limit approves it. The result of handling a claim is a single `String` such as
`"APPROVED:manager"`. The order of handlers decides who answers, and a handler written by the caller (for example one
that only logs requests) must be able to sit anywhere in the chain.

### Glossary

- **Cents**: all amounts are whole numbers of cents (`50000` = 500.00). "Up to N" means `amount <= N`.
- **Chain**: an ordered list of handlers; the claim is offered to them front to back.
- **Decision**: `Some(text)` from a handler that settles a claim; `None` means "not mine, pass it on".

## Requirements

1. `ApprovalChain::new()` is an empty chain; `then(handler)` (a `Box<dyn Decider>`) appends a handler to the **end** and returns the chain (builder style).
2. `handle(&request)` returns a `String` and asks the handlers in order. The first `Some(decision)` is the answer and **the rest of the chain is not asked**. If nobody decides: `"REJECTED:no approver for <amount_cents>"` (an empty chain therefore rejects everything).
3. Handlers implement `Decider` (`decide(&self, &ExpenseRequest) -> Option<String>`; each has `new()`; amounts in cents, limits inclusive):
   - `PolicyGuard`: `amount <= 0` or category `gambling` (any case) → `Some("REJECTED:policy violation")`; otherwise `None`.
   - `TeamLead`: up to **50000** → `Some("APPROVED:team lead")`, except category `travel` (any case), which always goes up; otherwise `None`.
   - `Manager`: up to **500000** → `Some("APPROVED:manager")`; otherwise `None`.
   - `Director`: up to **5000000** → `Some("APPROVED:director")`; otherwise `None`.
4. A `Decider` written by the caller can be placed anywhere in a chain, at the head, in the middle or at the end, alongside the four above.

## Examples

Chain `TeamLead, Manager, Director`, category `meals` unless stated:

| Claim | Answer |
|---|---|
| 50000 | `APPROVED:team lead` |
| 50001 | `APPROVED:manager` |
| 500000 | `APPROVED:manager` |
| 500001 | `APPROVED:director` |
| 5000000 | `APPROVED:director` |
| 5000001 | `REJECTED:no approver for 5000001` |
| 10000, category `Travel` | `APPROVED:manager` |

Order matters: a chain `Director, TeamLead` answers a claim of 100 with `APPROVED:director`. An empty chain answers 100 with `REJECTED:no approver for 100`.

Guard: chain `PolicyGuard, Audit, TeamLead`, where `Audit` is a caller-written `Decider` that records every claim it sees and returns `None`. A claim of 100 in category `Gambling`, or of amount `0` or `-5`, answers `REJECTED:policy violation` and `Audit` sees nothing; a claim of 100 `meals` answers `APPROVED:team lead` and `Audit` has seen exactly that one claim.

## Edge cases

- A chain holding only a `TeamLead` answers 50001 with `REJECTED:no approver for 50001`.
- Each handler on its own: `TeamLead.decide(100)` is `Some("APPROVED:team lead")` and `TeamLead.decide(100000)` is `None`; `PolicyGuard.decide(100)` is `None` and `PolicyGuard.decide(0)` is `Some("REJECTED:policy violation")`.
- Category matching ignores case (`Travel`, `GAMBLING`).

## Running

```bash
cargo test -p p73_chain_of_responsibility     # from rust/ — tests FAIL until you implement the stubs
cargo play run 73
```
