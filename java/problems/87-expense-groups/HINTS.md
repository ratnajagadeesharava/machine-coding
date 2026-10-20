# Hints — 87 Expense Groups (Splitwise) (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

If you only remember who owes whom after every expense, what single number per person is enough to answer 'how much is this person owed overall'?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Keep one net balance per member and update all affected balances in one atomic step after validating the whole request. For minimising payments, forget who owes whom and look only at the people with a non-zero net balance: which groups of them already add up to zero?

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: a group is `members + balances (map) + total`; each operation validates everything first, then mutates under the group's lock. For `settleUp`, a set of people whose balances sum to zero can settle among themselves with one payment fewer than its size, so the minimum equals the number of people with a balance minus the largest number of disjoint zero-sum subsets. Search over subsets of the (at most ~20) non-zero people with a bitmask, then turn each zero-sum subset into payments from debtors to creditors.

**Java:** Use `long` everywhere and `Math.floorMod` only if you ever see negatives; sort participants with `Collections.sort`/`TreeMap` before the remainder. A per-group lock (`ConcurrentHashMap<String, Group>` plus `synchronized (group)`) is enough.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
