# 35 · Inheritance (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 30 min | Level 2 — `PayrollError` is provided; `Staff`, the `Payable` trait and three employee types are stubs you fill in. |

## Problem

A payroll tool computes monthly pay for different kinds of staff. Every kind has a name and a base salary; only
the pay rule and the role label differ.

## Requirements

1. `Staff::new(name, base)`: blank name → `Err(PayrollError::InvalidName)`, negative salary → `Err(PayrollError::NegativeSalary)`.
2. `Payable` has `staff()` and the methods `role()` = `"Employee"`, `monthly_pay()` = base salary, `describe()` = `"{role}: {name} earns {monthly_pay}"`.
3. `Engineer::new(staff, level)`, level `1..=3` (else `InvalidLevel`): pay = `base * (100 + 20 * (level - 1)) / 100`. Role `"Engineer"`.
4. `Intern::new(staff)`: pay = `base / 2`. Role `"Intern"`.
5. `Manager::new(staff, reports)`: pay = **own base + (sum of reports' `monthly_pay()`) / 20**. Role `"Manager"`. Reports may be managers.
6. The tests define their own `Plain` type that implements only `staff()` and expect `role()`, `monthly_pay()` and `describe()` to behave as in requirement 2.

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
