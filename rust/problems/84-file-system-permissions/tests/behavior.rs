#![allow(unused_imports, clippy::needless_range_loop)]
use p84_file_system_permissions::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

fn fs() -> Option<Arc<dyn FileSystem>> {
    let fs = create()?;
    fs.add_user("alice", &["dev", "ops"]);
    fs.add_user("bob", &["dev"]);
    fs.add_user("carol", &["hr"]);
    Some(fs)
}

#[test]
fn should_create_read_and_list_files_and_directories() {
    let fs = sut!(fs());
    fs.mkdir("root", "/home").unwrap();
    fs.chmod("root", "/home", 0o777).unwrap();

    fs.mkdir("alice", "/home/alice").unwrap();
    fs.create_file("alice", "/home/alice/b.txt", "bee").unwrap();
    fs.create_file("alice", "/home/alice/a.txt", "ay").unwrap();

    assert_eq!(fs.list("alice", "/home/alice"), Ok(vec!["a.txt".to_string(), "b.txt".to_string()]));
    assert_eq!(fs.read("alice", "/home/alice/a.txt"), Ok("ay".to_string()));
    assert_eq!(fs.list("alice", "/"), Ok(vec!["home".to_string()]));
}

#[test]
fn should_give_new_entries_the_creators_ownership_and_default_modes() {
    let fs = sut!(fs());
    fs.chmod("root", "/", 0o777).unwrap();

    fs.mkdir("alice", "/d").unwrap();
    fs.create_file("alice", "/f", "x").unwrap();

    assert_eq!(fs.owner("/d"), Ok("alice".to_string()));
    assert_eq!(fs.group("/d"), Ok("dev".to_string()));
    assert_eq!(fs.mode("/d"), Ok(0o755));
    assert_eq!(fs.mode("/f"), Ok(0o644));
}

#[test]
fn should_reject_bad_paths_missing_entries_and_duplicates() {
    let fs = sut!(fs());
    fs.chmod("root", "/", 0o777).unwrap();
    fs.create_file("alice", "/f", "x").unwrap();

    assert_eq!(fs.mkdir("alice", "relative/path"), Err(FsError::InvalidPath));
    assert_eq!(fs.mkdir("alice", "/a/../b"), Err(FsError::InvalidPath));
    assert_eq!(fs.mkdir("alice", "/a//b"), Err(FsError::InvalidPath));
    assert_eq!(fs.mkdir("alice", "/missing/child"), Err(FsError::NotFound));
    assert_eq!(fs.read("alice", "/missing"), Err(FsError::NotFound));
    assert_eq!(fs.create_file("alice", "/f", "again"), Err(FsError::AlreadyExists));
    assert_eq!(fs.mkdir("alice", "/f/sub"), Err(FsError::NotADirectory));
    assert_eq!(fs.list("alice", "/f"), Err(FsError::NotADirectory));
    assert_eq!(fs.read("alice", "/"), Err(FsError::IsADirectory));
    assert_eq!(fs.read("nobody", "/f"), Err(FsError::UnknownUser));
}

#[test]
fn should_use_owner_bits_for_the_owner_group_bits_for_group_members_and_other_bits_for_the_rest() {
    let fs = sut!(fs());
    fs.chmod("root", "/", 0o777).unwrap();
    fs.create_file("alice", "/f", "secret").unwrap();
    fs.chmod("alice", "/f", 0o640).unwrap(); // rw- r-- ---

    fs.write("alice", "/f", "secret2").unwrap();
    assert_eq!(fs.read("bob", "/f"), Ok("secret2".to_string())); // bob shares group "dev"
    assert_eq!(fs.write("bob", "/f", "x"), Err(FsError::PermissionDenied));
    assert_eq!(fs.read("carol", "/f"), Err(FsError::PermissionDenied));
    assert_eq!(fs.write("carol", "/f", "x"), Err(FsError::PermissionDenied));
}

#[test]
fn should_not_fall_back_to_group_bits_when_the_owner_bits_are_more_restrictive() {
    let fs = sut!(fs());
    fs.chmod("root", "/", 0o777).unwrap();
    fs.create_file("alice", "/f", "x").unwrap();
    fs.chmod("alice", "/f", 0o077).unwrap(); // owner has nothing, group and others everything

    assert_eq!(fs.read("alice", "/f"), Err(FsError::PermissionDenied));
    assert_eq!(fs.read("bob", "/f"), Ok("x".to_string()));
    assert_eq!(fs.read("carol", "/f"), Ok("x".to_string()));
}

#[test]
fn should_let_root_do_anything() {
    let fs = sut!(fs());
    fs.chmod("root", "/", 0o777).unwrap();
    fs.create_file("alice", "/f", "x").unwrap();
    fs.chmod("alice", "/f", 0o000).unwrap();

    assert_eq!(fs.read("root", "/f"), Ok("x".to_string()));
    fs.write("root", "/f", "y").unwrap();
    fs.chown("root", "/f", "carol", "hr").unwrap();
    assert_eq!(fs.owner("/f"), Ok("carol".to_string()));
    assert_eq!(fs.group("/f"), Ok("hr".to_string()));
}

#[test]
fn should_require_execute_on_every_directory_on_the_path() {
    let fs = sut!(fs());
    fs.chmod("root", "/", 0o777).unwrap();
    fs.mkdir("alice", "/a").unwrap();
    fs.mkdir("alice", "/a/b").unwrap();
    fs.create_file("alice", "/a/b/f", "x").unwrap();
    fs.chmod("alice", "/a", 0o744).unwrap(); // others: r-- on /a (no x)

    assert_eq!(fs.read("alice", "/a/b/f"), Ok("x".to_string()));
    assert_eq!(fs.read("carol", "/a/b/f"), Err(FsError::PermissionDenied));
    assert_eq!(fs.list("carol", "/a"), Ok(vec!["b".to_string()])); // r on /a is enough to list
    assert_eq!(fs.list("carol", "/a/b"), Err(FsError::PermissionDenied));
}

