# Follow-ups — 123 · Snake Game With Food And Score

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Obstacles
The board has fixed obstacle cells given at creation. Hitting one ends the game like a wall. Food and obstacles never overlap in the list, but what happens to the win condition (the board can no longer be "full")? Where does the check for "can this cell be entered?" live so that walls, obstacles and the body are not three copies of the same code?

### V3 — Speed levels
Every 3 foods eaten raise the level by 1, and a move reports how many ticks the next move takes (for example `max(100, 500 - 50*level)` ms). Where does the level live, and how do you test it without a real timer?

### V4 — Foods with different points
The food list carries a point value and a growth amount per entry (a golden food gives 5 points and grows the snake by 3; a poison food shrinks it by 1). What happens to the tail when the snake must grow by 3 over the next moves? What if shrinking takes the snake to length 0? Does `score() == length() - 1` still hold?

### V5 — Wrap-around walls
A board option makes the edges wrap: leaving on the right re-enters on the left. How do you keep the movement rule free of if-ladders for the two wall modes, and how does the reversal and tail-vacating logic stay correct on a board of width 1 or 2?

### V6 — Two snakes
A second snake starts at the bottom-right and both are moved in alternating turns (or simultaneously). Who eats a contested food? What happens on a head-on collision or when one snake runs into the other's tail? How does `move` identify which snake is being moved?
