//! Document elements and operations on them.
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
    fn accept(&self, visitor: &mut dyn Visitor) {
        todo!()
    }
}

/// An image.
pub struct Image;

impl Image {
    pub fn new(url: &str, width: u32, height: u32) -> Result<Image, DocError> {
        todo!()
    }
}

impl Element for Image {
    fn accept(&self, visitor: &mut dyn Visitor) {
        todo!()
    }
}

/// A grid of cells.
pub struct Table;

impl Table {
    pub fn new(rows: Vec<Vec<String>>) -> Result<Table, DocError> {
        todo!()
    }
}

impl Element for Table {
    fn accept(&self, visitor: &mut dyn Visitor) {
        todo!()
    }
}

/// Turns elements into HTML fragments.
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

/// Counts words.
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

    /// Applies the visitor to every element.
    pub fn accept_all(&self, visitor: &mut dyn Visitor) {
        todo!()
    }

    /// The HTML of every element joined with "\n". Empty document -> "".
    pub fn to_html(&self) -> String {
        todo!()
    }

    /// Total number of words.
    pub fn total_words(&self) -> usize {
        todo!()
    }
}
