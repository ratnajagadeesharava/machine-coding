# 67 · Command (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 50 min | Level 2 — `TextBuffer`, `Command` and `EditError` provided; write `InsertCommand`, `DeleteCommand`, `MacroCommand` and `CommandHistory`. |

## Problem

A tiny text editor needs undo/redo and recordable macros. Every edit is a command with `execute` and `undo`.

## Requirements

1. `InsertCommand::new(pos, text)` inserts at that **character** position; `undo` removes exactly that text.
2. `DeleteCommand::new(from, to)` deletes `[from, to)`; `undo` puts the removed text back at `from`. A bad range makes `execute` return `Err(EditError::OutOfRange)` and nothing changes.
3. `MacroCommand::new(commands)`: `execute` runs them in order and is **all-or-nothing** (if one fails, the already-run ones are undone in reverse order and the error is returned); `undo` undoes them in reverse order.
4. `CommandHistory`:
   - `execute(cmd, buffer)` runs it; on success it becomes undoable and the redo list is **cleared**; on `Err` the history is unchanged and the error is returned.
   - `undo(buffer)` / `redo(buffer)` return `false` when there is nothing to do. `redo` **executes** the command again. `can_undo()` / `can_redo()`.

## Running

```bash
cargo test -p p67_command     # from rust/ — tests FAIL until you implement the stubs
cargo play run 67
```
