#![allow(unused_imports, clippy::needless_range_loop)]
use p108_document_comments::{contract::*, create};
use playground_support::{collect_concurrently, sut};
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, Ordering};

#[test]
fn should_give_readers_a_prefix_of_the_final_history_while_writers_run() {
    let s = sut!(create(&["doc-1"]));
    let writes = 1_500;
    let done = AtomicBool::new(false);

    let snapshots = collect_concurrently(3, |t| {
        let mut seen: Vec<Vec<String>> = Vec::new();
        if t == 0 {
            for i in 0..writes {
                s.add_comment("doc-1", "w", &format!("c{i}")).unwrap();
            }
            done.store(true, Ordering::SeqCst);
        } else {
            while !done.load(Ordering::SeqCst) {
                seen.push(s.comments("doc-1").unwrap());
            }
        }
        seen
    });

    let final_list = s.comments("doc-1").unwrap();
    assert_eq!(final_list.len(), writes);
    for per_reader in &snapshots {
        for snap in per_reader {
            assert_eq!(snap.as_slice(), &final_list[..snap.len()]);
        }
    }
}

#[test]
fn should_leave_no_holes_in_the_id_sequence_when_rejected_adds_race_with_good_ones() {
    let s = sut!(create(&["doc-1", "doc-2"]));

    let accepted = collect_concurrently(8, |t| {
        let mut mine = Vec::new();
        for i in 0..300 {
            match i % 4 {
                0 => mine.push(s.add_comment("doc-1", &format!("r{t}"), "ok").unwrap()),
                1 => assert!(s.add_comment("nope", "r", "x").is_err()),
                2 => mine.push(s.add_comment("doc-2", &format!("r{t}"), "ok").unwrap()),
                _ => assert!(s.add_comment("doc-1", " ", "x").is_err()),
            }
        }
        mine
    });

    let ids: BTreeSet<usize> = accepted
        .iter()
        .flatten()
        .map(|id| id.parse().unwrap())
        .collect();
    assert_eq!(ids, (1..=8 * 150).collect::<BTreeSet<_>>());
    let total = s.comments("doc-1").unwrap().len() + s.comments("doc-2").unwrap().len();
    assert_eq!(total, 8 * 150);
}

#[test]
fn should_not_change_a_snapshot_taken_before_concurrent_adds() {
    let s = sut!(create(&["doc-1"]));
    s.add_comment("doc-1", "alice", "seed").unwrap();
    let snapshot = s.comments("doc-1").unwrap();

    collect_concurrently(8, |t| {
        for i in 0..200 {
            s.add_comment("doc-1", &format!("w{t}"), &format!("c{i}"))
                .unwrap();
        }
    });

    assert_eq!(snapshot, vec!["alice: seed"]);
    let now = s.comments("doc-1").unwrap();
    assert_eq!(now.len(), 1_601);
    assert_eq!(now[0], "alice: seed");
}

#[test]
fn should_keep_each_threads_own_order_across_many_documents() {
    let names: Vec<String> = (0..50).map(|i| format!("doc-{i}")).collect();
    let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let s = sut!(create(&refs));

    collect_concurrently(8, |t| {
        for i in 0..500usize {
            s.add_comment(
                &names[(t * 7 + i) % names.len()],
                &format!("r{t}"),
                &i.to_string(),
            )
            .unwrap();
        }
    });

    let mut total = 0;
    for d in &names {
        let list = s.comments(d).unwrap();
        total += list.len();
        for t in 0..8 {
            let prefix = format!("r{t}: ");
            let mut last: i64 = -1;
            for c in list.iter().filter(|c| c.starts_with(&prefix)) {
                let n: i64 = c[prefix.len()..].parse().unwrap();
                assert!(n > last);
                last = n;
            }
        }
    }
    assert_eq!(total, 8 * 500);
}

#[test]
fn should_handle_a_huge_history_in_one_document() {
    let s = sut!(create(&["doc-1", "doc-2"]));
    for i in 0..50_000 {
        s.add_comment("doc-1", &format!("r{}", i % 10), &format!("c{i}"))
            .unwrap();
    }

    let list = s.comments("doc-1").unwrap();
    assert_eq!(list.len(), 50_000);
    assert_eq!(list[0], "r0: c0");
    assert_eq!(list[49_999], "r9: c49999");
    assert_eq!(s.comments("doc-2"), Ok(vec![]));
    assert_eq!(s.add_comment("doc-2", "x", "y"), Ok("50001".to_string()));
}

#[test]
fn should_not_create_a_document_by_failing_to_add_to_it() {
    let s = sut!(create(&["doc-1"]));

    assert_eq!(
        s.add_comment("ghost", "alice", "boo"),
        Err(CommentError::UnknownDocument)
    );
    assert_eq!(s.comments("ghost"), Err(CommentError::UnknownDocument));
    assert_eq!(
        s.add_comment("ghost", "alice", "boo"),
        Err(CommentError::UnknownDocument)
    );
}

#[test]
fn should_not_parse_the_reviewer_or_the_text_when_they_contain_the_separator() {
    let s = sut!(create(&["doc-1"]));
    s.add_comment("doc-1", "a: b", "c: d").unwrap();
    s.add_comment("doc-1", "x", ": ").unwrap();

    assert_eq!(s.comments("doc-1").unwrap(), vec!["a: b: c: d", "x: : "]);
}
