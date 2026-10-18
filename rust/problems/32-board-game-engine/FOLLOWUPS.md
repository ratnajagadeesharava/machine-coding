# Follow-ups — 32 Board Game Engine

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — More games
Reversi (flipping lines of pieces) and checkers (moves with from/to, captures, forced jumps). What in your design stops fitting a "place one piece" model, and how little do you have to change?

### V3 — Undo and replay
Undo the last N moves; export a game as a move list and replay it. Which parts of your state are derived and which are stored?

### V4 — Computer opponent
A pluggable player that picks moves. What does it need from the engine (legal moves, copy of a position, evaluation)?

### V5 — Spectators and tournaments
Observers notified of every move; a tournament runner playing many games between named bots.
