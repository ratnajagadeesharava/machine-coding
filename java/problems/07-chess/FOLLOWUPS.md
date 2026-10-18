# Follow-ups — 07 Chess

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Undo / redo
Take back moves, redo them, branch into a new line. Which parts of your model make undo cheap, and which make it painful?

### V3 — Move history and notation
Full move list, standard algebraic notation output (`Nf3`, `exd5`, `O-O`, `e8=Q+`), export/import of a game.

### V4 — Draw detection
Threefold repetition, fifty-move rule, insufficient material, draw offers/resignation. What extra state must you carry, and where does it live?

### V5 — Many concurrent games
A server hosts thousands of games, each with two remote players and clocks. Where is the boundary between the pure rules engine and the game session?

### V6 (stretch) — Search
A simple engine opponent (depth-limited minimax + alpha-beta). Does your move generation make this fast or painful?
