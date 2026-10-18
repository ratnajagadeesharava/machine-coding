use p60_composite::contract::*;
use p60_composite::{Directory, File};

/// A node type that exists only in the test: folders must treat it like any other node.
struct Shortcut(String);

impl FsNode for Shortcut {
    fn name(&self) -> &str {
        &self.0
    }
    fn size(&self) -> u64 {
        4
    }
    fn file_count(&self) -> usize {
        1
    }
    fn paths(&self) -> Vec<String> {
        vec![self.0.clone()]
    }
}

fn file(name: &str, size: u64) -> Box<dyn FsNode> {
    Box::new(File::new(name, size).unwrap())
}

fn sample_tree() -> Directory {
    let mut deep = Directory::new("deep").unwrap();
    deep.add(file("c.txt", 50)).unwrap();
    let mut docs = Directory::new("docs").unwrap();
    docs.add(file("b.txt", 250)).unwrap();
    docs.add(Box::new(deep)).unwrap();
    let mut root = Directory::new("root").unwrap();
    root.add(file("a.txt", 100)).unwrap();
    root.add(Box::new(docs)).unwrap();
    root.add(Box::new(Directory::new("empty").unwrap())).unwrap();
    root
}

#[test]
fn should_treat_a_file_as_a_single_node() {
    let f = File::new("a.txt", 100).unwrap();

    assert_eq!(f.name(), "a.txt");
    assert_eq!(f.size(), 100);
    assert_eq!(f.file_count(), 1);
    assert_eq!(f.paths(), vec!["a.txt"]);
}

#[test]
fn should_sum_sizes_and_count_files_recursively() {
    let root = sample_tree();

    assert_eq!(root.size(), 400);
    assert_eq!(root.file_count(), 3);
    let docs = root.find("docs").unwrap();
    assert_eq!(docs.size(), 300);
    assert_eq!(docs.file_count(), 2);
}

#[test]
fn should_list_every_file_with_its_path_in_insertion_order() {
    assert_eq!(sample_tree().paths(), vec!["root/a.txt", "root/docs/b.txt", "root/docs/deep/c.txt"]);
    let nothing = Directory::new("nothing").unwrap();
    assert!(nothing.paths().is_empty());
    assert_eq!(nothing.size(), 0);
}

#[test]
fn should_find_nodes_by_relative_path() {
    let root = sample_tree();

    assert_eq!(root.find("a.txt").unwrap().size(), 100);
    assert_eq!(root.find("docs/deep/c.txt").unwrap().name(), "c.txt");
    assert_eq!(root.find("docs/deep").unwrap().file_count(), 1);
    assert!(root.find("docs/missing.txt").is_none());
    assert!(root.find("a.txt/inside").is_none());
    assert!(root.find(" ").is_none());
}

#[test]
fn should_accept_node_types_it_has_never_seen() {
    let mut root = sample_tree();

    root.add(Box::new(Shortcut("link".to_string()))).unwrap();

    assert_eq!(root.size(), 404);
    assert_eq!(root.file_count(), 4);
    assert_eq!(root.paths().last().unwrap(), "root/link");
    assert!(root.find("link").is_some());
}

#[test]
fn should_remove_children_by_name() {
    let mut root = sample_tree();

    assert!(root.remove("docs").is_some());
    assert!(root.remove("docs").is_none());

    assert_eq!(root.size(), 100);
    assert_eq!(root.child_names(), vec!["a.txt", "empty"]);
}

#[test]
fn should_refuse_duplicate_names_and_leave_the_folder_unchanged() {
    let mut root = sample_tree();

    assert_eq!(root.add(file("a.txt", 1)), Err(FsError::DuplicateName));
    assert_eq!(root.add(Box::new(Directory::new("docs").unwrap())), Err(FsError::DuplicateName));

    assert_eq!(root.file_count(), 3);
    assert_eq!(root.child_names().len(), 3);
}

#[test]
fn should_validate_names() {
    assert!(matches!(File::new("", 1), Err(FsError::InvalidName)));
    assert!(matches!(File::new("a/b", 1), Err(FsError::InvalidName)));
    assert!(matches!(Directory::new(" "), Err(FsError::InvalidName)));
    assert!(matches!(Directory::new("x/y"), Err(FsError::InvalidName)));
    assert_eq!(File::new("empty", 0).unwrap().size(), 0);
}
