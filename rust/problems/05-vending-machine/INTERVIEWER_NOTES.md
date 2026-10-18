# Interviewer notes — 05 Vending Machine

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- Exact payment works with an empty coin inventory.
- Cancel returns the *same* coins inserted, even if the inventory could provide others.
- Rule priority: invalid slot beats everything; sold out beats insufficient funds.
- Coin inventory shrinks as change is dispensed; a later purchase may become `CANNOT_MAKE_CHANGE`.
- A refused purchase keeps the balance so the customer can add more coins.

## Likely interviewer questions
- What are the states? Is "has money" a state or just `balance > 0`? Where does "which operations are legal now" live?
- What happens when `select` is called twice in a row? When coins are inserted after a purchase?
- Is greedy change-making correct? When does greedy fail (coin systems, limited inventory)?
- Is the product a class? What if prices change, or a slot holds different products over time?

## Invariants
- `balance` equals the sum of escrowed coins.
- Coin inventory counts never go negative.
- A purchase is all-or-nothing.
- `Σ money in = Σ money out + price × purchases` (conservation).

## Common mistakes
- Moving inserted coins into inventory immediately, so `cancel` cannot return them.
- Greedy change that ignores limited inventory (takes a quarter that doesn't exist).
- Resetting balance on a refused purchase.
- Check order differing from the contract (e.g. insufficient funds reported for a sold-out slot).

## Over-engineering traps
- A full GoF State pattern with five state classes when the only "state" that changes legality is balance/maintenance mode.
- `Coin` enum, `Product` class, `Slot` class, `Inventory` interface, `ChangeStrategy` — each should be justified by a requirement.

## Scalability discussion
- Not a scaling problem; discuss correctness and extension (payments, maintenance mode).

## Trade-offs worth probing
- Explicit state machine vs guard clauses.
- Bounded coin-change: greedy with fallback vs DP; denominations are tiny, so DP is cheap.

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.
