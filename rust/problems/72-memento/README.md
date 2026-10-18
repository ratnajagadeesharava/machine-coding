# 72 · Memento (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 50 min | Level 2 — `GameCharacter`, an empty `Snapshot`, `SaveSlots` and `UndoHistory` are stubs; `GameError` is provided. |

> **Pattern:** *Memento* — capture and externalise an object's internal state **without violating encapsulation**, so
> it can be restored later. Three roles: the **originator** (`GameCharacter`: creates and consumes snapshots), the
> **memento** (`Snapshot`: an opaque box nobody else can open) and the **caretaker** (`SaveSlots`, `UndoHistory`:
> stores snapshots without looking inside).
>
> **Rust adaptation:** privacy is module-level, so keep `Snapshot`'s fields private (no `pub`) and give it no
> methods. The caretakers take the character as a parameter on every call instead of holding a reference to it (that
> would fight the borrow checker).

## Problem

A role-playing game needs manual save slots ("before the boss fight") and an undo/redo for the character-editor
screen. The character has plenty of internal state (hp, level, position, inventory) and the save system must not need
getters or setters to copy it.

## Requirements

1. `GameCharacter::new(name)` (blank → `Err(GameError::InvalidInput)`): hp 100, level 1, position (0, 0), empty inventory. `take_damage(n)` / `heal(n)` (hp stays within `0..=MAX_HP`), `level_up()`, `move_to(x, y)`, `pick_up(item)` (blank → `Err(InvalidInput)`, already `MAX_ITEMS` (3) → `Err(InventoryFull)`), `drop_item(item)` (returns whether it was carried).
2. `save()` returns a `Snapshot` holding a **deep, independent** copy of hp, level, position and inventory. `restore(&snapshot)` puts all of that back; the **name is not part of the state**, so a snapshot can be restored into another character. A snapshot can be restored repeatedly.
3. `Snapshot` is opaque: **no `pub` fields and no `impl Snapshot` block** (the tests scan `src/lib.rs`). Keep `#[derive(Clone)]`.
4. `SaveSlots`: `save(slot, &character)` (replaces any earlier snapshot; blank slot → `Err(InvalidInput)`), `load(slot, &mut character)` → `true` / `false` for an unknown slot (nothing changes), `slot_count()`.
5. `UndoHistory::new(max_checkpoints)` (0 → `Err(InvalidInput)`):
   - `checkpoint(&character)` remembers the current state and clears the redo list; beyond `max_checkpoints` the oldest is dropped.
   - `undo(&mut character)` goes back to the newest checkpoint (consuming it) and keeps the state it left so `redo` can return; `false` if there is none.
   - `redo(&mut character)` returns to the state left by the latest undo and makes that undo's target undoable again; `false` if there is none. `undo_depth()` / `redo_depth()` count the stacks.

## Running

```bash
cargo test -p p72_memento     # from rust/ — tests FAIL until you implement the stubs
cargo play run 72
```
