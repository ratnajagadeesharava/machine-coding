//! Memento — save and restore a `GameCharacter`.
#![allow(unused_variables)]

pub mod contract;

use contract::GameError;

pub const MAX_HP: u32 = 100;
pub const MAX_ITEMS: usize = 3;

/// A saved character state.
#[derive(Clone)]
pub struct Snapshot;

/// A game character. Initial state: hp 100, level 1, position (0, 0), empty inventory.
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

    /// Captures the character's state.
    pub fn save(&self) -> Snapshot {
        todo!()
    }

    /// Restores a saved state.
    pub fn restore(&mut self, snapshot: &Snapshot) {
        todo!()
    }
}

/// Named save slots.
pub struct SaveSlots;

impl SaveSlots {
    pub fn new() -> SaveSlots {
        todo!()
    }

    /// Saves the character into the slot.
    pub fn save(&mut self, slot: &str, character: &GameCharacter) -> Result<(), GameError> {
        todo!()
    }

    /// Loads the slot into the character.
    pub fn load(&self, slot: &str, character: &mut GameCharacter) -> bool {
        todo!()
    }

    pub fn slot_count(&self) -> usize {
        todo!()
    }
}

/// Undo/redo for one character.
pub struct UndoHistory;

impl UndoHistory {
    /// `max_checkpoints == 0` -> `Err(InvalidInput)`.
    pub fn new(max_checkpoints: usize) -> Result<UndoHistory, GameError> {
        todo!()
    }

    /// Remembers the current state.
    pub fn checkpoint(&mut self, character: &GameCharacter) {
        todo!()
    }

    /// Goes back one checkpoint.
    pub fn undo(&mut self, character: &mut GameCharacter) -> bool {
        todo!()
    }

    /// Re-applies the latest undone state.
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
