# 72 · Memento (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 50 min | Level 2 — `GameCharacter` (with an empty nested `Snapshot`), `SaveSlots` and `UndoHistory` are stubs. |

> **Pattern:** *Memento* — capture and externalise an object's internal state **without violating encapsulation**, so
> it can be restored later. Three roles: the **originator** (`GameCharacter`: creates and consumes snapshots), the
> **memento** (`Snapshot`: an opaque box nobody else can open) and the **caretaker** (`SaveSlots`, `UndoHistory`:
> stores snapshots without looking inside).

## Problem

A role-playing game needs manual save slots ("before the boss fight") and an undo/redo for the character-editor
screen. The character has plenty of internal state (hp, level, position, inventory), all of it private, and the
save system must not need getters, setters or friend access to copy it.

## Requirements

1. `GameCharacter(name)`: hp 100, level 1, position (0, 0), empty inventory. `takeDamage(n)` / `heal(n)` (negative → `IllegalArgumentException`; hp stays within `0..=MAX_HP`), `levelUp()`, `moveTo(x, y)`, `pickUp(item)` (blank → `IllegalArgumentException`, already `MAX_ITEMS` (3) → `IllegalStateException`), `drop(item)` (returns whether it was carried), `inventory()` is a read-only view.
2. `save()` returns a `Snapshot` holding a **deep, independent** copy of hp, level, position and inventory. `restore(snapshot)` puts all of that back (null → `IllegalArgumentException`); the **name is not part of the state**, so a snapshot can be restored into another character. A snapshot can be restored repeatedly.
3. `Snapshot` is opaque: **no public constructor, no public methods, no non-private fields** (checked by reflection), but it does have private fields. `GameCharacter`'s own fields are all private and it has no `setXxx` methods.
4. `SaveSlots`: `save(slot, character)` (replaces any earlier snapshot; blank slot or null character → `IllegalArgumentException`), `load(slot, character)` → `true` / `false` for an unknown slot (nothing changes), `slotCount()`.
5. `UndoHistory(character, maxCheckpoints)` (`< 1` or null → `IllegalArgumentException`):
   - `checkpoint()` remembers the current state and clears the redo list; beyond `maxCheckpoints` the oldest is dropped.
   - `undo()` goes back to the newest checkpoint (consuming it) and keeps the state it left so `redo()` can return; `false` if there is none.
   - `redo()` returns to the state left by the latest undo and makes that undo's target undoable again; `false` if there is none. `undoDepth()` / `redoDepth()` count the stacks.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
