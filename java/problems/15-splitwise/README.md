# 15 · Splitwise (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 60 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building the engine behind an expense-sharing app such as Splitwise. A group of friends (a flat, a trip)
records shared costs: "Alice paid 300 for dinner, split between Alice, Bob and Carol". The engine keeps track of who
owes whom, answers "how much does Bob owe Alice right now?", records the payments people make to settle up, and can
boil the whole web of debts down to the fewest payments that clear everybody. There are no accounts or logins:
anyone whose name appears in a call simply exists.

## Glossary

- **Payer**: the person who actually paid the whole bill out of their pocket.
- **Participant**: someone who benefits from the expense and therefore owes a share of it (to the payer).
- **Share**: the part of an expense assigned to one participant.
- **Net / balance**: a person's overall position. Positive means others owe them money in total; negative means they owe others; the positions of all people always add up to 0.
- **Settlement**: a real-world payment `from` -> `to` that reduces what `from` owes `to`.
- **Minor unit**: the smallest money unit (cents). All amounts are whole numbers of these.

## Requirements

1. People are identified by name (case-sensitive strings); there is no registration step (any name may appear in an expense or settlement).
2. An expense is paid in full by one **payer** and shared among participants in one of three ways:
   - **Equal** among a list of participants,
   - **Exact** amounts per participant (which must add up to the total),
   - **Percent** per participant (integers adding up to 100).
3. Amounts are whole currency units (cents). Splits must add up **exactly** to the total, so when an equal or percent split leaves a remainder it is distributed **one unit at a time to participants in ascending name order** (plain string order; the first `remainder` names get one extra unit each).
   - Equal: every participant gets `floor(total / n)`, then the remainder is handed out as above.
   - Percent: every participant first gets `floor(total × pct / 100)`, then the remainder (`total` minus the sum of those floors) is handed out as above.
   - Exact: the given amounts are used as they are.
4. The payer may or may not be a participant. Every participant other than the payer owes the payer their share; a payer who is also a participant owes nothing to themselves (their own share creates no debt).
5. `owed(a, b)` is the **net** amount `a` owes `b` after offsetting both directions (what `a` owes `b` minus what `b` owes `a`), or 0 when `a` owes `b` nothing, when `b` owes `a` instead, or when either name is unknown.
6. `balances()` gives each person's net position over all expenses and settlements (positive = is owed money, negative = owes money). People whose position is exactly 0 are omitted. A payer's position rises by the amount paid on behalf of others; a participant's falls by their share.
7. A **settlement** `settle(from, to, amount)` records that `from` actually paid `to`: it reduces what `from` owes `to` by `amount` (the position of `from` rises by `amount`, that of `to` falls by `amount`).
8. `simplify()` returns a list of payments that would bring everyone to zero, using at most `(number of people with a non-zero balance) − 1` payments. It only reports; it does not record anything.
9. Invalid expenses are refused (the call returns `false`) and change nothing. Invalid means: non-positive amount, no participants (empty list or empty map), duplicate participants in an equal split, exact shares that do not add up to the total, percents that do not add up to 100, or any negative share or negative percent.

## Constraints

- Up to 100,000 expenses among 1,000 people; `owed`, `balances` and `simplify` must stay cheap (do not replay every expense on each query).
- Money is integral; never use floating point.

## Examples

Equal split with remainder (payer is also a participant):

```
A pays 100 equally for [A, B, C]:   shares 34, 33, 33 (the 1 left over goes to A, first by name)
                                    owed(B, A) = 33, owed(C, A) = 33, owed(A, B) = 0
                                    balances = {A: +66, B: -33, C: -33}
```

Debts in both directions are netted:

```
A pays 100 equally for [A, B]:      B owes A 50
B pays 30 equally for [A, B]:       A owes B 15
                                    owed(B, A) = 35, owed(A, B) = 0
```

Percent split with a remainder (shares are given in any order; the remainder still goes by name):

```
P pays 101 by percent  C:34, A:33, B:33
    floors: C = floor(101*34/100) = 34, A = 33, B = 33   -> 100, remainder 1
    remainder goes to "A"                                 -> A 34, B 33, C 34
```

Settlement and simplification:

```
A pays 100 equally for [A, B]:      owed(B, A) = 50
settle(B, A, 20) -> true            owed(B, A) = 30
settle(B, A, 30) -> true            owed(B, A) = 0, balances = {}

A pays 200 equally for [A, B]:      B owes A 100
B pays 200 equally for [B, C]:      C owes B 100
balances = {A: +100, B: 0 (omitted), C: -100}
simplify() = ["C>A:100"]            (B is skipped: one payment instead of two); stored debts unchanged
```

## Edge cases

- `addEqualExpense("A", 50, ["A"])` is valid and creates no debt: `balances()` stays empty.
- `addExactExpense("A", 100, {B: 130, C: -30})` is refused even though 130 + (−30) = 100, because a share is negative.
- `addPercentExpense("A", 100, {B: 120, C: -20})` is refused (negative percent); so are `50 + 49` and `60 + 50`.
- A payer who is not a participant: `A pays 90 equally for [B, C]` gives `owed(B, A) = 45`, `owed(C, A) = 45`, `balances()["A"] = 90`.
- Settling more than is owed is allowed and returns `true`; the debt flips direction. Example: B owes A 50, `settle(B, A, 80)` gives `owed(B, A) = 0` and `owed(A, B) = 30`.
- `settle` with amount `<= 0` or with `from == to` returns `false` and changes nothing.
- `owed` and `balances` for people never mentioned: `0` and absent.

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- All three `add…Expense` methods return `true` when recorded and `false` when refused.
- `owed(a, b)` is `≥ 0` and `0` for unknown people or when `a` owes nothing to `b`.
- `balances()` contains only people with non-zero net position; the values sum to 0.
- `settle(from, to, amount)` returns `false` for a non-positive amount or when `from == to`; otherwise it records that `from` paid `to` (so `owed(from, to)` decreases by `amount`, possibly going negative, i.e. `to` then owes `from`) and returns `true`.
- `simplify()` returns strings of the form `debtor>creditor:amount` (for example `C>A:100`, amount a positive whole number), sorted ascending as plain strings, such that applying them as settlements zeroes every balance. When several valid answers exist the tests accept any of them (they check that every balance is zeroed and that there are at most `n − 1` payments). It does not change the stored state.

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
