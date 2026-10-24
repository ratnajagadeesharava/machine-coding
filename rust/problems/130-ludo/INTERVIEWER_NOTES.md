# Interviewer notes — 130 Ludo

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- One move capturing two different colours that stand alone on the same cell (`captured:RED:1,captured:YELLOW:1`, in turn order).
- A 6 with no legal move (all tokens in the home column or HOME) still gives the extra turn; a non-6 with no legal move passes.
- A capture made with a 6 counts towards the three-sixes limit: 6 (capture), 6, 6 forfeits the third one even though the first came with a capture.
- A BLUE token capturing after the track wraps past cell 51 (cells are computed modulo 52 from start cell 39).
- The game ends the instant the fourth token reaches HOME: no extra turn, no more calls accepted even though dice are left; `current_player()` stays on the winner.
- Five long pseudo-random games (2, 3 and 4 players, three move policies) whose end state was computed by an independent implementation: any off-by-one in a rare corner (for example progress 51, a block next to a safe cell, a captured token that re-enters) shows up here.

## Likely interviewer questions
- Where does the state "waiting for roll / waiting for move / over" live? What happens if `move` is called in the wrong state?
- How do you compute the cell of a token? What happens at progress 51 versus 52?
- Why is a capture decided per opponent colour rather than per token?
- When exactly does the count of sixes reset? What if the player rolls 6, captures, rolls 6, rolls 6?
- What does `legal_moves` look like for a roll of 6 when all four tokens are in different places?
- How would you add a bot? Where does the choice happen?

## Invariants
- Every token's progress is in 0..57; only progress 1..51 maps to a shared cell.
- At most one roll is pending and only for the current player; a pending roll always has at least one legal token.
- The dice position advances by exactly one per successful roll and never otherwise.
- After the game is over nothing can change.
- A failed call changes nothing.

## Common mistakes
- Treating the track as having 52 steps per lap: progress 51 is the last shared cell, the home column starts at 52.
- Computing the cell with `(start + progress) % 52` (off by one) or forgetting that a token on its start cell has progress 1.
- Allowing overshoot of HOME, or requiring a 6 to enter the home column.
- Capturing on safe cells, capturing blocks, or capturing own tokens.
- Capturing in the home column (progress 52+) because the "cell" number collides with a shared cell.
- Giving the extra turn for a capture only, forgetting HOME, or not giving it for a 6 that could not move.
- Resetting the six counter when an extra turn begins (instead of when a non-6 is rolled or the turn passes), or never resetting it.
- Forfeiting the third six but still asking for a move, or consuming the move choice.
- Passing the turn after a forfeited six to the same player, or advancing twice.
- Not consuming the die when the roll is auto-passed, or consuming it when a rejected `roll()` fails.
- Not sorting the list returned by `legal_moves`.
- Leaving a captured token's old cell occupied in a cell-indexed structure.

## Over-engineering traps
- A full class hierarchy of cell types (start cell, safe cell, home cell...) when a cell is a number and "safe" a lookup.
- A per-token object graph with back-references to the board and the player, then trouble keeping them in sync.
- An event bus, observers and command objects before undo or a UI exists.
- A rules trait with one implementation, before the house-rules follow-up is asked.

## Patterns and data structures an interviewer hopes to hear (only if the candidate raises them or in the discussion)
- **State**: the game is in one of three states (awaiting a roll, awaiting a move, over). Each state accepts different calls, and transitions are explicit. A boolean `rolled` plus scattered ifs is the weak version; an enum or a small state hierarchy that owns "what is allowed now" is the strong one. Worth discussing only if it simplifies the invalid-call checks, not as decoration.
- **Strategy**: how a token is chosen (bot players), how the next player is determined, and house rules as pluggable decisions (is capture mandatory, how many sixes forfeit). The base engine asks "which token?" through one seam so that a human, a bot or a test script can answer.
- **Command / Memento**: undo as stored moves (reverse them) or as snapshots of the small mutable state (tokens, current seat, six count, dice index). Snapshots are simpler because the state is tiny.
- **Observer**: announcing captures and wins to a UI or a log, without the engine knowing the listener.
- Data structures: `int[4][4]` or a `Token` array for progress; a function `(colour, progress) -> cell` instead of storing cells; for capture checks a scan of at most 12 tokens or a map `cell -> tokens`, both fine; the dice as an index into an immutable list.

## Design and scaling discussion
- Cost of a move is constant (at most 12 tokens to scan); there is nothing to scale, so the discussion is about clarity and extensibility, not performance.
- Track coordinates: relative progress per player is easier to reason about than absolute cells; absolute cells are needed only to compare positions of different colours. Keeping both representations in sync by deriving one from the other avoids bugs.
- Randomness: dice are injected, so the engine is deterministic; ask how production would plug a random source in and how a game would be replayed from a log of dice and choices.
- Concurrency: the game is single-threaded by contract; in a server each game would be serialised (one lock or one actor per game).

## Trade-off questions to ask when the candidate is done
- Store progress or store cells? What does each make easy and hard?
- A state class per game state, or a flag? What did you gain or lose?
- Where would a house rule "capturing is mandatory" go? How many places change?
- How would you replay a game from a log and prove the engine is deterministic?
- How would you test the rule engine without writing out whole games?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with `cargo test --features hidden`. They only use the same public contract.
