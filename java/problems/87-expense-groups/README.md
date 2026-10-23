# 87 · Expense Groups (Splitwise) (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 75 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building the backend of a Splitwise-like app for flatmates and travel groups. People share expenses: one person pays the restaurant bill for everyone, another pays the rent for two. The app tracks who owes whom as a net balance per person, lets people pay each other back (settlements), and at the end of the trip computes the smallest possible list of payments that makes everybody square.

Money is whole currency units (integers). A bill of 100 split three ways cannot be 33.33 each: it must add up to exactly 100 and be reproducible, so the leftover units are handed out by a fixed rule.

## Glossary

- **Group**: a named set of members who share expenses; groups are independent of each other.
- **Balance**: a member's net position in a group. Positive means the group owes them money, negative means they owe the group. All balances in a group always sum to 0.
- **Expense**: someone (the payer) paid an amount on behalf of some members (participants), who now owe their share.
- **Share**: the part of an expense a participant owes.
- **Settlement**: a real payment between two members that reduces the debt (it is not an expense).
- **Square**: balance exactly 0.

## Requirements

1. `createGroup(group, members)` needs at least one member and no duplicates; the group name must be new. `addMember(group, member)` adds a new member (duplicates are errors).
2. `removeMember(group, member)` succeeds only if the member's balance is 0; otherwise it returns false and keeps the member. A removed member can no longer be used in expenses.
3. `addExpense(group, payer, amount, participants)`: the payer paid `amount` for the participants, split equally. When the amount does not divide evenly, the remainder is handed out one unit at a time to participants in ascending name order. The payer need not be a participant.
4. `addExactExpense(group, payer, shares)`: the payer paid the sum of the shares and each listed member owes exactly their share. Each share must be > 0.
5. `settle(group, from, to, amount)`: `from` paid `to` to reduce a debt (overpaying is allowed). The amount must be positive and `from != to`.
6. `balances(group)` is the net position per member (positive = is owed money, negative = owes), with zero balances omitted. `totalSpent(group)` is the sum of all expenses (settlements are not expenses).
7. `settleUp(group)` returns payments formatted `from>to:amount` that bring every balance to zero using the FEWEST payments possible. Any optimal set is accepted; the list is sorted ascending; it is empty when everyone is square.
8. Any call with an unknown group or member, a duplicate, or an invalid amount fails and changes nothing (no partial updates).

## Constraints

- Many requests arrive at once (Java: `Must be thread-safe`; Rust: `&self`, `Send + Sync`). An expense is applied to all balances atomically; `balances` never shows an expense half applied.
- 4,000 concurrent expenses must neither lose any amount nor change the zero-sum property (the sum of all balances is always 0).
- Amounts are whole units held as `long`/`i64`; never use floating point. Overflow handling beyond `long` is out of scope.
- Splitting is deterministic: the same expense always charges the same members the same amounts (remainder by ascending name, not by input order).
- `settleUp` must be exact: it handles up to 12 people with non-zero balance in the tests and 20 in the hidden test (within 10 s); a greedy matcher is not accepted because it can use more payments than necessary.

## Examples

```
Equal split with remainder
createGroup("flat", [ann, bob, cy])
addExpense("flat", payer=ann, 100, [cy, ann, bob])   -> shares by name order: ann 34, bob 33, cy 33
balances("flat")   -> {ann: +66, bob: -33, cy: -33}   (ann paid 100, owes 34 herself)
totalSpent("flat") -> 100

Exact shares, payer outside the split
addExactExpense("g", payer=ann, {bob: 30, cy: 70}) -> balances {ann: +100, bob: -30, cy: -70}

Settlements
settle("flat", from=bob, to=ann, 20)   -> {ann: +46, bob: -13, cy: -33}   totalSpent still 100
settleUp("flat") -> ["bob>ann:13", "cy>ann:33"]    (payments sorted ascending; "from>to:amount")
After both are paid, balances("flat") -> {} and settleUp("flat") -> []

Fewest payments (greedy is not enough)
balances {a:+2, b:+3, c:-4, d:-5, e:+4}
greedy matching needs 4 payments; the optimum is 3: ["c>e:4", "d>a:2", "d>b:3"]

Leaving a group
createGroup("g", [ann, bob, cy]); addExpense("g", ann, 90, [ann, bob])   -> ann +45, bob -45
removeMember("g", bob) -> false (owes 45)     removeMember("g", cy) -> true (never took part)
addExpense("g", ann, 10, [cy])                -> error (cy was removed)
settle("g", from=bob, to=ann, 45); removeMember("g", bob) -> true; removeMember("g", ann) -> true
```

## Edge cases

