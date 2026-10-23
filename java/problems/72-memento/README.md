# 72 · Memento (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 50 min | Level 2 — you write `GameCharacter` (with its nested `Snapshot`), `SaveSlots` and `UndoHistory`. |


## Problem

A role-playing game needs manual save slots ("before the boss fight") and an undo/redo for the character-editor
screen. The character has plenty of internal state (hp, level, position, inventory).

What you are building: a game character whose full state can be captured as a snapshot and put back later, without
outsiders being able to read or tamper with that snapshot. On top of that you build named save slots (the player
saves and loads by name) and an undo/redo history (the editor takes checkpoints; undo and redo move through them).

### Glossary

- **Snapshot**: an opaque saved copy of a character's state (hp, level, position, inventory). Callers can only hand it back to `restore`.
- **Save slot**: a named place holding one snapshot.
- **Checkpoint**: a state remembered by `UndoHistory` so it can be undone back to.
- **Undo / redo stacks**: the checkpoints you can go back to / the states you can return forward to.

## Requirements

1. `GameCharacter(name)` (blank or null name → `IllegalArgumentException`): hp 100, level 1, position (0, 0), empty inventory. Accessors: `name()`, `hp()`, `level()`, `x()`, `y()`, `inventory()` (read-only view; modifying it throws `UnsupportedOperationException`). Mutators: `takeDamage(n)` / `heal(n)` (negative → `IllegalArgumentException`; hp stays within `0..=MAX_HP`, `MAX_HP` = 100), `levelUp()` (level + 1), `moveTo(x, y)` (absolute position, negatives allowed), `pickUp(item)` (blank → `IllegalArgumentException`, already `MAX_ITEMS` (3) items → `IllegalStateException`; items keep pick-up order), `drop(item)` (returns whether it was carried).
2. `save()` returns a `Snapshot` holding a **deep, independent** copy of hp, level, position and inventory (later changes to the character do not alter it). `restore(snapshot)` puts all of that back (null → `IllegalArgumentException`); the **name is not part of the state**, so a snapshot can be restored into another character and that character keeps its own name. A snapshot can be restored repeatedly.
3. `Snapshot` exposes no public constructor, methods or fields, and `GameCharacter` has no `setXxx` methods (checked by reflection); all non-static `GameCharacter` fields are private.
4. `SaveSlots`: `save(slot, character)` (replaces any earlier snapshot in that slot; blank slot or null character → `IllegalArgumentException`), `load(slot, character)` → `true` after restoring, or `false` for an unknown slot (the character is left unchanged), `slotCount()` (number of distinct slots; overwriting does not add one).
5. `UndoHistory(character, maxCheckpoints)` (`< 1` or null character → `IllegalArgumentException`):
   - `checkpoint()` remembers the character's current state and clears the redo list; beyond `maxCheckpoints` the oldest is dropped.
   - `undo()` goes back to the newest checkpoint (consuming it) and keeps the state it left so `redo()` can return; `false` (nothing changes) if there is none.
   - `redo()` returns to the state left by the latest undo and makes that undo's target undoable again; `false` if there is none. `undoDepth()` / `redoDepth()` count the stacks.

## Examples

Character `c = new GameCharacter("Aria")`, history `new UndoHistory(c, 10)`. State shown as hp / level.

| Step | State | undoDepth / redoDepth |
|---|---|---|
| `checkpoint()` | 100 / 1 | 1 / 0 |
| `takeDamage(30)`, `checkpoint()` | 70 / 1 | 2 / 0 |
| `levelUp()`, `takeDamage(20)` | 50 / 2 | 2 / 0 |
| `undo()` → `true` | 70 / 1 | 1 / 1 |
| `undo()` → `true` | 100 / 1 | 0 / 2 |
| `undo()` → `false` | 100 / 1 | 0 / 2 |
| `redo()` → `true` | 70 / 1 | 1 / 1 |
| `redo()` → `true` | 50 / 2 | 2 / 0 |

Limit: with `maxCheckpoints` = 2, three rounds of (`checkpoint()`, `takeDamage(10)`) leave hp 70 and `undoDepth()` 2; two undos give 80 then 90, the third returns `false` (the checkpoint at hp 100 was dropped).

Save slots: `save("before-boss", c)` at hp 100; `takeDamage(90)`, `moveTo(5, 5)`, `save("after-boss", c)`; `heal(100)`. `slotCount()` = 2; `load("after-boss", c)` → hp 10 at (5, 5); `load("nope", c)` → `false`, no change.

## Edge cases

- Taking a checkpoint after an undo discards every redo state (`redoDepth()` becomes 0).
- `heal(1000)` caps hp at 100; `takeDamage(1000)` floors it at 0.
- Restoring a snapshot taken with `["sword"]` after picking up more items leaves exactly `["sword"]`, and a snapshot restored twice gives the same state both times.
- Dropping an item that is not carried returns `false` and changes nothing.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```
