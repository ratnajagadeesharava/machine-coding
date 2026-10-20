//! Composite.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// The leaf. Blank name or a name containing '/' -> `InvalidName`.
pub struct File;

impl File {
    pub fn new(name: &str, size_bytes: u64) -> Result<File, FsError> {
        todo!()
    }
}

impl FsNode for File {
    fn name(&self) -> &str {
        todo!()
    }
    fn size(&self) -> u64 {
        todo!()
    }
    fn file_count(&self) -> usize {
        todo!()
    }
    fn paths(&self) -> Vec<String> {
        todo!()
    }
}

/// The composite: a folder that contains nodes of any kind, including other folders.
pub struct Directory;

impl Directory {
    /// Blank name or a name containing '/' -> `InvalidName`.
    pub fn new(name: &str) -> Result<Directory, FsError> {
        todo!()
    }

    /// A name already used in this folder -> `Err(DuplicateName)` (and the child is dropped).
    pub fn add(&mut self, child: Box<dyn FsNode>) -> Result<(), FsError> {
        todo!()
    }

    /// Removes and returns the child with that name.
    pub fn remove(&mut self, name: &str) -> Option<Box<dyn FsNode>> {
        todo!()
    }

    /// Names of the direct children, in insertion order.
    pub fn child_names(&self) -> Vec<String> {
        todo!()
    }

    /// Looks a node up by a '/'-separated path RELATIVE to this folder, e.g. `"docs/b.txt"`. `None` when any step
    /// is missing or goes through a file, or when the path is blank.
    pub fn find(&self, path: &str) -> Option<&dyn FsNode> {
        todo!()
    }
}

impl FsNode for Directory {
    fn name(&self) -> &str {
        todo!()
    }
    fn size(&self) -> u64 {
        todo!()
    }
    fn file_count(&self) -> usize {
        todo!()
    }
    fn paths(&self) -> Vec<String> {
        todo!()
    }
    fn child(&self, name: &str) -> Option<&dyn FsNode> {
        todo!()
    }
}
