# 67 · Command (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 50 min | Level 2 — `TextBuffer`, `Command` and `EditError` provided; write `InsertCommand`, `DeleteCommand`, `MacroCommand` and `CommandHistory`. |

## Problem

A tiny text editor needs undo/redo and recordable macros. Every edit is a command with `execute` and `undo`.

**What you are building:** the editing core of a small text editor. Each edit (insert text, delete a range) is an
object that can be run and reversed. Several edits can be bundled into one macro that behaves as a single edit,
and a `CommandHistory` lets the user step backwards (undo) and forwards (redo) through what they did, exactly like
Ctrl+Z / Ctrl+Y. The history must work with any command, including ones it has never seen.

### Provided types

- `TextBuffer` — the text being edited. `new()`, `text()`, `len()` (in characters), `is_empty()`, `insert(pos, s) -> Result<(), EditError>` (valid `pos` is `0..=len`) and `delete(from, to) -> Result<String, EditError>` which removes characters `[from, to)` (needs `from <= to <= len`) and returns them. Invalid arguments return `Err(EditError::OutOfRange)` and change nothing.
- `Command` — `execute(&mut self, buffer: &mut TextBuffer) -> Result<(), EditError>` (on `Err` it must have changed nothing) and `undo(&mut self, buffer: &mut TextBuffer)` which reverses the most recent successful `execute`. The buffer is passed in on each call.

### Glossary

- **Range `[from, to)`**: includes position `from`, excludes position `to`. `delete(5, 12)` on `"Hello, world"` removes `", world"`.
- **Position**: a character index, not a byte index.
- **Undo list / redo list**: commands that can be undone / re-done. Undoing moves a command from the first to the second.
- **Macro**: several commands that act as one.

## Requirements

1. `InsertCommand::new(pos, text)` inserts at that **character** position on `execute`; `undo` removes exactly that text. A bad `pos` makes `execute` return `Err(EditError::OutOfRange)` and nothing changes.
2. `DeleteCommand::new(from, to)` deletes `[from, to)` on `execute`; `undo` puts the removed text back at `from`. A bad range makes `execute` return `Err(EditError::OutOfRange)` and nothing changes. What is removed is determined when `execute` runs, so `undo` restores whatever was actually removed.
3. `MacroCommand::new(commands)`: `execute` runs them in list order and is **all-or-nothing** (if one fails, the ones that already ran are undone in reverse order, leaving the buffer as before, and the same error is returned); `undo` undoes them in reverse order.
4. `CommandHistory` (`CommandHistory::new()`):
   - `execute(cmd, buffer)` runs it; on `Ok` it becomes undoable and the redo list is **cleared**; on `Err` the history is unchanged and the error is returned.
   - `undo(buffer)` undoes the most recently executed (or re-done) command and returns `true`; with nothing to undo it returns `false`. `redo(buffer)` re-applies the most recently undone command by calling its `execute` again and returns `true`; `false` when there is nothing to redo. `can_undo()` / `can_redo()` report whether the corresponding call would return `true`.
   - It works with any `Command` implementation, not only the ones in this problem; it only calls `execute` / `undo` on them.

## Examples

Insert / delete:

```
buffer "Hello"        InsertCommand::new(5, ", world").execute -> "Hello, world"; undo -> "Hello"
buffer "Hello, world" DeleteCommand::new(5, 12).execute        -> "Hello";        undo -> "Hello, world"
```

Macro on `"abc"` = `[Insert(3,"def"), Delete(0,1), Insert(0,"X")]`:

```
execute: "abc" -> "abcdef" -> "bcdef" -> "Xbcdef"
undo:    "Xbcdef" -> "bcdef" -> "abcdef" -> "abc"
```

Failing macro `[Insert(3,"def"), Delete(0,99), Insert(0,"X")]` on `"abc"`: the insert runs (`"abcdef"`), the delete returns `Err(OutOfRange)`, the insert is undone, the buffer is `"abc"` again and `execute` returns `Err(EditError::OutOfRange)`.

History:

```
execute Insert(0,"one"); execute Insert(3," two"); execute Insert(7," three")   // "one two three"
undo(); undo()  -> "one"          redo() -> "one two"      can_undo() true, can_redo() true

execute Insert(0,"a"); execute Insert(1,"b"); undo()   // "a", can_redo() true
execute Insert(1,"c")                                   // "ac", can_redo() false, redo() returns false
```

A custom command that records `"run"` in `execute` and `"undo"` in `undo`: `execute(cmd); undo(); redo();` yields the log `run, undo, run`.

## Edge cases

- On a fresh history `undo` and `redo` return `false`, `can_undo()` is `false`.
- `history.execute(Box::new(DeleteCommand::new(0, 5)), &mut empty_buffer)` returns `Err(OutOfRange)`, and afterwards `can_undo()` is `false` and the buffer is still empty.

## Running

```bash
cargo test -p p67_command     # from rust/ — tests FAIL until you implement the stubs
cargo play run 67
```
