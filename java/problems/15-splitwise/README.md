# 15 · Splitwise (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 60 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Build an expense-sharing engine like Splitwise: friends record who paid for what and how it was split, and the
system tells everybody who owes whom, lets them settle up, and can simplify the web of debts into few payments.

## Requirements

1. People are identified by name; there is no registration step (any name may appear in an expense).
2. An expense is paid in full by one **payer** and shared among participants in one of three ways:
   - **Equal** among a list of participants,
   - **Exact** amounts per participant (which must add up to the total),
   - **Percent** per participant (integers adding up to 100).
3. Amounts are whole currency units (cents). Splits must add up **exactly** to the total: when an equal or percent split leaves a remainder, it is distributed **one unit at a time to participants in ascending name order**. Percent shares are first computed as `floor(total × pct / 100)`.
4. The payer may or may not be a participant; a payer who is a participant owes nothing to themselves.
5. `owed(a, b)` is the **net** amount `a` owes `b` after offsetting both directions, or 0.
6. `balances()` gives each person's net position over all expenses and settlements (positive = is owed money, negative = owes money). Zero balances are omitted.
7. A **settlement** records a payment between two people, reducing the debt.
8. `simplify()` returns a list of payments that would bring everyone to zero, using at most `(number of people with a non-zero balance) − 1` payments.
9. Invalid expenses (non-positive amount, no participants, duplicate participants, shares not matching the total, percents not summing to 100, negative shares) are refused and change nothing.

## Constraints

- Up to 100,000 expenses among 1,000 people; `owed`, `balances` and `simplify` must stay cheap (do not replay every expense on each query).
- Money is integral; never use floating point.

## Examples

```
A pays 100 equally for [A, B, C]:   shares 34, 33, 33 (remainder to A first)
                                    owed(B, A) = 33, owed(C, A) = 33, balances = {A: +66, B: −33, C: −33}
B pays 30 for [A, B] equally:       A owes B 15  →  owed(B, A) = 18, owed(A, B) = 0
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- All three `add…Expense` methods return `true` when recorded and `false` when refused.
- `owed(a, b)` is `≥ 0` and `0` for unknown people or when `a` owes nothing to `b`.
- `balances()` contains only people with non-zero net position; the values sum to 0.
- `settle(from, to, amount)` returns `false` for a non-positive amount or when `from == to`; otherwise it records that `from` paid `to` (so `owed(from, to)` decreases by `amount`, possibly going negative, i.e. `to` then owes `from`).
- `simplify()` returns strings of the form `debtor>creditor:amount` (for example `C>A:100`), sorted ascending, such that applying them as settlements zeroes every balance. It does not change the stored state.

## Test entry point (Java)

The tests drive your system through the thin, primitive-typed contract below. It is **not** your domain
model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in
`solution/Solution.java`.

```java
package contract;

import java.util.List;
import java.util.Map;

/** Thin test entry point. Your domain model lives behind it. */
public interface SplitwiseApi {

    boolean addEqualExpense(String payer, long amount, List<String> participants);

    /** shares must add up to amount. */
    boolean addExactExpense(String payer, long amount, Map<String, Long> shares);

    /** percents (integers) must add up to 100. */
    boolean addPercentExpense(String payer, long amount, Map<String, Integer> percents);

    /** Net amount debtor owes creditor (>= 0). */
    long owed(String debtor, String creditor);

    /** Net position per person (positive = is owed); zero balances omitted. */
    Map<String, Long> balances();

    boolean settle(String from, String to, long amount);

    /** Payments "debtor>creditor:amount", sorted ascending, that would zero all balances. */
    List<String> simplify();
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    SplitwiseApi create()
}
```

## Language notes

Amounts are `long`. Maps passed in are caller-owned: copy what you keep.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/15-splitwise test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.
