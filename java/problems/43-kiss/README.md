# 43 · KISS — Keep It Simple (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 20 min | Level 2 — one stub class, `LateFees`. The body throws `UnsupportedOperationException`. |

> **Principle:** the simplest thing that works is usually the right thing. Do not build strategy hierarchies,
> policy objects and config loaders for a rule that fits in five lines.

## Problem

A library charges late fees. The rule is tiny and has not changed in ten years. It is tempting to build a
`FeePolicy` interface, a `StandardFeePolicy`, a `PremiumFeePolicy`, a factory and a config file. **Don't.**

## Requirements

`LateFees.fee(daysLate, premium)` returns the fee in cents:

1. `daysLate <= 0` → 0.
2. Standard member: 25 cents per day, **capped at 1000**.
3. Premium member: 10 cents per day, **capped at 500**.
4. Huge values (`Integer.MAX_VALUE`) must not overflow.

## Constraints — the KISS budget (checked by the tests)

- The `solution` package contains **exactly one** source file.
- That file has **at most 25 lines of code** (blank lines and comments do not count), including `package`, class and method lines.
- None of: `interface`, `abstract`, `extends`, `implements`, `enum`, `record`, `switch`.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement fee()
```
