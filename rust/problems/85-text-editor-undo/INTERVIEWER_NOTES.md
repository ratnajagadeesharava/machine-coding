# Interviewer notes — 85 Text Editor with Undo/Redo

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Empty `type` between two non-empty types: does not end or create a group (it does nothing).
- Type over a selection, then more typing: the first is its own step, later plain typing may group among themselves.
- Backspace at position 0 creates no step; backspace with a selection removes the whole selection.
- `replaceAll` returning 0 leaves the undo stack alone; a non-zero replace clears redo and clamps the cursor.
- `replaceAll` with overlapping occurrences (`aaa`, `aa`) counts non-overlapping left to right.
- Undo with the cursor in the middle of the text restores the cursor where it was, not at the end.
- A new edit after undo clears the redo history.
- A long typing group (20,000 characters) is still a single undo step and stays cheap.

## Likely interviewer questions
- Command objects with undo(), or snapshots of the whole text? What does each cost in memory and in code?
- Where do you decide that two keystrokes belong to the same step? What state does that decision need?
- What is stored in a step for cut? For replaceAll? Could one step type cover all of them?
- What exactly do you restore on undo: text, cursor, selection? Why is the selection cleared?
- Why must a new edit clear the redo stack? What bug appears if it does not?

## Invariants
- `0 <= cursor <= text.length` always; a selection, if present, lies within the text.
- undo followed by redo returns exactly the state before the undo (text and cursor).
- After any new edit the redo stack is empty.
- The number of undo steps equals the number of groups, not the number of calls.

## Common mistakes
- One undo step per `type` call (no grouping), or grouping everything including cursor moves and selections.
- Letting `moveCursor`/`select`/`copy` create undo steps, or not ending the group on them.
- Not ending the group on `undo`/`redo`, so typing after an undo merges into the old group.
- Restoring only the text and not the cursor.
- Forgetting to clear the redo stack on a new edit, or clearing it on `undo`.
- Counting overlapping matches in `replaceAll`, or replacing in a loop that re-scans replaced text.
- A step that stores the entire text for every keystroke, making 20,000 characters quadratic.

## Over-engineering traps
- A generic `Command` hierarchy with a macro/composite command for the group before simple steps pass.
- A rope/gap buffer on day one; a `StringBuilder` is fine at this size.
- Persistent (immutable) data structures for the whole history when two stacks suffice.

## Design and scaling discussion
- Memento (snapshot) vs command (inverse operation): snapshots are trivial and O(n) per edit; commands are compact but each operation needs a correct inverse.
- Grouping is a UX policy, so keep it isolated (a coalesce predicate) and test it separately from the editing mechanics.
- A bounded history needs a ring or deque; dropping the oldest step is safe because later steps do not depend on it for the current state.
- For collaborative editing undo becomes 'undo my last operation, transformed against everything that happened after it'.
- Large buffers: gap buffer for local edits, piece table for cheap undo, rope for large insertions and random access.

## Trade-off questions to ask when the candidate is done
- How much memory does your history use after 20,000 typed characters? After 20,000 single-character cursor-separated edits?
- Where does the 'open group' state live and what resets it?
- Would time-based grouping change your design? Where would the clock go?
- How would you test undo/redo exhaustively (a model-based random test: undo returns every earlier text)?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.
