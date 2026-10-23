# 35 · Inheritance (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 30 min | Level 2 — four stub classes with the `extends` relationships declared; you write the bodies. |

## Problem

**What you are building.** A payroll tool computes monthly pay for different kinds of staff. Every kind has a name
and a base salary; only the pay rule and the role label differ (an engineer is paid more per level, an intern gets
half, a manager gets a bonus based on the people reporting to them). Whatever the kind, asking for a one-line
description of a person must give the same sentence shape, so any list of staff can be printed uniformly.

### Terms used below

- **Base salary / base**: the monthly amount an employee is hired at, as a whole number (`long`).
- **Monthly pay**: what the employee actually earns per month after the kind-specific rule is applied.
- **Role**: a label for the kind of employee (`"Engineer"`, ...).
- **Report**: an employee who reports to a manager. A report may itself be a manager.
- **Integer division**: fractions are dropped (`1001 / 2 = 500`); all pay is a whole `long`.

## Requirements

1. `Employee(name, baseMonthly)`: name must not be blank (empty or only whitespace) and the salary must not be negative (`IllegalArgumentException`). `name()` returns the name. `monthlyPay()` is the base salary. `role()` is `"Employee"`.
2. `Engineer(name, base, level)`, level `1..3` (outside that range → `IllegalArgumentException`): pay = `base * (100 + 20 * (level - 1)) / 100` (integer division), i.e. +20% per level above 1. Role `"Engineer"`.
3. `Intern(name, base)`: pay = `base / 2` (integer division). Role `"Intern"`.
4. `Manager(name, base, reports)`: pay = **own base salary + (sum of the reports' `monthlyPay()`) / 20** (integer division; the sum is divided, not each report). Role `"Manager"`. A report can itself be a manager, in which case its full `monthlyPay()` counts. A manager with no reports is paid just the base.
5. `describe()` returns `"{role}: {name} earns {monthlyPay}"`, e.g. `"Engineer: E earns 1400"`. It works for every kind through an `Employee` reference.
6. The tests check with reflection that each of `Engineer`, `Intern` and `Manager` extends `Employee`, declares its own `role()`, and does **not** declare its own `describe()` (`describe()` is declared only on `Employee`).
7. The invalid-input rules also apply to `Intern` (e.g. an empty name is rejected).

## Examples

```
Manager base 5000, reports: Engineer(L2, 4000 -> 4800), Intern(1000 -> 500)
pay = 5000 + (4800 + 500) / 20 = 5265
```

Engineer levels with base 4000: level 1 → `4000`, level 2 → `4800`, level 3 → `5600`. Intern with base 1001 → `500`.

Describing a mixed list (all with base 1000):

```
Employee("P")            -> "Employee: P earns 1000"
Engineer("E", level 3)   -> "Engineer: E earns 1400"
Intern("I")              -> "Intern: I earns 500"
Manager("M", no reports) -> "Manager: M earns 1000"
```

Nested managers: `team = Manager("Lead", 5000, [E(L2,4000), I(1000)])` pays 5265; `Manager("Head", 8000, [team])` pays `8000 + 5265 / 20 = 8263`.

## Edge cases

- `new Employee(" ", 1)`, `new Employee("A", -1)`, `new Engineer("A", 1, 0)`, `new Engineer("A", 1, 4)`, `new Intern("", 1)` all throw `IllegalArgumentException`.
- A salary of `0` is allowed.
- Integer division applies at every step: `1001 / 2 = 500`; a manager's bonus of `5300 / 20 = 265`.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
