#![allow(unused_imports, clippy::needless_range_loop)]
use p86_document_versioning::{contract::*, create};
use playground_support::sut;

#[test]
fn should_handle_a_large_document_with_many_versions() {
    let s = sut!(create());
    let mut base = String::new();
    for i in 0..2_000 {
        base.push_str(&format!("line {i}\n"));
    }
    s.create("big", "ann", &base).unwrap();

    for v in 1..=200 {
        s.edit("big", "ann", v, &format!("{base}extra {v}"))
            .unwrap();
    }

    assert_eq!(s.latest("big"), Ok(201));
    assert!(s
        .diff("big", 1, 201)
        .unwrap()
        .contains(&"+ extra 200".to_string()));
}

#[test]
fn should_merge_two_stale_edits_against_a_moved_base() {
    let s = sut!(create());
    s.create("d", "ann", "1\n2\n3\n4\n5\n6").unwrap();
    s.edit("d", "ann", 1, "1\n2\n3\n4\n5\nsix").unwrap(); // v2
    s.edit("d", "bob", 1, "one\n2\n3\n4\n5\n6").unwrap(); // stale base 1 -> merged v3
    s.edit("d", "cy", 2, "1\n2\nthree\n4\n5\nsix").unwrap(); // stale base 2 -> merged v4

    assert_eq!(
        s.content("d", 4),
        Ok("one\n2\nthree\n4\n5\nsix".to_string())
    );
    assert_eq!(
        s.edit("d", "dee", 1, "1\n2\n3\n4\n5\nSIX"),
        Err(DocError::Conflict)
    );
}
