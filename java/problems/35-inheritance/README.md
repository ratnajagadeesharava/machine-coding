# 35 · Inheritance (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 30 min | Level 2 — four stub classes with the `extends` relationships declared; you write the bodies. |

## Problem

A payroll tool computes monthly pay for different kinds of staff. Every kind has a name and a base salary; only
the pay rule and the role label differ.

## Requirements

1. `Employee(name, baseMonthly)`: name must not be blank and the salary must not be negative (`IllegalArgumentException`). `monthlyPay()` is the base salary. `role()` is `"Employee"`.
2. `Engineer(name, base, level)`, level `1..3`: pay = `base * (100 + 20 * (level - 1)) / 100` (integer division). Role `"Engineer"`.
3. `Intern(name, base)`: pay = `base / 2`. Role `"Intern"`.
4. `Manager(name, base, reports)`: pay = **own base pay + (sum of the reports' `monthlyPay()`) / 20**. Role `"Manager"`. A report can itself be a manager.
5. `describe()` returns `"{role}: {name} earns {monthlyPay}"`, e.g. `"Engineer: E earns 1400"`.
6. The tests check with reflection that no subclass declares its own `describe()`.

## Examples

```
Manager base 5000, reports: Engineer(L2, 4000 -> 4800), Intern(1000 -> 500)
pay = 5000 + (4800 + 500) / 20 = 5265
```

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
