# 67 · Command (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 50 min | Level 2 — `TextBuffer` (the receiver) and the `Command` interface are provided; three commands and `CommandHistory` are stubs. |

> **Pattern:** *Command* — turn a request into an **object**. Once an action is an object it can be stored,
> queued, logged, composed (macros) and **undone**. The invoker (`CommandHistory`) neither knows nor cares what a
> command does.

## Problem

A tiny text editor needs undo/redo and recordable macros. Every edit is a command with `execute()` and `undo()`.

## Requirements

1. `InsertCommand(buffer, pos, text)` inserts; `undo()` removes exactly that text. `null` buffer or text → `IllegalArgumentException`.
2. `DeleteCommand(buffer, from, to)` deletes `[from, to)`, **remembering the removed text**; `undo()` puts it back at `from`. A bad range makes `execute()` throw `IllegalArgumentException` (the buffer rejects it) and nothing changes. `null` buffer → `IllegalArgumentException`.
3. `MacroCommand(commands)`: `execute()` runs them in order and is **all-or-nothing** (if one throws, the already-run ones are undone in reverse order and the exception is rethrown); `undo()` undoes them in reverse order. `null`/empty list → `IllegalArgumentException`.
4. `CommandHistory`:
   - `execute(cmd)` runs it; on success it becomes undoable and the redo list is **cleared**; if it throws, the history is unchanged. `null` → `IllegalArgumentException`.
   - `undo()` / `redo()` return `false` when there is nothing to do. `redo()` **executes** the command again. `canUndo()` / `canRedo()`.

## Constraints (what the tests catch)

- The tests add an `UppercaseCommand` (defined in the test file) and an anonymous logging command to the history; `CommandHistory` must treat them like any other.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
