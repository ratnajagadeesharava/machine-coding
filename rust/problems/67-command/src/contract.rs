//! Provided: the receiver (`TextBuffer`), the error type and the command trait.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditError {
    /// A position or range outside the text.
    OutOfRange,
}

/// The text being edited.
#[derive(Debug, Default)]
pub struct TextBuffer {
    text: String,
}

impl TextBuffer {
    pub fn new() -> TextBuffer {
        TextBuffer::default()
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn len(&self) -> usize {
        self.text.chars().count()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Inserts at character position `pos` (0..=len), otherwise `Err(OutOfRange)`.
    pub fn insert(&mut self, pos: usize, s: &str) -> Result<(), EditError> {
        if pos > self.len() {
            return Err(EditError::OutOfRange);
        }
        let byte = self.text.char_indices().nth(pos).map_or(self.text.len(), |(i, _)| i);
        self.text.insert_str(byte, s);
        Ok(())
    }

    /// Removes characters `[from, to)` (from <= to <= len) and returns them, otherwise `Err(OutOfRange)`.
    pub fn delete(&mut self, from: usize, to: usize) -> Result<String, EditError> {
        if from > to || to > self.len() {
            return Err(EditError::OutOfRange);
        }
        let start = self.text.char_indices().nth(from).map_or(self.text.len(), |(i, _)| i);
        let end = self.text.char_indices().nth(to).map_or(self.text.len(), |(i, _)| i);
        let removed: String = self.text[start..end].to_string();
        self.text.replace_range(start..end, "");
        Ok(removed)
    }
}

/// An undoable edit; the buffer is passed in on each call.
pub trait Command {
    /// Does the work. On `Err` it must have changed nothing.
    fn execute(&mut self, buffer: &mut TextBuffer) -> Result<(), EditError>;
    /// Reverses the most recent successful `execute`.
    fn undo(&mut self, buffer: &mut TextBuffer);
}
