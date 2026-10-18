//! Prototype — new servers are stamped out by cloning registered blueprints through `Box<dyn Blueprint>`.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// kind() is "web"; starts with one 20 GB disk and no tags. Blank name -> `BlueprintError::InvalidName`.
pub struct WebServer;

impl WebServer {
    pub fn new(name: &str) -> Result<WebServer, BlueprintError> {
        todo!()
    }
}

impl Blueprint for WebServer {
    fn name(&self) -> &str {
        todo!()
    }
    fn rename(&mut self, new_name: &str) {
        todo!()
    }
    fn kind(&self) -> &str {
        todo!()
    }
    fn add_tag(&mut self, tag: &str) {
        todo!()
    }
    fn tags(&self) -> Vec<String> {
        todo!()
    }
    fn disk_sizes_gb(&self) -> Vec<u32> {
        todo!()
    }
    fn resize_disk(&mut self, index: usize, new_size_gb: u32) -> Result<(), BlueprintError> {
        todo!()
    }
    fn clone_box(&self) -> Box<dyn Blueprint> {
        todo!()
    }
}

/// kind() is "db"; starts with two disks, 100 GB and 500 GB, and no tags. Blank name -> `InvalidName`.
pub struct DbServer;

impl DbServer {
    pub fn new(name: &str) -> Result<DbServer, BlueprintError> {
        todo!()
    }
}

impl Blueprint for DbServer {
    fn name(&self) -> &str {
        todo!()
    }
    fn rename(&mut self, new_name: &str) {
        todo!()
    }
    fn kind(&self) -> &str {
        todo!()
    }
    fn add_tag(&mut self, tag: &str) {
        todo!()
    }
    fn tags(&self) -> Vec<String> {
        todo!()
    }
    fn disk_sizes_gb(&self) -> Vec<u32> {
        todo!()
    }
    fn resize_disk(&mut self, index: usize, new_size_gb: u32) -> Result<(), BlueprintError> {
        todo!()
    }
    fn clone_box(&self) -> Box<dyn Blueprint> {
        todo!()
    }
}

/// A catalogue of ready-made blueprints.
pub struct BlueprintRegistry;

impl BlueprintRegistry {
    pub fn new() -> BlueprintRegistry {
        todo!()
    }

    /// Stores a COPY (`clone_box`) of the prototype under the key, replacing any earlier one. Blank key -> `BlankKey`.
    pub fn register(&mut self, key: &str, prototype: &dyn Blueprint) -> Result<(), RegistryError> {
        todo!()
    }

    /// A new independent copy of the registered prototype renamed to `new_name`. Unknown key -> `UnknownKey`;
    /// blank name -> `BlankName`.
    pub fn create(&self, key: &str, new_name: &str) -> Result<Box<dyn Blueprint>, RegistryError> {
        todo!()
    }
}
