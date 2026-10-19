//! Server blueprints and a registry of them.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// A web server blueprint.
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

/// A database server blueprint.
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

    /// Registers a prototype under a key.
    pub fn register(&mut self, key: &str, prototype: &dyn Blueprint) -> Result<(), RegistryError> {
        todo!()
    }

    /// Creates a new server from the registered prototype.
    pub fn create(&self, key: &str, new_name: &str) -> Result<Box<dyn Blueprint>, RegistryError> {
        todo!()
    }
}