#[test]
fn should_require_write_and_execute_on_the_parent_to_create_or_delete() {
    let fs = sut!(fs());
    fs.chmod("root", "/", 0o777).unwrap();
    fs.mkdir("alice", "/shared").unwrap();
    fs.create_file("alice", "/shared/f", "x").unwrap();

    assert_eq!(fs.create_file("carol", "/shared/g", "x"), Err(FsError::PermissionDenied)); // others: r-x
    assert_eq!(fs.delete("carol", "/shared/f"), Err(FsError::PermissionDenied));
    assert_eq!(fs.mkdir("bob", "/shared/sub"), Err(FsError::PermissionDenied)); // group dev has r-x, no w

    fs.chmod("alice", "/shared", 0o775).unwrap();
    fs.create_file("bob", "/shared/g", "x").unwrap();
    fs.delete("bob", "/shared/f").unwrap();
    assert_eq!(fs.list("alice", "/shared"), Ok(vec!["g".to_string()]));
}

#[test]
fn should_only_delete_empty_directories() {
    let fs = sut!(fs());
    fs.chmod("root", "/", 0o777).unwrap();
    fs.mkdir("alice", "/d").unwrap();
    fs.create_file("alice", "/d/f", "x").unwrap();

    assert_eq!(fs.delete("alice", "/d"), Err(FsError::NotEmpty));
    fs.delete("alice", "/d/f").unwrap();
    fs.delete("alice", "/d").unwrap();
    assert_eq!(fs.list("alice", "/"), Ok(vec![]));
    assert_eq!(fs.delete("alice", "/"), Err(FsError::InvalidPath));
}

#[test]
fn should_restrict_chmod_to_owner_and_chown_to_root() {
    let fs = sut!(fs());
    fs.chmod("root", "/", 0o777).unwrap();
    fs.create_file("alice", "/f", "x").unwrap();

    assert_eq!(fs.chmod("bob", "/f", 0o777), Err(FsError::PermissionDenied));
    assert_eq!(fs.chown("alice", "/f", "bob", "dev"), Err(FsError::PermissionDenied));
    fs.chmod("alice", "/f", 0o600).unwrap();
    assert_eq!(fs.mode("/f"), Ok(0o600));
    assert_eq!(fs.chmod("alice", "/f", 0o1000), Err(FsError::InvalidMode));
}

#[test]
fn should_move_files_and_directories() {
    let fs = sut!(fs());
    fs.chmod("root", "/", 0o777).unwrap();
    fs.mkdir("alice", "/src").unwrap();
    fs.mkdir("alice", "/dst").unwrap();
    fs.mkdir("alice", "/src/inner").unwrap();
    fs.create_file("alice", "/src/inner/f", "x").unwrap();

    fs.rename("alice", "/src/inner", "/dst/moved").unwrap();

    assert_eq!(fs.read("alice", "/dst/moved/f"), Ok("x".to_string()));
    assert_eq!(fs.list("alice", "/src"), Ok(vec![]));
    assert_eq!(fs.rename("alice", "/dst", "/dst/moved/loop"), Err(FsError::InvalidPath)); // into its own subtree
    fs.create_file("alice", "/x", "1").unwrap();
    assert_eq!(fs.rename("alice", "/x", "/dst"), Err(FsError::AlreadyExists));
    fs.mkdir("alice", "/ro").unwrap();
    fs.create_file("alice", "/ro/f", "1").unwrap();
    assert_eq!(fs.rename("carol", "/ro/f", "/y"), Err(FsError::PermissionDenied)); // carol cannot remove entries from /ro
}

#[test]
fn should_let_exactly_one_thread_create_the_same_path() {
    let fs = sut!(fs());
    fs.chmod("root", "/", 0o777).unwrap();

    let wins = collect_concurrently(16, |t| fs.create_file("alice", "/contested", &format!("by {t}")).is_ok());

    assert_eq!(wins.iter().filter(|w| **w).count(), 1);
}

#[test]
fn should_keep_all_entries_when_many_threads_create_files_in_the_same_directory() {
    let fs = sut!(fs());
    fs.chmod("root", "/", 0o777).unwrap();
    fs.mkdir("alice", "/d").unwrap();
    fs.chmod("alice", "/d", 0o777).unwrap();

    run_concurrently(8, |t| {
        for i in 0..250 {
            fs.create_file(if t % 2 == 0 { "alice" } else { "bob" }, &format!("/d/f-{t}-{i}"), "x").unwrap();
        }
    });

    assert_eq!(fs.list("alice", "/d").unwrap().len(), 2_000);
}

#[test]
fn should_survive_concurrent_create_and_delete_cycles() {
    let fs = sut!(fs());
    fs.chmod("root", "/", 0o777).unwrap();
    let cycles = AtomicUsize::new(0);

    run_concurrently(8, |t| {
        for _ in 0..300 {
            let dir = format!("/w{t}");
            fs.mkdir("alice", &dir).unwrap();
            fs.create_file("alice", &format!("{dir}/f"), "x").unwrap();
            fs.rename("alice", &format!("{dir}/f"), &format!("{dir}/g")).unwrap();
            fs.delete("alice", &format!("{dir}/g")).unwrap();
            fs.delete("alice", &dir).unwrap();
            cycles.fetch_add(1, Ordering::SeqCst);
        }
    });

    assert_eq!(cycles.load(Ordering::SeqCst), 2_400);
    assert_eq!(fs.list("alice", "/"), Ok(vec![]));
}
