# 43 · KISS — Keep It Simple (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 20 min | Level 2 — a `late_fee` stub is provided; you write the body. |

## Problem

**What you are building.** A public library charges a late fee when a member returns an item after its due date. The rule is tiny and has not changed in ten years: a fixed amount per day late, with a ceiling so nobody pays an absurd amount, and a cheaper rate for premium members. You write one function that takes how many days late the item is and whether the member is premium, and returns the fee in cents.

## Requirements

`late_fee(days_late, premium)` takes an `i32` number of days late and a boolean, and returns the fee in **cents** as `u64` (a whole number, never negative).

| Member | Rate per late day | Fee ceiling (cap) |
|---|---|---|
| Standard (`premium = false`) | 25 cents | 1000 cents |
| Premium (`premium = true`) | 10 cents | 500 cents |

1. `days_late <= 0` (on time or returned early) → fee is 0, for both member types.
2. Standard member: 25 cents per day, **capped at 1000**.
3. Premium member: 10 cents per day, **capped at 500**.
4. The fee is `min(rate × days_late, cap)`. The cap is reached at 40 days for standard and 50 days for premium; every value beyond that returns the cap.
5. Huge values (`i32::MAX`) must not overflow: they simply return the cap.
6. No errors or exceptions: every input has a defined answer.

## Examples

| Call | Result | Why |
|---|---|---|
| `late_fee(0, false)` | 0 | on time |
| `late_fee(-3, true)` | 0 | returned early |
| `late_fee(1, false)` | 25 | 1 × 25 |
| `late_fee(10, false)` | 250 | 10 × 25, under the cap |
| `late_fee(30, true)` | 300 | 30 × 10, under the cap |
| `late_fee(40, false)` | 1000 | 40 × 25 = 1000, exactly the cap |
| `late_fee(41, false)` | 1000 | 1025 would exceed the cap |
| `late_fee(51, true)` | 500 | 510 would exceed the cap |
| `late_fee(i32::MAX, true)` | 500 | no overflow |

## Edge cases

- Exactly 0 days late is free; 1 day late is the first charged day.
- The cap is inclusive: reaching it exactly (40 days standard, 50 days premium) is not an error, the fee is just the cap.

## Glossary

- **Cents**: the smallest currency unit; all amounts here are whole cents (1000 cents = 10.00).
- **Cap**: the maximum fee a member can ever be charged, regardless of how late the item is.
- **Premium member**: a member on the higher membership tier, with a lower rate and lower cap.

## Running

```bash
cargo test -p p43_kiss     # from rust/ — tests FAIL until you implement late_fee
cargo play run 43
```
