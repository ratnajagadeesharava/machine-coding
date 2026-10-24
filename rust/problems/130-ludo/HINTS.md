# Hints — 130 Ludo (Rust)

Try the problem for at least 20 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Make a list of the situations the game can be in when a caller calls `roll()` or `move_token()` (is anything pending? is the game finished?), and decide what each call does in each of them. Separately, decide which single number describes where a token is, and where the conversion to a board cell happens.

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

A small state machine (waiting for a roll, waiting for a move, over) keeps the invalid-call checks in one place. Keep each token's progress relative to its own colour, and derive the absolute cell only when two colours must be compared. The question "which tokens can move for this roll?" is asked twice (to decide whether a choice is needed and to validate the choice), so give it one home. Deciding who plays next is one small function of: the value rolled, whether a capture happened, whether a token reached HOME, and the number of sixes in a row. If you add bots or house rules later, the place that picks a token is the seam for a strategy.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `Game { players, dice (immutable) + position, progress[player][4], current, pendingRoll (0 = none), sixesInARow, winner }`. `roll`: reject unless awaiting a roll and dice remain, read the die, update the six counter, forfeit on the third six, otherwise compute the legal tokens; if there are none, finish the turn right away (a 6 keeps the turn), else remember the roll. `move`: validate against the legal tokens, update the progress, if the new progress is 1..51 and the cell is not safe look at each other colour: exactly one token of it on the cell means back to the yard; then decide extra turn (6, capture, HOME) or advance to the next colour and reset the six counter; check the win.

**Rust:** an enum for the turn state, `[[u8; 4]; 4]` for progress, and a `Vec<u8>` plus an index for the dice; return errors with a `Result` and make the failing checks happen before any mutation.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.
