//! Command — edits as objects, with a history that can undo and redo them.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// Inserts text into the buffer.
pub struct InsertCommand;

impl InsertCommand {
    pub fn new(pos: usize, text: &str) -> InsertCommand {
        todo!()
    }
}

impl Command for InsertCommand {
    fn execute(&mut self, buffer: &mut TextBuffer) -> Result<(), EditError> {
        todo!()
    }
    fn undo(&mut self, buffer: &mut TextBuffer) {
        todo!()
    }
}

/// Deletes a range of text.
pub struct DeleteCommand;

impl DeleteCommand {
    pub fn new(from: usize, to: usize) -> DeleteCommand {
        todo!()
    }
}

impl Command for DeleteCommand {
    fn execute(&mut self, buffer: &mut TextBuffer) -> Result<(), EditError> {
        todo!()
    }
    fn undo(&mut self, buffer: &mut TextBuffer) {
        todo!()
    }
}

/// Several commands as one.
pub struct MacroCommand;

impl MacroCommand {
    pub fn new(commands: Vec<Box<dyn Command>>) -> MacroCommand {
        todo!()
    }
}

impl Command for MacroCommand {
    fn execute(&mut self, buffer: &mut TextBuffer) -> Result<(), EditError> {
        todo!()
    }
    fn undo(&mut self, buffer: &mut TextBuffer) {
        todo!()
    }
}

/// Runs commands with undo / redo support.
pub struct CommandHistory;

impl CommandHistory {
    pub fn new() -> CommandHistory {
        todo!()
    }

    /// Runs the command.
    pub fn execute(&mut self, command: Box<dyn Command>, buffer: &mut TextBuffer) -> Result<(), EditError> {
        todo!()
    }

    /// Returns false if there is nothing to undo.
    pub fn undo(&mut self, buffer: &mut TextBuffer) -> bool {
        todo!()
    }

    /// Returns false if there is nothing to redo.
    pub fn redo(&mut self, buffer: &mut TextBuffer) -> bool {
        todo!()
    }

    pub fn can_undo(&self) -> bool {
        todo!()
    }

    pub fn can_redo(&self) -> bool {
        todo!()
    }
}
