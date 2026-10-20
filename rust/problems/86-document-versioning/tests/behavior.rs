#![allow(unused_imports, clippy::needless_range_loop)]
use p86_document_versioning::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::sync::atomic::{AtomicUsize, Ordering};

const FIVE: &str = "l1\nl2\nl3\nl4\nl5";

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn should_create_and_read_versions() {
    let s = sut!(create());

    s.create("d", "ann", "hello").unwrap();

    assert_eq!(s.latest("d"), Ok(1));
    assert_eq!(s.content("d", 1), Ok("hello".to_string()));
    assert_eq!(s.history("d"), Ok(strs(&["1:ann"])));
}

#[test]
fn should_reject_duplicates_and_unknown_documents_or_versions() {
    let s = sut!(create());
    s.create("d", "ann", "x").unwrap();

    assert_eq!(s.create("d", "bob", "y"), Err(DocError::AlreadyExists));
    assert_eq!(s.latest("nope"), Err(DocError::UnknownDocument));
    assert_eq!(s.content("d", 2), Err(DocError::UnknownVersion));
    assert_eq!(s.content("d", 0), Err(DocError::UnknownVersion));
    assert_eq!(s.edit("d", "bob", 2, "z"), Err(DocError::UnknownVersion));
    assert_eq!(s.edit("d", "bob", 0, "z"), Err(DocError::UnknownVersion));
}

#[test]
fn should_append_a_version_per_edit_and_keep_old_ones() {
    let s = sut!(create());
    s.create("d", "ann", "one").unwrap();

    assert_eq!(s.edit("d", "bob", 1, "two"), Ok(2));
    assert_eq!(s.edit("d", "ann", 2, "three"), Ok(3));

    assert_eq!(s.content("d", 1), Ok("one".to_string()));
    assert_eq!(s.content("d", 2), Ok("two".to_string()));
    assert_eq!(s.history("d"), Ok(strs(&["1:ann", "2:bob", "3:ann"])));
}

#[test]
fn should_not_create_a_version_for_an_unchanged_edit() {
    let s = sut!(create());
    s.create("d", "ann", "same").unwrap();

    assert_eq!(s.edit("d", "bob", 1, "same"), Ok(1));

    assert_eq!(s.history("d").unwrap().len(), 1);
}

#[test]
fn should_restore_an_old_version_as_a_new_version() {
    let s = sut!(create());
    s.create("d", "ann", "one").unwrap();
    s.edit("d", "ann", 1, "two").unwrap();

    assert_eq!(s.restore("d", "bob", 1), Ok(3));

    assert_eq!(s.content("d", 3), Ok("one".to_string()));
    assert_eq!(s.history("d"), Ok(strs(&["1:ann", "2:ann", "3:bob"])));
    assert_eq!(s.restore("d", "bob", 3), Ok(3)); // already the latest content
}

#[test]
fn should_diff_line_by_line() {
    let s = sut!(create());
    s.create("d", "ann", "a\nb\nc\nd").unwrap();
    s.edit("d", "ann", 1, "a\nB\nc\nd\ne").unwrap();

    assert_eq!(
        s.diff("d", 1, 2),
        Ok(strs(&["  a", "- b", "+ B", "  c", "  d", "+ e"]))
    );
    assert_eq!(
        s.diff("d", 2, 1),
        Ok(strs(&["  a", "- B", "+ b", "  c", "  d", "- e"]))
    );
    assert_eq!(s.diff("d", 1, 1), Ok(strs(&["  a", "  b", "  c", "  d"])));
}

#[test]
fn should_merge_stale_edits_that_touch_different_lines() {
    let s = sut!(create());
    s.create("d", "ann", FIVE).unwrap();
    s.edit("d", "ann", 1, "L1\nl2\nl3\nl4\nl5").unwrap();

    let merged = s.edit("d", "bob", 1, "l1\nl2\nl3\nl4\nL5");

    assert_eq!(merged, Ok(3));
    assert_eq!(s.content("d", 3), Ok("L1\nl2\nl3\nl4\nL5".to_string()));
    assert_eq!(s.history("d"), Ok(strs(&["1:ann", "2:ann", "3:bob"])));
}

#[test]
fn should_merge_insertions_and_deletions_at_different_places() {
    let s = sut!(create());
    s.create("d", "ann", FIVE).unwrap();
    s.edit("d", "ann", 1, "l1\nnew\nl2\nl3\nl4\nl5").unwrap(); // inserts before l2

    s.edit("d", "bob", 1, "l1\nl2\nl3\nl5").unwrap(); // deletes l4

    let latest = s.latest("d").unwrap();
    assert_eq!(
        s.content("d", latest),
        Ok("l1\nnew\nl2\nl3\nl5".to_string())
    );
}

