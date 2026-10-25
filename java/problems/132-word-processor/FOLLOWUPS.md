# Follow-ups — 132 Word Processor

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Named styles
Introduce named styles ("Heading 1", "Quote") that define a whole format. A character can reference a style and override a few fields; changing the style changes every character that references it and has not overridden that field. What is stored per character now, and what does `runs()` compare?

### V3 — Tables and nested content
A cell of a table contains its own paragraphs, and a table can sit between two paragraphs. How does position-based editing (`insert(pos, ...)`, `find`) address text inside cells? What does `wordCount` count, and how do you avoid special-casing every operation?

### V4 — Headers and footers
Each section has a header and a footer that repeat on every page, with page-number fields that render differently per page. Where do they live relative to the body, do `find`/`replaceAll` cross into them, and how is a field represented?

### V5 — Undo/redo through commands
Every edit becomes an undoable command, including `replaceAll` (one undo step for all matches) and typing bursts that merge into one step. What does a command have to remember to reverse a delete of formatted text, and how do concurrent editors fit in?

### V6 — Tracked changes and spell-check
Edits can be recorded as suggestions (inserted or deleted text with an author) that are accepted or rejected later, and a background spell-checker marks misspelt words without blocking typing. How do suggestions interact with positions and runs, and how do you keep the checker from racing with edits?