- An expense where the payer is also a participant: the payer's balance goes up by the whole amount and down by their own share (net `amount - share`).
- Participant order in the input never matters; the remainder always goes to the smallest names first (`10` over `[d, a, c, b]` -> a 3, b 3, c 2, d 2).
- `balances` omits members whose balance is 0, so an empty group (or one that is fully settled) returns an empty map. `totalSpent` does not change on `settle`.
- Rejected, with nothing changed (including `totalSpent`): unknown group or member (payer, participant, `from`, `to`), a duplicate in a list, `amount <= 0`, a share `<= 0`, an empty participant list, `from == to`, creating a group that exists or with no members, adding an existing member.
- Overpaying in `settle` is allowed: it just flips the sign of the balances involved.
- `settleUp` does not change any balance; it only reports what would have to be paid. If everyone is square it returns an empty list.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `ExpenseGroups`; invalid input throws `IllegalArgumentException`; `balances` returns `Map<String, Long>`, `settleUp` returns `List<String>`.
- Rust: `Res<T> = Result<T, GroupError>` with `UnknownGroup`, `UnknownMember`, `AlreadyExists` (duplicate group or member), `InvalidInput` (no members/participants, amount or share <= 0, duplicates in a list, paying yourself); `balances` is a `BTreeMap<String, i64>`; shares are `&[(&str, i64)]`.
- Remainder rule: with amount A and n participants (sorted by name), each pays `A / n` and the first `A % n` pay one more. 100 over 3 -> 34, 33, 33; 10 over `a,b,c,d` -> 3, 3, 2, 2.
- The payer's balance increases by the whole amount (even if they are also a participant, their own share is subtracted separately), each participant's balance decreases by their share.
- `settle(from, to, amount)` increases `from`'s balance by `amount` and decreases `to`'s: it moves both toward zero when `from` owed `to`.
- Validation failures leave everything unchanged, including `totalSpent`; the tests check that after a series of rejected calls `balances` is empty and `totalSpent` is 0.
- A zero share, zero or negative amount, empty participant list, a participant listed twice, `settle` with a negative amount or with `from == to` are invalid.
- A removed member (balance 0) cannot be used as payer or participant afterwards; removing a member is rejected (false) while they have a non-zero balance. Members that owe nothing but never took part can be removed.
- `settleUp` payments have positive amounts and, applied to the balances, bring every balance to exactly zero. The count must equal the minimum possible (the tests compute it independently).

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.List;
import java.util.Map;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Amounts are whole currency units. Every method throws IllegalArgumentException for an unknown group, an unknown
 * member, a duplicate, or an invalid amount, and then changes nothing.
 */
public interface ExpenseGroups {

    /** Creates a group with at least one distinct member. */
    void createGroup(String group, List<String> members);

    void addMember(String group, String member);

    /** Removes a member only if their balance in the group is zero. Returns false (and keeps them) otherwise. */
    boolean removeMember(String group, String member);

    /**
     * {@code payer} paid {@code amount} for everyone in {@code participants} equally. A remainder is handed out one
     * unit at a time to the participants in ascending name order. The payer need not be a participant.
     */
    void addExpense(String group, String payer, long amount, List<String> participants);

    /** {@code payer} paid the sum of the shares; each member owes exactly their share. Shares must be > 0. */
    void addExactExpense(String group, String payer, Map<String, Long> shares);

    /** {@code from} paid {@code to} {@code amount} to reduce what they owe. (Overpaying is allowed.) */
    void settle(String group, String from, String to, long amount);

    /** Net position per member (positive = is owed money); zero balances omitted. */
    Map<String, Long> balances(String group);

    /** Sum of all expenses (not settlements) of the group. */
    long totalSpent(String group);

    /**
     * A list of payments "from>to:amount" that brings every balance to zero using the FEWEST payments possible.
     * Any optimal set is accepted; the list is sorted ascending. Empty if everybody is square.
     */
    List<String> settleUp(String group);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static ExpenseGroups create()
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Time-dependent behaviour uses an injectable fake clock, never real sleeping. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Equal split with remainder in name order (input order scrambled) and `totalSpent`.
- Exact shares with a payer outside the split.
- Validation: duplicate group, empty/duplicated members, unknown group/member, zero amount, empty or duplicated participants, zero share, negative settle, settle to self - and nothing changes.
- Separate groups with the same members stay separate; settlements reduce debt and do not count as spending.
- Remove member only when square; a removed member cannot be used.
- `settleUp`: nothing when square, one payment for a pair, three payments (not four) for the greedy trap, and the optimum on 20 random 12-person groups (verified by applying the payments and comparing the count with an independent bitmask solver).
- Concurrency: 8 threads x 500 expenses (4,000 expenses with exact expected balances and sum 0); a 100x repeat of concurrent expenses and settlements.
- More edge cases (20 people within 10 seconds, five independent pairs need exactly five payments) are in `interviewer/`.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/87-expense-groups test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`FakeClock`, `Candidate`): `java/testkit`.
