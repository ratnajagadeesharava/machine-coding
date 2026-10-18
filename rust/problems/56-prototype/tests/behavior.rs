use p56_prototype::contract::*;
use p56_prototype::{BlueprintRegistry, DbServer, WebServer};
use std::cell::Cell;
use std::rc::Rc;

/// A blueprint type written in the test: the registry must clone it through `clone_box`, nothing else.
struct GpuServer {
    name: String,
    tags: Vec<String>,
    disks: Vec<u32>,
    copies: Rc<Cell<u32>>,
}

impl GpuServer {
    fn new(name: &str, copies: Rc<Cell<u32>>) -> Self {
        GpuServer { name: name.into(), tags: vec![], disks: vec![1000], copies }
    }
}

impl Blueprint for GpuServer {
    fn name(&self) -> &str {
        &self.name
    }
    fn rename(&mut self, new_name: &str) {
        self.name = new_name.into();
    }
    fn kind(&self) -> &str {
        "gpu"
    }
    fn add_tag(&mut self, tag: &str) {
        if !tag.trim().is_empty() && !self.tags.iter().any(|t| t == tag) {
            self.tags.push(tag.into());
        }
    }
    fn tags(&self) -> Vec<String> {
        self.tags.clone()
    }
    fn disk_sizes_gb(&self) -> Vec<u32> {
        self.disks.clone()
    }
    fn resize_disk(&mut self, index: usize, gb: u32) -> Result<(), BlueprintError> {
        *self.disks.get_mut(index).ok_or(BlueprintError::InvalidDisk)? = gb;
        Ok(())
    }
    fn clone_box(&self) -> Box<dyn Blueprint> {
        self.copies.set(self.copies.get() + 1);
        Box::new(GpuServer { name: self.name.clone(), tags: self.tags.clone(), disks: self.disks.clone(), copies: self.copies.clone() })
    }
}

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn should_start_a_web_server_small() {
    let web = WebServer::new("w1").unwrap();

    assert_eq!(web.name(), "w1");
    assert_eq!(web.kind(), "web");
    assert_eq!(web.disk_sizes_gb(), vec![20]);
    assert!(web.tags().is_empty());
}

#[test]
fn should_start_a_database_server_with_two_disks() {
    let db = DbServer::new("d1").unwrap();

    assert_eq!(db.kind(), "db");
    assert_eq!(db.disk_sizes_gb(), vec![100, 500]);
}

#[test]
fn should_copy_the_state_and_the_concrete_type() {
    let mut db = DbServer::new("d1").unwrap();
    db.add_tag("prod");
    db.add_tag("eu");
    db.resize_disk(1, 750).unwrap();

    let clone = db.clone_box();

    assert_eq!(clone.name(), "d1");
    assert_eq!(clone.kind(), "db");
    assert_eq!(clone.tags(), strs(&["prod", "eu"]));
    assert_eq!(clone.disk_sizes_gb(), vec![100, 750]);
}

#[test]
fn should_make_the_copy_completely_independent_of_the_original() {
    let mut original = WebServer::new("w1").unwrap();
    original.add_tag("blue");
    let mut clone = original.clone_box();

    clone.add_tag("canary");
    clone.rename("w1-clone");
    clone.resize_disk(0, 99).unwrap();
    original.add_tag("green");
    original.resize_disk(0, 40).unwrap();

    assert_eq!(original.name(), "w1");
    assert_eq!(original.tags(), strs(&["blue", "green"]));
    assert_eq!(original.disk_sizes_gb(), vec![40]);
    assert_eq!(clone.name(), "w1-clone");
    assert_eq!(clone.tags(), strs(&["blue", "canary"]));
    assert_eq!(clone.disk_sizes_gb(), vec![99]);
}

#[test]
fn should_stamp_out_independent_servers_from_a_registered_prototype() {
    let mut registry = BlueprintRegistry::new();
    let mut prototype = DbServer::new("template").unwrap();
    prototype.add_tag("prod");
    registry.register("db-prod", &prototype).unwrap();

    let mut one = registry.create("db-prod", "db-1").unwrap();
    let two = registry.create("db-prod", "db-2").unwrap();
    one.add_tag("only-one");
    one.resize_disk(0, 1).unwrap();

    assert_eq!(one.name(), "db-1");
    assert_eq!(two.name(), "db-2");
    assert_eq!(two.kind(), "db");
    assert_eq!(two.tags(), strs(&["prod"]));
    assert_eq!(two.disk_sizes_gb(), vec![100, 500]);
    assert_eq!(registry.create("db-prod", "db-3").unwrap().tags(), strs(&["prod"]));
}

#[test]
fn should_keep_the_catalogue_safe_from_later_changes_to_what_was_registered() {
    let mut registry = BlueprintRegistry::new();
    let mut prototype = WebServer::new("template").unwrap();
    registry.register("web", &prototype).unwrap();

    prototype.add_tag("late");
    prototype.resize_disk(0, 999).unwrap();

    let created = registry.create("web", "w1").unwrap();
    assert!(created.tags().is_empty());
    assert_eq!(created.disk_sizes_gb(), vec![20]);
}

#[test]
fn should_clone_through_clone_box_for_any_blueprint_type() {
    let copies = Rc::new(Cell::new(0));
    let mut registry = BlueprintRegistry::new();
    let mut gpu = GpuServer::new("template", copies.clone());
    gpu.add_tag("ml");

    registry.register("gpu", &gpu).unwrap();
    let made = registry.create("gpu", "g1").unwrap();

    assert_eq!(copies.get(), 2);
    assert_eq!(made.kind(), "gpu");
    assert_eq!(made.name(), "g1");
    assert_eq!(made.tags(), strs(&["ml"]));
    assert_eq!(gpu.name(), "template");
}

#[test]
fn should_replace_an_earlier_prototype_with_the_same_key() {
    let mut registry = BlueprintRegistry::new();
    registry.register("x", &WebServer::new("a").unwrap()).unwrap();
    registry.register("x", &DbServer::new("b").unwrap()).unwrap();

    assert_eq!(registry.create("x", "n").unwrap().kind(), "db");
}

#[test]
fn should_reject_bad_input() {
    let mut registry = BlueprintRegistry::new();
    let mut web = WebServer::new("w").unwrap();
    registry.register("web", &web).unwrap();

    assert!(matches!(registry.create("nope", "n"), Err(RegistryError::UnknownKey)));
    assert!(matches!(registry.create("web", " "), Err(RegistryError::BlankName)));
    assert_eq!(registry.register(" ", &web), Err(RegistryError::BlankKey));
    assert_eq!(web.resize_disk(5, 10), Err(BlueprintError::InvalidDisk));
    assert_eq!(web.resize_disk(0, 0), Err(BlueprintError::InvalidDisk));
    assert!(matches!(WebServer::new(" "), Err(BlueprintError::InvalidName)));
    assert_eq!(web.disk_sizes_gb(), vec![20]);
}
