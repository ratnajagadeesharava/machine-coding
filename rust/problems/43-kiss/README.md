# 43 · KISS — Keep It Simple (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 20 min | Level 2 — a `late_fee` stub is provided; you write the body. |

## Problem

A library charges late fees. The rule is tiny and has not changed in ten years.

## Requirements

`late_fee(days_late, premium)` returns the fee in cents:

1. `days_late <= 0` → 0.
2. Standard member: 25 cents per day, **capped at 1000**.
3. Premium member: 10 cents per day, **capped at 500**.
4. `i32::MAX` days must not overflow.

## Running

```bash
cargo test -p p43_kiss     # from rust/ — tests FAIL until you implement late_fee
cargo play run 43
```
