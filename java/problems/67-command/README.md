# 67 · Command (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 50 min | Level 2 — `TextBuffer` and `Command` provided; write `InsertCommand`, `DeleteCommand`, `MacroCommand` and `CommandHistory`. |

## Problem

A tiny text editor needs undo/redo and recordable macros. Every edit is a command with `execute()` and `undo()`.

## Requirements

1. `InsertCommand(buffer, pos, text)` inserts; `undo()` removes exactly that text. `null` buffer or text → `IllegalArgumentException`.
2. `DeleteCommand(buffer, from, to)` deletes `[from, to)`; `undo()` puts the removed text back at `from`. A bad range makes `execute()` throw `IllegalArgumentException` (the buffer rejects it) and nothing changes. `null` buffer → `IllegalArgumentException`.
3. `MacroCommand(commands)`: `execute()` runs them in order and is **all-or-nothing** (if one throws, the already-run ones are undone in reverse order and the exception is rethrown); `undo()` undoes them in reverse order. `null`/empty list → `IllegalArgumentException`.
4. `CommandHistory`:
   - `execute(cmd)` runs it; on success it becomes undoable and the redo list is **cleared**; if it throws, the history is unchanged. `null` → `IllegalArgumentException`.
   - `undo()` / `redo()` return `false` when there is nothing to do. `redo()` **executes** the command again. `canUndo()` / `canRedo()`.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
