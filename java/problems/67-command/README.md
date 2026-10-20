# 67 · Command (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 50 min | Level 2 — `TextBuffer` and `Command` provided; write `InsertCommand`, `DeleteCommand`, `MacroCommand` and `CommandHistory`. |

## Problem

A tiny text editor needs undo/redo and recordable macros. Every edit is a command with `execute()` and `undo()`.

**What you are building:** the editing core of a small text editor. Each edit (insert text, delete a range) is an
object that can be run and reversed. Several edits can be bundled into one macro that behaves as a single edit,
and a `CommandHistory` lets the user step backwards (undo) and forwards (redo) through what they did, exactly like
Ctrl+Z / Ctrl+Y. The history must work with any command, including ones it has never seen.

### Provided types

- `TextBuffer` — the text being edited. `text()`, `length()`, `insert(pos, s)` (valid `pos` is `0..length`) and `delete(from, to)` which removes `[from, to)` (needs `0 <= from <= to <= length`) and returns the removed text. Invalid arguments throw `IllegalArgumentException` and change nothing.
- `Command` — `execute()` does the work (if it throws, it must have changed nothing); `undo()` reverses the most recent successful `execute()`.

### Glossary

- **Range `[from, to)`**: includes index `from`, excludes index `to`. `delete(5, 12)` on `"Hello, world"` removes `", world"`.
- **Undo list / redo list**: commands that can be undone / re-done. Undoing moves a command from the first to the second.
- **Macro**: several commands that act as one.

## Requirements

1. `InsertCommand(buffer, pos, text)` inserts `text` at `pos` on `execute()`; `undo()` removes exactly that text. `null` buffer or text → `IllegalArgumentException` (from the constructor). A bad `pos` makes `execute()` throw `IllegalArgumentException` (the buffer rejects it) and nothing changes.
2. `DeleteCommand(buffer, from, to)` deletes `[from, to)` on `execute()`; `undo()` puts the removed text back at `from`. A bad range makes `execute()` throw `IllegalArgumentException` (the buffer rejects it) and nothing changes. `null` buffer → `IllegalArgumentException`. What is removed is determined when `execute()` runs, so `undo()` restores whatever was actually removed.
3. `MacroCommand(commands)`: `execute()` runs them in list order and is **all-or-nothing** (if one throws, the ones that already ran are undone in reverse order, leaving the buffer as before, and the same exception is rethrown); `undo()` undoes them in reverse order. `null`/empty list → `IllegalArgumentException`.
4. `CommandHistory` (no-argument constructor):
   - `execute(cmd)` runs it; on success it becomes undoable and the redo list is **cleared**; if it throws, the exception propagates and the history is unchanged. `null` → `IllegalArgumentException`.
   - `undo()` undoes the most recently executed (or re-done) command and returns `true`; with nothing to undo it returns `false`. `redo()` re-applies the most recently undone command by calling its `execute()` again and returns `true`; `false` when there is nothing to redo. `canUndo()` / `canRedo()` report whether the corresponding call would return `true`.
   - It works with any `Command` implementation, not only the ones in this problem; it only calls `execute()` / `undo()` on them.

## Examples

Insert / delete:

```
buffer "Hello"        InsertCommand(buffer, 5, ", world").execute() -> "Hello, world"; undo() -> "Hello"
buffer "Hello, world" DeleteCommand(buffer, 5, 12).execute()        -> "Hello";        undo() -> "Hello, world"
```

Macro on `"abc"` = `[Insert(3,"def"), Delete(0,1), Insert(0,"X")]`:

```
execute(): "abc" -> "abcdef" -> "bcdef" -> "Xbcdef"
undo():    "Xbcdef" -> "bcdef" -> "abcdef" -> "abc"
```

Failing macro `[Insert(3,"def"), Delete(0,99), Insert(0,"X")]` on `"abc"`: the insert runs (`"abcdef"`), the delete throws `IllegalArgumentException`, the insert is undone, buffer is `"abc"` again and the exception reaches the caller.

History:

```
execute Insert(0,"one"); execute Insert(3," two"); execute Insert(7," three")   // "one two three"
undo(); undo()  -> "one"          redo() -> "one two"      canUndo() true, canRedo() true

execute Insert(0,"a"); execute Insert(1,"b"); undo()   // "a", canRedo() true
execute Insert(1,"c")                                   // "ac", canRedo() false, redo() returns false
```

A custom command that logs `"run"` in `execute` and `"undo"` in `undo`: `execute(cmd); undo(); redo();` yields the log `run, undo, run`.

## Edge cases

- On a fresh history `undo()` and `redo()` return `false`, `canUndo()` is `false`.
- `history.execute(new DeleteCommand(emptyBuffer, 0, 5))` throws `IllegalArgumentException`, and afterwards `canUndo()` is `false` and the buffer is unchanged.
- `new MacroCommand(List.of())`, `new InsertCommand(null, 0, "x")`, `history.execute(null)` throw `IllegalArgumentException`.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
