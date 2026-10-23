# Hints — 114 Elevator Request Feasibility (Rust)

Try the problem for at least 25 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Work `r1 = 0->9`, `r2 = 3->5` with a limit of 1 by hand on paper. What did you have to know about the *whole future* of the lift to say "no", and could a single number per rider have told you?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

The circuit is deterministic, so you can run the future: take the promised riders plus the newcomer, play the lift forward stop by stop on a copy, and look at what happened to everybody. Keep, per rider, whether they are inside and how many stops they have sat through. Use the very same movement code for `step()` and for the simulation so `plan()` and `step()` cannot disagree.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `Lift { position on the circuit, riders: [{id, from, to, persons, boarded, stopsSoFar}] }`. One function `nextStop(position, riders)` finds the first position at or after the current one where someone leaves or a matching rider waits; applying a stop does: increment `stopsSoFar` for riders who stay inside, remove those who leave (checking their limit), board the matching waiting ones (checking capacity). `request` = copy, add newcomer, run applyStop until nobody is left; accept only if no check failed, then commit the copy's *input* (the new rider list) to the live state. Think in a ring of `2*(floors-1)` positions so the turn floors are visited once.

**Rust:** one `Mutex<State>` around the whole check-and-record is fine here; simulate on a `clone()` of the rider vector (or a small copy of the fields you mutate) so the live state is never half-changed.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
