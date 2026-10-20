# 72 · Memento (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 50 min | Level 2 — `GameError` is provided; you write `GameCharacter`, `Snapshot`, `SaveSlots` and `UndoHistory`. |


## Problem

A role-playing game needs manual save slots ("before the boss fight") and an undo/redo for the character-editor
screen. The character has plenty of internal state (hp, level, position, inventory).

## Requirements

1. `GameCharacter::new(name)` (blank → `Err(GameError::InvalidInput)`): hp 100, level 1, position (0, 0), empty inventory. `take_damage(n)` / `heal(n)` (hp stays within `0..=MAX_HP`), `level_up()`, `move_to(x, y)`, `pick_up(item)` (blank → `Err(InvalidInput)`, already `MAX_ITEMS` (3) → `Err(InventoryFull)`), `drop_item(item)` (returns whether it was carried).
2. `save()` returns a `Snapshot` holding a **deep, independent** copy of hp, level, position and inventory. `restore(&snapshot)` puts all of that back; the **name is not part of the state**, so a snapshot can be restored into another character. A snapshot can be restored repeatedly.
3. `Snapshot` has no `pub` fields and no `impl Snapshot` block (the tests scan `src/lib.rs`).
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
