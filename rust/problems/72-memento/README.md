# 72 · Memento (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 50 min | Level 2 — `GameError` is provided; you write `GameCharacter`, `Snapshot`, `SaveSlots` and `UndoHistory`. |


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

1. `GameCharacter::new(name)` (blank → `Err(GameError::InvalidInput)`): hp 100, level 1, position (0, 0), empty inventory. Accessors: `name()`, `hp()` (`u32`), `level()` (`u32`), `position()` (`(i32, i32)`), `inventory()` (`&[String]`). Mutators: `take_damage(n)` / `heal(n)` (`u32` amounts; hp stays within `0..=MAX_HP`, `MAX_HP` = 100), `level_up()` (level + 1), `move_to(x, y)` (absolute position, negatives allowed), `pick_up(item)` (blank → `Err(InvalidInput)`, already `MAX_ITEMS` (3) items → `Err(InventoryFull)`; items keep pick-up order), `drop_item(item)` (returns whether it was carried).
2. `save()` returns a `Snapshot` holding a **deep, independent** copy of hp, level, position and inventory (later changes to the character do not alter it). `restore(&snapshot)` puts all of that back; the **name is not part of the state**, so a snapshot can be restored into another character and that character keeps its own name. A snapshot can be restored repeatedly.
3. `Snapshot` has no `pub` fields and no `impl Snapshot` block (the tests scan `src/lib.rs`; no struct in that file may have a `pub` field).
4. `SaveSlots`: `SaveSlots::new()`, `save(slot, &character)` (replaces any earlier snapshot in that slot; blank slot → `Err(InvalidInput)`), `load(slot, &mut character)` → `true` after restoring, or `false` for an unknown slot (the character is left unchanged), `slot_count()` (number of distinct slots; overwriting does not add one).
5. `UndoHistory::new(max_checkpoints)` (0 → `Err(InvalidInput)`):
   - `checkpoint(&character)` remembers the character's current state and clears the redo list; beyond `max_checkpoints` the oldest is dropped.
   - `undo(&mut character)` goes back to the newest checkpoint (consuming it) and keeps the state it left so `redo` can return; `false` (nothing changes) if there is none.
   - `redo(&mut character)` returns to the state left by the latest undo and makes that undo's target undoable again; `false` if there is none. `undo_depth()` / `redo_depth()` count the stacks.

## Examples

Character `c = GameCharacter::new("Aria")`, history `UndoHistory::new(10)`. State shown as hp / level.

| Step | State | undo_depth / redo_depth |
|---|---|---|
| `checkpoint(&c)` | 100 / 1 | 1 / 0 |
| `take_damage(30)`, `checkpoint(&c)` | 70 / 1 | 2 / 0 |
| `level_up()`, `take_damage(20)` | 50 / 2 | 2 / 0 |
| `undo(&mut c)` → `true` | 70 / 1 | 1 / 1 |
| `undo(&mut c)` → `true` | 100 / 1 | 0 / 2 |
| `undo(&mut c)` → `false` | 100 / 1 | 0 / 2 |
| `redo(&mut c)` → `true` | 70 / 1 | 1 / 1 |
| `redo(&mut c)` → `true` | 50 / 2 | 2 / 0 |

Limit: with `max_checkpoints` = 2, three rounds of (`checkpoint(&c)`, `take_damage(10)`) leave hp 70 and `undo_depth()` 2; two undos give 80 then 90, the third returns `false` (the checkpoint at hp 100 was dropped).

Save slots: `save("before-boss", &c)` at hp 100; `take_damage(90)`, `move_to(5, 5)`, `save("after-boss", &c)`; `heal(100)`. `slot_count()` = 2; `load("after-boss", &mut c)` → hp 10 at (5, 5); `load("nope", &mut c)` → `false`, no change.

## Edge cases

- Taking a checkpoint after an undo discards every redo state (`redo_depth()` becomes 0).
- `heal(1000)` caps hp at 100; `take_damage(1000)` floors it at 0.
- Restoring a snapshot taken with `["sword"]` after picking up more items leaves exactly `["sword"]`, and a snapshot restored twice gives the same state both times.
- Dropping an item that is not carried returns `false` and changes nothing.

## Running

```bash
cargo test -p p72_memento     # from rust/ — tests FAIL until you implement the stubs
cargo play run 72
```
