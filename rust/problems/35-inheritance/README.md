# 35 · Inheritance (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 30 min | Level 2 — `src/lib.rs` has the stubs: `Staff`, the `Payable` trait and three employee types. Bodies are `todo!()`. |

> **Rust adaptation:** Rust has no class inheritance. The same idea is expressed with **composition + trait
> default methods**: shared data lives in `Staff`, shared behaviour lives in `Payable`'s provided methods, and each
> kind of employee wraps a `Staff` and overrides only what differs.

## Problem

A payroll tool computes monthly pay for different kinds of staff. Every kind has a name and a base salary; only
the pay rule and the role label differ.

## Requirements

1. `Staff::new(name, base)`: blank name → `Err(PayrollError::InvalidName)`, negative salary → `Err(PayrollError::NegativeSalary)`.
2. `Payable` requires only `staff()`. Its default methods: `role()` = `"Employee"`, `monthly_pay()` = base salary, `describe()` = `"{role}: {name} earns {monthly_pay}"`.
3. `Engineer::new(staff, level)`, level `1..=3` (else `InvalidLevel`): pay = `base * (100 + 20 * (level - 1)) / 100`. Role `"Engineer"`.
4. `Intern::new(staff)`: pay = `base / 2`. Role `"Intern"`.
5. `Manager::new(staff, reports)`: pay = **own base + (sum of reports' `monthly_pay()`) / 20**. Role `"Manager"`. Reports may be managers.
6. `describe()` is written once, as the trait default. No type overrides it. The tests define their own `Plain` type that implements only `staff()` and expect it to get the defaults.

## Examples

```
Manager base 5000, reports: Engineer(L2, 4000 -> 4800), Intern(1000 -> 500)
pay = 5000 + (4800 + 500) / 20 = 5265
```

## Running

```bash
cargo test -p p35_inheritance     # from rust/ — tests FAIL until you implement the stubs
cargo play run 35
```
