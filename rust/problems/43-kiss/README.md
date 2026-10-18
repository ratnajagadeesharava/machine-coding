# 43 · KISS — Keep It Simple (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 20 min | Level 2 — one stub function, `late_fee`, in `src/lib.rs` (`todo!()`). |

> **Principle:** the simplest thing that works is usually the right thing. Do not build trait hierarchies, policy
> structs and config loaders for a rule that fits in five lines.

## Problem

A library charges late fees. The rule is tiny and has not changed in ten years. It is tempting to build a
`FeePolicy` trait, two implementors, a factory and a config file. **Don't.**

## Requirements

`late_fee(days_late, premium)` returns the fee in cents:

1. `days_late <= 0` → 0.
2. Standard member: 25 cents per day, **capped at 1000**.
3. Premium member: 10 cents per day, **capped at 500**.
4. `i32::MAX` days must not overflow.

## Constraints — the KISS budget (checked by the tests)

- The whole solution is in `src/lib.rs`, with **at most 25 lines of code** (blank lines and `//` comments do not count).
- None of: `trait `, `dyn `, `struct `, `enum `, `impl `, `Box<`, `macro_rules`, `mod `. A plain function is enough.

## Running

```bash
cargo test -p p43_kiss     # from rust/ — tests FAIL until you implement late_fee
cargo play run 43
```
