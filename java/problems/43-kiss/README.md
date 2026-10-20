# 43 · KISS — Keep It Simple (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 20 min | Level 2 — a `LateFees` stub is provided; you write the body. |

## Problem

A library charges late fees. The rule is tiny and has not changed in ten years.

## Requirements

`LateFees.fee(daysLate, premium)` returns the fee in cents:

1. `daysLate <= 0` → 0.
2. Standard member: 25 cents per day, **capped at 1000**.
3. Premium member: 10 cents per day, **capped at 500**.
4. Huge values (`Integer.MAX_VALUE`) must not overflow.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement fee()
```
