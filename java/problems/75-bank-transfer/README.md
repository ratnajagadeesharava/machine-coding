# 75 · Bank Transfers Without Deadlock (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 3 — infrastructure provided. Test utilities for threads/clocks exist; business/domain code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Build the core of an in-memory bank: accounts with balances and an atomic `transfer` between two accounts that
is called by thousands of threads at once. Money must never be created, destroyed or temporarily "in flight", an account
must never go negative, and the system must never deadlock - even when A sends to B at the very moment B sends to A.
An auditor thread may ask for the total of all balances at any time; it must always see the true total.

## Requirements

1. `open(account, balance)` creates an account (names are unique, initial balance >= 0).
2. `transfer(from, to, amount)` moves `amount` atomically: either both balances change or neither does.
3. A transfer from an account that has less than `amount` fails (returns `false`) and changes nothing. Transferring exactly the balance is allowed.
4. Invalid requests are rejected without side effects: unknown account, `from == to`, `amount <= 0`.
5. `balance(account)` returns the current balance.
6. `total()` returns the sum of all balances and must **never** observe a half-applied transfer.

## Constraints and thread-safety requirements

- Any number of threads call `transfer`, `balance` and `total` concurrently.
- No deadlock for any pattern of transfers: pairs sending to each other, cycles A->B->C->A, one hot account receiving from everybody.
- Two transfers between disjoint pairs of accounts should not have to wait for each other.
- Throughput: ~16 threads x 10,000 transfers over 20 accounts well within a few seconds.
- Accounts are opened before the concurrent phase; `open` need not be concurrent with transfers (but must not corrupt anything if it is).

## Examples

```
A=100, B=100
thread 1: transfer(A, B, 30)      thread 2: transfer(B, A, 10)     (at the same instant)
-> both succeed, A=80, B=120, total() is 200 at every moment

A=500, 100 threads each: transfer(A, sink, 10)
-> exactly 50 succeed, A=0, sink=500
```

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: invalid requests throw `IllegalArgumentException`. Rust: `Err(BankError::UnknownAccount | AccountExists | InvalidRequest)`.
- `transfer` returns `false` / `Ok(false)` for insufficient funds - not an error.
- `total()` is a consistent snapshot: at any time, `total()` equals the sum of the opening balances (the tests never deposit or withdraw outside of transfers).
- Balances are `long` (Java) / `u64` (Rust). Overflow is out of scope.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface Bank {

    /** Creates an account with an initial balance. IllegalArgumentException if it exists or balance < 0. */
    void open(String account, long balance);

    /** Current balance. IllegalArgumentException for an unknown account. */
    long balance(String account);

    /**
     * Moves money atomically: either both balances change or neither does. Returns false (nothing changes) when
     * {@code from} has less than {@code amount}; a transfer of exactly the balance succeeds.
     * IllegalArgumentException for an unknown account, from == to, or amount <= 0.
     */
    boolean transfer(String from, String to, long amount);

    /** Sum of all balances. Must never include a half-applied transfer (no money "in flight"). */
    long total();
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static Bank create()
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Concurrency tests run many threads against one object, use deadlock timeouts (a hang becomes a failure) and repeat a workload 100 times to catch rare interleavings.

- A <-> B ping-pong by 4 threads (the textbook deadlock) and an 8-account ring where each thread sends to the next.
- 100 threads withdrawing 10 from a 500 account: exactly 50 succeed (no double spend, no lost update).
- 16 threads of random transfers: total conserved, no negative balance.
- An auditor calls `total()` in a tight loop while 6 threads move money: it must never see a wrong total *and* must make progress.
- **Repeat 100x** a small deadlock-prone workload; every run has a deadlock timeout.

## Language notes

`ReentrantLock` per account (or `synchronized` on the account object in id order). `ConcurrentHashMap<String, Account>` for lookup. `System.identityHashCode` is NOT a safe tiebreaker - why?

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/75-bank-transfer test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`Concurrently`, `Eventually`, `Candidate`): `java/testkit`.
