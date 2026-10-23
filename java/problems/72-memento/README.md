# 72 · Memento (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 50 min | Level 2 — you write `GameCharacter` (with its nested `Snapshot`), `SaveSlots` and `UndoHistory`. |


## Problem

A role-playing game needs manual save slots ("before the boss fight") and an undo/redo for the character-editor
screen. The character has plenty of internal state (hp, level, position, inventory).

## Requirements

1. `GameCharacter(name)`: hp 100, level 1, position (0, 0), empty inventory. `takeDamage(n)` / `heal(n)` (negative → `IllegalArgumentException`; hp stays within `0..=MAX_HP`), `levelUp()`, `moveTo(x, y)`, `pickUp(item)` (blank → `IllegalArgumentException`, already `MAX_ITEMS` (3) → `IllegalStateException`), `drop(item)` (returns whether it was carried), `inventory()` is a read-only view.
2. `save()` returns a `Snapshot` holding a **deep, independent** copy of hp, level, position and inventory. `restore(snapshot)` puts all of that back (null → `IllegalArgumentException`); the **name is not part of the state**, so a snapshot can be restored into another character. A snapshot can be restored repeatedly.
3. `Snapshot` exposes no public constructor, methods or fields, and `GameCharacter` has no `setXxx` methods (checked by reflection).
4. `SaveSlots`: `save(slot, character)` (replaces any earlier snapshot; blank slot or null character → `IllegalArgumentException`), `load(slot, character)` → `true` / `false` for an unknown slot (nothing changes), `slotCount()`.
5. `UndoHistory(character, maxCheckpoints)` (`< 1` or null → `IllegalArgumentException`):
   - `checkpoint()` remembers the current state and clears the redo list; beyond `maxCheckpoints` the oldest is dropped.
   - `undo()` goes back to the newest checkpoint (consuming it) and keeps the state it left so `redo()` can return; `false` if there is none.
   - `redo()` returns to the state left by the latest undo and makes that undo's target undoable again; `false` if there is none. `undoDepth()` / `redoDepth()` count the stacks.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
