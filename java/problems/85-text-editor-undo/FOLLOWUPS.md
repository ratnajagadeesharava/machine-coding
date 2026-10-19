# Follow-ups — 85 Text Editor with Undo/Redo

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Time-based grouping
Typing is grouped only if the keystrokes come within 1 second of each other (injectable clock), and a pause ends the group. How does the clock enter the design without becoming global state?

### V3 — Bounded history
Keep at most 100 undo steps and at most 5 MB of history. Which steps are dropped, and what do you do with a single step that is bigger than the whole budget?

### V4 — Multiple cursors and documents
Add multiple selections that are edited together and several open documents with independent histories plus a shared clipboard. What becomes the unit of undo for a multi-cursor edit?

### V5 — Collaborative editing
Two users edit the same document and each must be able to undo only their own changes. Why does a linear undo stack stop working, and what does an operational-transform or CRDT approach change about what an undo step is?

### V6 — Large documents
The buffer is 100 MB. Replace the string with a gap buffer, rope or piece table, and make undo steps store diffs, not text. Which operations get cheaper and which get more complicated?
