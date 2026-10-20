# 35 · Inheritance (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 30 min | Level 2 — `PayrollError` is provided; `Staff`, the `Payable` trait and three employee types are stubs you fill in. |

## Problem

**What you are building.** A payroll tool computes monthly pay for different kinds of staff. Every kind has a name
and a base salary; only the pay rule and the role label differ (an engineer is paid more per level, an intern gets
half, a manager gets a bonus based on the people reporting to them). Whatever the kind, asking for a one-line
description of a person must give the same sentence shape, so any list of staff can be printed uniformly, even for
a new kind that only says where its name and salary live.

### Terms used below

- **Staff**: the pair (name, base salary) every kind of employee has.
- **Base salary / base**: the monthly amount an employee is hired at, as a whole number.
- **Monthly pay**: what the employee actually earns per month after the kind-specific rule is applied.
- **Role**: a label for the kind of employee (`"Engineer"`, ...).
- **Report**: a staff member who reports to a manager. A report may itself be a manager.
- **Integer division**: fractions are dropped (`1001 / 2 = 500`); all pay is a whole number.

## Requirements

1. `Staff::new(name, base)` (`base` is an `i64`): a blank name (empty or only whitespace) → `Err(PayrollError::InvalidName)`; a negative salary → `Err(PayrollError::NegativeSalary)`. `name()` returns the name; `base()` the salary (a salary of `0` is allowed).
2. `Payable` has `staff()` and the methods `role()` = `"Employee"`, `monthly_pay()` = base salary, `describe()` = `"{role}: {name} earns {monthly_pay}"`. A type that only provides `staff()` still answers all three: role `"Employee"`, pay = base, and the matching description.
3. `Engineer::new(staff, level)`, level `1..=3` (else `Err(PayrollError::InvalidLevel)`): pay = `base * (100 + 20 * (level - 1)) / 100` (integer division), i.e. +20% per level above 1. Role `"Engineer"`.
4. `Intern::new(staff)`: pay = `base / 2` (integer division). Role `"Intern"`.
5. `Manager::new(staff, reports)` (`reports` is a `Vec<Box<dyn Payable>>`): pay = **own base + (sum of reports' `monthly_pay()`) / 20** (integer division; the sum is divided, not each report). Role `"Manager"`. Reports may be managers, in which case their full `monthly_pay()` counts. A manager with no reports is paid just the base.
6. `describe()` works the same way for every kind, through a `&dyn Payable` / `Box<dyn Payable>`.

## Examples

```
Manager base 5000, reports: Engineer(L2, 4000 -> 4800), Intern(1000 -> 500)
pay = 5000 + (4800 + 500) / 20 = 5265
```

Engineer levels with base 4000: level 1 → `4000`, level 2 → `4800`, level 3 → `5600`. Intern with base 1001 → `500`.

Describing a mixed list (all with base 1000):

```
Plain (only staff())     -> "Employee: P earns 1000"
Engineer("E", level 3)   -> "Engineer: E earns 1400"
Intern("I")              -> "Intern: I earns 500"
Manager("M", no reports) -> "Manager: M earns 1000"
```

Nested managers: `team = Manager("Lead", 5000, [E(L2,4000), I(1000)])` pays 5265; `Manager("Head", 8000, [team])` pays `8000 + 5265 / 20 = 8263`.

## Edge cases

- `Staff::new("  ", 1)` → `Err(InvalidName)`; `Staff::new("A", -1)` → `Err(NegativeSalary)`; `Engineer::new(staff, 0)` and `Engineer::new(staff, 4)` → `Err(InvalidLevel)`.
- Integer division applies at every step: `1001 / 2 = 500`; a manager's bonus of `5300 / 20 = 265`.

## Running

```bash
cargo test -p p35_inheritance     # from rust/ — tests FAIL until you implement the stubs
cargo play run 35
```
