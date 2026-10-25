# Follow-ups — 130 Ludo

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Variable dice
Players may be given a different kind of die: a die with another number of faces, or two dice per roll where the player picks which value to play (and may play both on two different tokens). Which of your rules hard-coded "6" or "1..6", and which of them were really "the maximum face"? What does `legalMoves` return when a roll is two values? Decide what "an extra turn on a 6" means now and write it down.

### V3 — Teams
In a 4-player game RED+YELLOW play against GREEN+BLUE. Partners never capture each other, partners' tokens on one cell count together for block protection, and a team wins when all eight of its tokens are HOME (once a player's four tokens are HOME, that player's turns are used to move the partner's tokens). What changes in capture, in blocks, in turn order and in winner detection? Which of those was a single place in your code?

### V4 — Undo
Add `undo()` that takes back the last `move` (and the roll that preceded it), restoring tokens, the turn, the count of sixes and the dice position; repeated undo goes back further. What is the smallest state you must remember per step, and how does a forfeited six or an automatically passed roll fit in? Is a replayed game after undo identical to one that never made the mistake?

### V5 — Bot players
Some seats are played by a bot that chooses a token on its own when it is its turn. Offer at least two strategies (for example "furthest token first" and "capture if possible, otherwise leave the yard, otherwise furthest") and let the game run until a human seat has to choose. How do you keep the rules engine unaware of who is choosing? How do you test a strategy without the whole game?

### V6 — House rules
Games can be created with options: a six is required to enter HOME as well as to leave the yard; capturing is mandatory when possible; blocks of two stop opponents from passing; a different set of safe cells; the number of sixes that forfeit a turn (two, three or never). Where do such options live so the base rules stay readable, and how do you test combinations?
