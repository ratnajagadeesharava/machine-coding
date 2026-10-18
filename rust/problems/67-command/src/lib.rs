//! Command — edits as objects, with a history that can undo and redo them.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// Inserts `text` at character position `pos`. Undo removes exactly what was inserted.
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

/// Deletes `[from, to)`. It remembers the text it removed so that undo can put it back at `from`.
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

/// Several commands as one. `execute` runs them in order; if one fails, the ones already run are undone in
/// reverse order (all-or-nothing) and the error is returned. `undo` undoes them in reverse order.
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

/// The invoker: runs commands and remembers them for undo / redo.
pub struct CommandHistory;

impl CommandHistory {
    pub fn new() -> CommandHistory {
        todo!()
    }

    /// Runs the command. On success it becomes the newest undoable command and the redo list is cleared. On `Err`
    /// the history is unchanged and the error is returned.
    pub fn execute(&mut self, command: Box<dyn Command>, buffer: &mut TextBuffer) -> Result<(), EditError> {
        todo!()
    }

    /// Undoes the newest command and makes it redoable. Returns false (doing nothing) if there is nothing to undo.
    pub fn undo(&mut self, buffer: &mut TextBuffer) -> bool {
        todo!()
    }

    /// Executes the newest undone command again and makes it undoable. Returns false if there is nothing to redo.
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
