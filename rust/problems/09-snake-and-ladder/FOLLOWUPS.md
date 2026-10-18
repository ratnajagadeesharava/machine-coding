# Follow-ups — 09 Snake and Ladder

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Extra turns
Rolling a 6 grants another turn (but three 6s in a row cancel the move). How does your turn model absorb this?

### V3 — Different dice and rules
Two dice, a custom dice (e.g. 1..4), "must roll a 6 to enter", "bounce back from the end" instead of staying put. Which rules are configuration, which are code?

### V4 — Captures
If a player lands on a square occupied by another player, that player returns to the start. What must you index to make this cheap?

### V5 — Many games and spectators
A server hosts many games; spectators subscribe to move events. Where do events come from?
