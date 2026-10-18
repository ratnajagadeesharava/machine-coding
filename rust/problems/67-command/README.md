# 67 · Command (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 50 min | Level 2 — `TextBuffer` (the receiver), the `Command` trait and `EditError` are provided; three commands and `CommandHistory` are stubs. |

> **Pattern:** *Command* — turn a request into an **object**. Once an action is an object it can be stored, queued,
> logged, composed (macros) and **undone**. The invoker (`CommandHistory`) neither knows nor cares what a command does.
>
> **Rust adaptation:** a command that held `&mut TextBuffer` would fight the borrow checker, so the receiver is
> **passed in on every call** (`execute(&mut self, buffer: &mut TextBuffer)`). Commands own their own data
> (position, text, the removed text) and the history owns the commands as `Box<dyn Command>`.

## Problem

A tiny text editor needs undo/redo and recordable macros. Every edit is a command with `execute` and `undo`.

## Requirements

1. `InsertCommand::new(pos, text)` inserts at that **character** position; `undo` removes exactly that text.
2. `DeleteCommand::new(from, to)` deletes `[from, to)`, **remembering the removed text**; `undo` puts it back at `from`. A bad range makes `execute` return `Err(EditError::OutOfRange)` and nothing changes.
3. `MacroCommand::new(commands)`: `execute` runs them in order and is **all-or-nothing** (if one fails, the already-run ones are undone in reverse order and the error is returned); `undo` undoes them in reverse order.
4. `CommandHistory`:
   - `execute(cmd, buffer)` runs it; on success it becomes undoable and the redo list is **cleared**; on `Err` the history is unchanged and the error is returned.
   - `undo(buffer)` / `redo(buffer)` return `false` when there is nothing to do. `redo` **executes** the command again. `can_undo()` / `can_redo()`.

## Constraints (what the tests catch)

- The tests add an `UppercaseCommand` and a logging command (both defined in the test file) to the history; `CommandHistory` must treat them like any other.

## Running

```bash
cargo test -p p67_command     # from rust/ — tests FAIL until you implement the stubs
cargo play run 67
```