#[test]
fn should_merge_adjacent_changes_without_conflict() {
    let s = sut!(create());
    s.create("d", "ann", FIVE).unwrap();
    s.edit("d", "ann", 1, "l1\nl2\nX\nl4\nl5").unwrap(); // changes l3

    s.edit("d", "bob", 1, "l1\nY\nl3\nl4\nl5").unwrap(); // changes l2 (adjacent, different line)

    let latest = s.latest("d").unwrap();
    assert_eq!(s.content("d", latest), Ok("l1\nY\nX\nl4\nl5".to_string()));
}

#[test]
fn should_report_a_conflict_when_both_change_the_same_line() {
    let s = sut!(create());
    s.create("d", "ann", FIVE).unwrap();
    s.edit("d", "ann", 1, "l1\nl2\nANN\nl4\nl5").unwrap();

    assert_eq!(
        s.edit("d", "bob", 1, "l1\nl2\nBOB\nl4\nl5"),
        Err(DocError::Conflict)
    );

    assert_eq!(s.latest("d"), Ok(2));
    assert_eq!(s.content("d", 2), Ok("l1\nl2\nANN\nl4\nl5".to_string()));
}

#[test]
fn should_report_a_conflict_for_two_insertions_at_the_same_place() {
    let s = sut!(create());
    s.create("d", "ann", "a\nb").unwrap();
    s.edit("d", "ann", 1, "a\nx\nb").unwrap();

    assert_eq!(s.edit("d", "bob", 1, "a\ny\nb"), Err(DocError::Conflict));
}

#[test]
fn should_accept_the_same_change_made_twice() {
    let s = sut!(create());
    s.create("d", "ann", FIVE).unwrap();
    s.edit("d", "ann", 1, "l1\nl2\nSAME\nl4\nl5").unwrap();

    assert_eq!(s.edit("d", "bob", 1, "l1\nl2\nSAME\nl4\nl5"), Ok(2));
    assert_eq!(s.history("d").unwrap().len(), 2);
}

#[test]
fn should_merge_concurrent_edits_of_different_lines_without_losing_any() {
    let s = sut!(create());
    s.create("d", "root", "0\n1\n2\n3\n4\n5\n6\n7").unwrap();

    run_concurrently(8, |t| {
        let mut lines: Vec<String> = "0\n1\n2\n3\n4\n5\n6\n7"
            .split('\n')
            .map(String::from)
            .collect();
        lines[t] = format!("edited-{t}");
        s.edit("d", &format!("user{t}"), 1, &lines.join("\n"))
            .unwrap();
    });

    assert_eq!(s.latest("d"), Ok(9));
    assert_eq!(
        s.content("d", 9),
        Ok(
            "edited-0\nedited-1\nedited-2\nedited-3\nedited-4\nedited-5\nedited-6\nedited-7"
                .to_string()
        )
    );
    assert_eq!(s.history("d").unwrap().len(), 9);
}

#[test]
fn should_let_exactly_one_concurrent_edit_of_the_same_line_win() {
    let s = sut!(create());
    s.create("d", "root", "a\nb\nc").unwrap();

    let wins = collect_concurrently(8, |t| {
        s.edit(
            "d",
            &format!("user{t}"),
            1,
            &format!("changed-by-{t}\nb\nc"),
        )
        .is_ok()
    });

    assert_eq!(wins.iter().filter(|w| **w).count(), 1);
    assert_eq!(s.latest("d"), Ok(2));
}

#[test]
fn should_keep_versions_gapless_under_concurrent_writers_and_readers() {
    let s = sut!(create());
    s.create("d", "root", "start").unwrap();
    let conflicts = AtomicUsize::new(0);

    run_concurrently(8, |t| {
        for i in 0..50 {
            let base = s.latest("d").unwrap();
            if s.edit("d", &format!("u{t}"), base, &format!("text-{t}-{i}"))
                == Err(DocError::Conflict)
            {
                conflicts.fetch_add(1, Ordering::SeqCst);
            }
            let latest = s.latest("d").unwrap();
            s.content("d", latest).unwrap(); // readers must always find the latest version
        }
    });

    let latest = s.latest("d").unwrap();
    let history = s.history("d").unwrap();
    assert_eq!(history.len(), latest);
    for v in 1..=latest {
        assert!(history[v - 1].starts_with(&format!("{v}:")));
    }
}

#[test]
fn should_stay_correct_over_100_runs() {
    for run in 0..100 {
        let s = sut!(create());
        s.create("d", "root", "0\n1\n2\n3").unwrap();

        run_concurrently(4, |t| {
            let mut lines: Vec<String> = "0\n1\n2\n3".split('\n').map(String::from).collect();
            lines[t] = format!("e{t}");
            s.edit("d", &format!("u{t}"), 1, &lines.join("\n")).unwrap();
        });

        let latest = s.latest("d").unwrap();
        assert_eq!(
            s.content("d", latest),
            Ok("e0\ne1\ne2\ne3".to_string()),
            "run {run}"
        );
    }
}
