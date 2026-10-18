//! Provided: the visitor and element traits and the error type.
//!
//! Rust adaptation: a generic `accept<R>` would make `Element` impossible to use as `Box<dyn Element>`, so a visitor
//! here is a `&mut` object that accumulates its result inside itself (`HtmlExporter` collects fragments,
//! `WordCounter` keeps a total) and `accept` returns nothing.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocError {
    /// Blank image url, zero width/height.
    InvalidImage,
    /// Empty, ragged or zero-width table.
    InvalidTable,
}

/// One method per kind of document element. A new operation = a new `Visitor` type; elements never change.
pub trait Visitor {
    fn visit_paragraph(&mut self, text: &str);
    fn visit_image(&mut self, url: &str, width: u32, height: u32);
    /// `rows` is a rectangular, non-empty grid of cell texts.
    fn visit_table(&mut self, rows: &[Vec<String>]);
}

/// The entry point of the double dispatch.
pub trait Element {
    fn accept(&self, visitor: &mut dyn Visitor);
}
