# 06 · ATM (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 60 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

**What you are building.** The software of a cash machine (ATM). A customer identifies themselves with a **card id** and a **PIN**; after three wrong tries in a row the card is locked. Once logged in they have a **session** in which they can check their balance, deposit money and withdraw cash. The machine physically holds a limited stock of banknotes (10, 20, 50 and 100), so a withdrawal is only possible if the exact amount can be paid from the notes currently loaded, and the machine should hand out as few notes as possible. A withdrawal must either fully succeed (account debited and notes dispensed) or change nothing.

## Requirements

1. Accounts are identified by **card id** and protected by a **PIN** (a string); each has a balance (in whole currency units, no fractions). Accounts are created with `addAccount(cardId, pin, balance)`.
2. The machine holds notes in denominations **10, 20, 50 and 100**; the operator loads them with `loadCash(denomination, count)` (loads accumulate; the machine starts with no notes).
3. `authenticate(card, pin)` starts a **session** and returns its token. Three consecutive wrong PINs **lock** the card: it can no longer authenticate, even with the right PIN. A correct PIN resets the failure count.
4. Within a session the customer can query the **balance**, **withdraw**, and **deposit**. After `endSession` the token is invalid. Each `authenticate` creates a new session with its own withdrawal total starting at 0.
5. A withdrawal amount must be a positive multiple of 10, not exceed the account balance, and be payable **exactly** from the notes currently in the machine, using the **fewest notes** possible.
6. A session may withdraw at most **500** in total (the sum of its successful withdrawals; exactly 500 is allowed, 510 is not).
7. A withdrawal is atomic: either the account is debited **and** the notes leave the machine, or nothing changes.
8. A deposit must be positive and increases the account balance. (Deposited cash does not become dispensable in this version.)

## Constraints

- Notes are limited: choosing the fewest notes is a **bounded** problem, and the greedy choice can fail (see the example).
- All failures are plain refusals (empty/false); no exceptions are required by the contract.

## Examples

**1. Authentication and locking** (card `card-1`, PIN `1234`)

```
authenticate("card-1", "0000") -> empty        (failure 1)
authenticate("card-1", "1234") -> session S1    (success; failure count back to 0)
authenticate("card-1", "1") ; ("card-1", "2") ; ("card-1", "3") -> all empty
authenticate("card-1", "1234") -> empty        (3 consecutive wrong PINs: locked for good)
authenticate("nope", "1234")   -> empty        (unknown card)
```
Two wrong PINs, then the right one, then two more wrong PINs does **not** lock the card (the right PIN reset the count).

**2. Withdrawal with fewest notes** (balance 2000; machine holds 10 each of 100, 50, 20, 10 = 1800 total)

```
withdraw(S, 180) -> {100: 1, 50: 1, 20: 1, 10: 1}   (4 notes)
balance(S)       = 1820        cashInMachine() = 1620
withdraw(S, 15)  -> empty     (not a multiple of 10)
withdraw(S, 0) , withdraw(S, -10) -> empty
```

**3. Greedy fails, exact payment must still be found** (machine holds 50 x1 and 20 x3)

```
withdraw(60) -> {20: 3}      (taking the 50 first would leave 10, which cannot be paid)
withdraw(30) -> empty       (machine holding only 50s cannot pay 30; balance and cash unchanged)
```

**4. Session limit of 500** (balance 2000, plenty of notes)

```
withdraw(300) -> notes      total 300
withdraw(300) -> empty     (would make 600)
withdraw(200) -> notes      total exactly 500
withdraw(10)  -> empty
endSession(S); authenticate again -> new session; withdraw(500) -> notes  (fresh limit)
```

**5. Other refusals**

```
account balance 50, machine has notes: withdraw(60) -> empty; balance stays 50
deposit(S, 250) -> true  (balance 2000 -> 2250);  deposit(S, 0) , deposit(S, -5) -> false
after endSession(S): balance(S) -> empty, withdraw(S, 10) -> empty, deposit(S, 10) -> false
balance("bogus") -> empty
```

## Edge cases

- A refused withdrawal changes nothing: balance, machine notes and the session's withdrawn total are unchanged (refused attempts do not count toward the 500).
- Withdrawing exactly the account balance is allowed; one more unit than the balance is refused.
- A withdrawal is checked against what the machine can pay *now*; a later `loadCash` can make a previously refused amount possible.
- Deposits only change the account; they add no notes to the machine, so `cashInMachine()` is unchanged by a deposit.

## Glossary

- **Card id**: identifies the account (like the card number).
- **Session**: the logged-in period between `authenticate` and `endSession`, identified by a token string.
- **Lock**: the permanent refusal to authenticate a card after three consecutive wrong PINs.
- **Denomination**: the face value of a note (10, 20, 50 or 100).
- **Fewest notes**: among all exact ways to pay the amount from the notes in the machine, one with the smallest total number of notes.

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `authenticate` returns a non-empty session token on success; empty for unknown cards, wrong PINs, and locked cards. Locking happens on the third consecutive wrong PIN.
- `balance(session)`, `withdraw`, `deposit` on an unknown or ended session are refused (`empty`/`false`).
- `withdraw` returns the notes dispensed as a map `denomination → count` whose total equals the amount, or empty when refused (invalid amount, insufficient balance, session limit exceeded, machine cannot pay exactly).
- The session limit counts only successful withdrawals.
- `cashInMachine()` is the total value of notes currently in the machine.
- `deposit` returns `true` when accepted; `balance` returns the current account balance in whole units.
- Wrong-PIN attempts count per card across sessions; only a correct PIN resets the count.
- `cashInMachine()` is the sum of `denomination x count` of the notes currently loaded, decreased only by successful withdrawals.

## Test entry point (Java)

The tests drive your system through the thin, primitive-typed contract below. It is **not** your domain
model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in
`solution/Solution.java`.

```java
package contract;

import java.util.Map;
import java.util.Optional;
import java.util.OptionalLong;

/** Thin test entry point. Your domain model lives behind it. */
public interface AtmApi {

    void addAccount(String cardId, String pin, long balance);

    /** Adds notes of the given denomination (10, 20, 50 or 100) to the machine. */
    void loadCash(int denomination, int count);

    /** Returns a session token, or empty (unknown card, wrong PIN, locked card). */
    Optional<String> authenticate(String cardId, String pin);

    OptionalLong balance(String session);

    /** Notes dispensed (denomination to count), or empty if refused. */
    Optional<Map<Integer, Integer>> withdraw(String session, int amount);

    boolean deposit(String session, long amount);

    void endSession(String session);

    long cashInMachine();
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    AtmApi create()
}
```

## Language notes

`Optional<Map<Integer,Integer>>` is the dispensed notes. Use whatever internal representation you like for notes.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/06-atm test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.
