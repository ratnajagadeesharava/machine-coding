//! Memento — `GameCharacter` (originator) saves into an opaque `Snapshot` (memento) kept by `SaveSlots` and
//! `UndoHistory` (caretakers) without them ever seeing inside.
#![allow(unused_variables)]

pub mod contract;

use contract::GameError;

pub const MAX_HP: u32 = 100;
pub const MAX_ITEMS: usize = 3;

/// The memento: opaque to everybody except `GameCharacter`. Keep every field PRIVATE (no `pub` fields) and give it
/// no accessor methods; the tests scan this file. Add private fields to hold the state, and keep the derive: the
/// caretakers need to clone snapshots.
#[derive(Clone)]
pub struct Snapshot;

/// The originator. Initial state: hp 100, level 1, position (0, 0), empty inventory.
pub struct GameCharacter;

impl GameCharacter {
    /// Blank name -> `Err(GameError::InvalidInput)`.
    pub fn new(name: &str) -> Result<GameCharacter, GameError> {
        todo!()
    }

    pub fn name(&self) -> &str {
        todo!()
    }

    pub fn hp(&self) -> u32 {
        todo!()
    }

    pub fn level(&self) -> u32 {
        todo!()
    }

    pub fn position(&self) -> (i32, i32) {
        todo!()
    }

    pub fn inventory(&self) -> &[String] {
        todo!()
    }

    /// hp never goes below 0.
    pub fn take_damage(&mut self, amount: u32) {
        todo!()
    }

    /// hp never goes above `MAX_HP`.
    pub fn heal(&mut self, amount: u32) {
        todo!()
    }

    pub fn level_up(&mut self) {
        todo!()
    }

    pub fn move_to(&mut self, x: i32, y: i32) {
        todo!()
    }

    /// Blank item -> `Err(InvalidInput)`; already carrying `MAX_ITEMS` -> `Err(InventoryFull)`.
    pub fn pick_up(&mut self, item: &str) -> Result<(), GameError> {
        todo!()
    }

    /// Removes one copy of the item. Returns whether the character was carrying it.
    pub fn drop_item(&mut self, item: &str) -> bool {
        todo!()
    }

    /// Captures hp, level, position and inventory as a deep, independent copy.
    pub fn save(&self) -> Snapshot {
        todo!()
    }

    /// Puts the character back into the saved state (the name is not part of the state). A snapshot can be restored
    /// repeatedly.
    pub fn restore(&mut self, snapshot: &Snapshot) {
        todo!()
    }
}

/// The caretaker for manual saves: it stores snapshots under a name and hands them back, never looking inside.
pub struct SaveSlots;

impl SaveSlots {
    pub fn new() -> SaveSlots {
        todo!()
    }

    /// Stores a snapshot in the slot, replacing any earlier one. Blank slot -> `Err(InvalidInput)`.
    pub fn save(&mut self, slot: &str, character: &GameCharacter) -> Result<(), GameError> {
        todo!()
    }

    /// Restores the slot's snapshot into the character. Returns false (and changes nothing) for an unknown slot.
    pub fn load(&self, slot: &str, character: &mut GameCharacter) -> bool {
        todo!()
    }

    pub fn slot_count(&self) -> usize {
        todo!()
    }
}

/// The caretaker for undo/redo of one character. It stores snapshots only.
pub struct UndoHistory;

impl UndoHistory {
    /// `max_checkpoints == 0` -> `Err(InvalidInput)`.
    pub fn new(max_checkpoints: usize) -> Result<UndoHistory, GameError> {
        todo!()
    }

    /// Remembers the character's CURRENT state as something to go back to, and forgets everything that could be
    /// redone. When more than `max_checkpoints` are remembered the oldest is dropped.
    pub fn checkpoint(&mut self, character: &GameCharacter) {
        todo!()
    }

    /// Goes back to the newest checkpoint (which is consumed). The state the character had just before going back is
    /// kept so that `redo` can return to it. False (doing nothing) when there is no checkpoint.
    pub fn undo(&mut self, character: &mut GameCharacter) -> bool {
        todo!()
    }

    /// Returns to the state left by the latest undo; that undo's target becomes undoable again. False if nothing to redo.
    pub fn redo(&mut self, character: &mut GameCharacter) -> bool {
        todo!()
    }

    pub fn undo_depth(&self) -> usize {
        todo!()
    }

    pub fn redo_depth(&self) -> usize {
        todo!()
    }
}
