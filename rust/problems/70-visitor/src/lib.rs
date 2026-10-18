//! Visitor — operations on a document as separate objects, with double dispatch through `Element::accept`.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// A block of text.
pub struct Paragraph;

impl Paragraph {
    pub fn new(text: &str) -> Paragraph {
        todo!()
    }
}

impl Element for Paragraph {
    /// Calls `visitor.visit_paragraph(text)`.
    fn accept(&self, visitor: &mut dyn Visitor) {
        todo!()
    }
}

/// An image. Blank url or zero width/height -> `Err(DocError::InvalidImage)`.
pub struct Image;

impl Image {
    pub fn new(url: &str, width: u32, height: u32) -> Result<Image, DocError> {
        todo!()
    }
}

impl Element for Image {
    /// Calls `visitor.visit_image(url, width, height)`.
    fn accept(&self, visitor: &mut dyn Visitor) {
        todo!()
    }
}

/// A grid of cells. The rows must be non-empty and every row must have the same non-zero number of cells,
/// otherwise `Err(DocError::InvalidTable)`.
pub struct Table;

impl Table {
    pub fn new(rows: Vec<Vec<String>>) -> Result<Table, DocError> {
        todo!()
    }
}

impl Element for Table {
    /// Calls `visitor.visit_table(rows)`.
    fn accept(&self, visitor: &mut dyn Visitor) {
        todo!()
    }
}

/// Turns elements into HTML fragments (one per visited element, in visiting order). Text and attribute values are
/// escaped: `&` -> `&amp;` (first), `<` -> `&lt;`, `>` -> `&gt;`, `"` -> `&quot;`.
/// - paragraph: `<p>TEXT</p>`
/// - image: `<img src="URL" width="W" height="H"/>`
/// - table: `<table><tr><td>A</td><td>B</td></tr><tr>...</tr></table>`
pub struct HtmlExporter;

impl HtmlExporter {
    pub fn new() -> HtmlExporter {
        todo!()
    }

    pub fn fragments(&self) -> &[String] {
        todo!()
    }
}

impl Visitor for HtmlExporter {
    fn visit_paragraph(&mut self, text: &str) {
        todo!()
    }
    fn visit_image(&mut self, url: &str, width: u32, height: u32) {
        todo!()
    }
    fn visit_table(&mut self, rows: &[Vec<String>]) {
        todo!()
    }
}

/// Counts words (runs of non-whitespace characters): a paragraph counts its words, an image 0, a table all words in
/// all cells. The total accumulates over everything visited.
pub struct WordCounter;

impl WordCounter {
    pub fn new() -> WordCounter {
        todo!()
    }

    pub fn total(&self) -> usize {
        todo!()
    }
}

impl Visitor for WordCounter {
    fn visit_paragraph(&mut self, text: &str) {
        todo!()
    }
    fn visit_image(&mut self, url: &str, width: u32, height: u32) {
        todo!()
    }
    fn visit_table(&mut self, rows: &[Vec<String>]) {
        todo!()
    }
}

/// An ordered list of elements.
pub struct Document;

impl Document {
    pub fn new() -> Document {
        todo!()
    }

    pub fn add(&mut self, element: Box<dyn Element>) {
        todo!()
    }

    pub fn len(&self) -> usize {
        todo!()
    }

    /// Applies the visitor to every element, in order.
    pub fn accept_all(&self, visitor: &mut dyn Visitor) {
        todo!()
    }

    /// The HTML of every element joined with "\n", produced with an `HtmlExporter`. Empty document -> "".
    pub fn to_html(&self) -> String {
        todo!()
    }

    /// Total number of words, computed with a `WordCounter`.
    pub fn total_words(&self) -> usize {
        todo!()
    }
}
