# 73 · Chain of Responsibility (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — `ExpenseRequest` and the `Decider` trait are provided; four handlers and `ApprovalChain` are stubs. |

> **Pattern:** *Chain of Responsibility* — give more than one object a chance to handle a request by linking the
> receiving objects into a chain and passing the request along until one handles it. The sender does not know which
> handler will answer; handlers can be added, removed or reordered without touching the sender.
>
> **Rust adaptation:** a classic linked list of handlers (each holding `Option<Box<next>>`) fights ownership, so the
> chain is an ordered `Vec<Box<dyn Decider>>`. The walk "ask, and pass on if undecided" is the same.

## Problem

Expense claims need approval, and who can approve depends on the amount: the team lead handles small claims, the
manager bigger ones, the director the biggest. Some rules (no gambling) must stop a claim before anyone approves it.
The sender just hands the claim to the chain.

## Requirements

1. `ApprovalChain::new()` is an empty chain; `then(handler)` appends a handler to the **end** and returns the chain (builder style).
2. `handle(&request)` asks the handlers in order. The first `Some(decision)` is the answer and **the rest of the chain is not asked**. If nobody decides: `"REJECTED:no approver for <amount_cents>"` (an empty chain therefore rejects everything).
3. Handlers implement `Decider` (amounts in cents, limits inclusive):
   - `PolicyGuard`: `amount <= 0` or category `gambling` (any case) → `Some("REJECTED:policy violation")`; otherwise `None`.
   - `TeamLead`: up to **50000** → `Some("APPROVED:team lead")`, except category `travel` (any case), which always goes up; otherwise `None`.
   - `Manager`: up to **500000** → `Some("APPROVED:manager")`; otherwise `None`.
   - `Director`: up to **5000000** → `Some("APPROVED:director")`; otherwise `None`.

## Constraints (what the tests catch)

- The tests put an `Audit` handler (defined in the test file; it records requests and never decides) at the start of a chain and behind a guard, and check that a guard rejection never reaches it.
- Putting `Director` first makes it approve a 100-cent claim: order is part of the design.

## Running

```bash
cargo test -p p73_chain_of_responsibility     # from rust/ — tests FAIL until you implement the stubs
cargo play run 73
```
