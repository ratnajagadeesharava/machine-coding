#![allow(unused_imports, clippy::needless_range_loop)]
use p84_file_system_permissions::{contract::*, create};
use playground_support::sut;

#[test]
fn should_handle_a_deep_tree() {
    let fs = sut!(create());
    fs.chmod("root", "/", 0o777).unwrap();
    let mut path = String::new();
    for i in 0..500 {
        path.push_str(&format!("/d{i}"));
        fs.mkdir("root", &path).unwrap();
    }
    fs.create_file("root", &format!("{path}/leaf"), "deep")
        .unwrap();

    assert_eq!(
        fs.read("root", &format!("{path}/leaf")),
        Ok("deep".to_string())
    );
}

#[test]
fn should_pick_the_first_matching_class_not_the_most_permissive_one() {
    let fs = sut!(create());
    fs.add_user("alice", &["dev"]);
    fs.add_user("bob", &["dev"]);
    fs.chmod("root", "/", 0o777).unwrap();
    fs.create_file("alice", "/f", "x").unwrap();
    fs.chmod("alice", "/f", 0o407).unwrap(); // owner r--, group ---, other rwx

    assert_eq!(fs.read("alice", "/f"), Ok("x".to_string()));
    assert_eq!(fs.read("bob", "/f"), Err(FsError::PermissionDenied));
}
