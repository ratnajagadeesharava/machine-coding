# Hints — 107 Excel-Like Autofill Engine (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Before generating anything, what do the seeds have to satisfy, and what single number describes how the series moves from one value to the next?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Turn each kind of seed into a number (a decimal for numbers, a rank for letter labels), do the arithmetic on those numbers, and only turn the results back into text at the very end. For labels, think about what is different between ranking `A..Z` and an ordinary base-26 number that has a zero digit.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: a recogniser per kind that either rejects the seeds or produces `(start, step)` plus a way to print a value. Check that all seeds fit the kind, compute the step, verify every consecutive pair, then emit `count` values of `last + k * step`, stopping for labels when the rank drops below 1. A label from a rank repeatedly takes `(rank - 1) % 26` as the last letter and continues with `(rank - 1) / 26`.

**Java:** `BigDecimal` built from the seed text keeps decimals exact; print with `stripTrailingZeros().toPlainString()` and special-case zero. Do not go through `double`.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
