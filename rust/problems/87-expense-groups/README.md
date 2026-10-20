# 87 · Expense Groups (Splitwise) (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 75 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Build the backend of a Splitwise-like app for flatmates and travel groups. People share expenses: one person pays the
restaurant bill for everyone, another pays the rent for two. The app tracks who owes whom, shows each person's net balance, lets
people pay each other back, and at the end of the trip computes the smallest number of payments that makes everybody square.
Money is whole currency units, so a bill of 100 split three ways cannot be 33.33 each: it has to come out to exactly 100
and be reproducible.

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
group flat = [ann, bob, cy]
addExpense(flat, payer=ann, 100, [cy, ann, bob])    -> shares in name order: ann 34, bob 33, cy 33
balances(flat) -> {ann: +66, bob: -33, cy: -33}      totalSpent -> 100
settle(flat, from=bob, to=ann, 20)                   -> {ann: +46, bob: -13, cy: -33}
settleUp(flat) -> ["bob>ann:13", "cy>ann:33"]

balances {a:+2, b:+3, c:-4, d:-5, e:+4}   (sum 0)
greedy matching takes 4 payments; the optimum is 3: ["c>e:4", "d>a:2", "d>b:3"]
```

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

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Amounts are whole currency units. A failing call changes nothing.

use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq)]
pub enum GroupError {
    UnknownGroup,
    UnknownMember,
    /// duplicate group name or duplicate member
    AlreadyExists,
    /// no members/participants, amount or share <= 0, duplicates in a list, paying yourself
    InvalidInput,
}

pub type Res<T> = Result<T, GroupError>;

pub trait ExpenseGroups: Send + Sync {
    /// Creates a group with at least one distinct member.
    fn create_group(&self, group: &str, members: &[&str]) -> Res<()>;

    fn add_member(&self, group: &str, member: &str) -> Res<()>;

    /// Removes a member only if their balance in the group is zero. Returns `Ok(false)` (and keeps them) otherwise.
    fn remove_member(&self, group: &str, member: &str) -> Res<bool>;

    /// `payer` paid `amount` for everyone in `participants` equally. A remainder is handed out one unit at a time
    /// to the participants in ascending name order. The payer need not be a participant.
    fn add_expense(&self, group: &str, payer: &str, amount: i64, participants: &[&str]) -> Res<()>;

    /// `payer` paid the sum of the shares; each member owes exactly their share. Shares must be > 0.
    fn add_exact_expense(&self, group: &str, payer: &str, shares: &[(&str, i64)]) -> Res<()>;

    /// `from` paid `to` `amount` to reduce what they owe. (Overpaying is allowed.)
    fn settle(&self, group: &str, from: &str, to: &str, amount: i64) -> Res<()>;

    /// Net position per member (positive = is owed money); zero balances omitted.
    fn balances(&self, group: &str) -> Res<BTreeMap<String, i64>>;

    /// Sum of all expenses (not settlements) of the group.
    fn total_spent(&self, group: &str) -> Res<i64>;

    /// A list of payments "from>to:amount" that brings every balance to zero using the FEWEST payments possible.
    /// Any optimal set is accepted; the list is sorted ascending. Empty if everybody is square.
    fn settle_up(&self, group: &str) -> Res<Vec<String>>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create() -> Option<Arc<dyn ExpenseGroups>> { ... }
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

## Language notes

Use `i64`, `BTreeMap<String, i64>` for balances (the contract returns one, and it iterates in sorted order), and a `Mutex` around the groups map or one `Mutex` per group. Validate before mutating, so a failing call leaves nothing behind.

## Running

```bash
cargo test -p p87_expense_groups
STRICT=1 cargo test -p p87_expense_groups   # fail instead of skip while unimplemented
cargo test -p p87_expense_groups --features hidden   # interviewer scenarios
cargo play run 87              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`FakeClock`, `sut!`): `rust/support`.
