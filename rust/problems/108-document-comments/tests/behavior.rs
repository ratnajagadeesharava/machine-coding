#![allow(unused_imports, clippy::needless_range_loop)]
use p108_document_comments::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::collections::BTreeSet;
use std::sync::Arc;

fn service(docs: &[&str]) -> Option<Arc<dyn CommentService>> {
    create(docs)
}

#[test]
fn should_return_an_empty_list_for_a_document_without_comments() {
    let s = sut!(service(&["doc-1"]));

    assert_eq!(s.comments("doc-1"), Ok(vec![]));
}

#[test]
fn should_return_comment_ids_from_one_sequence_across_documents() {
    let s = sut!(service(&["doc-1", "doc-2"]));

    assert_eq!(s.add_comment("doc-1", "alice", "a"), Ok("1".to_string()));
    assert_eq!(s.add_comment("doc-2", "bob", "b"), Ok("2".to_string()));
    assert_eq!(s.add_comment("doc-1", "carol", "c"), Ok("3".to_string()));
}

#[test]
fn should_return_comments_in_insertion_order() {
    let s = sut!(service(&["doc-1"]));
    s.add_comment("doc-1", "alice", "Fix the typo").unwrap();
    s.add_comment("doc-1", "bob", "Add a diagram").unwrap();
    s.add_comment("doc-1", "carol", "Looks good").unwrap();

    assert_eq!(
        s.comments("doc-1").unwrap(),
        vec![
            "alice: Fix the typo",
            "bob: Add a diagram",
            "carol: Looks good"
        ]
    );
}

#[test]
fn should_format_each_comment_as_reviewer_colon_space_text() {
    let s = sut!(service(&["doc-1"]));
    s.add_comment("doc-1", "alice", "hello").unwrap();

    assert_eq!(s.comments("doc-1").unwrap(), vec!["alice: hello"]);
}

#[test]
fn should_keep_comments_of_different_documents_separate() {
    let s = sut!(service(&["doc-1", "doc-2"]));
    s.add_comment("doc-1", "alice", "one").unwrap();
    s.add_comment("doc-2", "bob", "two").unwrap();
    s.add_comment("doc-1", "carol", "three").unwrap();

    assert_eq!(
        s.comments("doc-1").unwrap(),
        vec!["alice: one", "carol: three"]
    );
    assert_eq!(s.comments("doc-2").unwrap(), vec!["bob: two"]);
}

#[test]
fn should_allow_the_same_reviewer_to_comment_repeatedly() {
    let s = sut!(service(&["doc-1"]));
    s.add_comment("doc-1", "alice", "first").unwrap();
    s.add_comment("doc-1", "alice", "second").unwrap();

    assert_eq!(
        s.comments("doc-1").unwrap(),
        vec!["alice: first", "alice: second"]
    );
}

#[test]
fn should_keep_identical_comments_as_separate_entries() {
    let s = sut!(service(&["doc-1"]));
    let first = s.add_comment("doc-1", "alice", "same").unwrap();
    let second = s.add_comment("doc-1", "alice", "same").unwrap();

    assert_ne!(first, second);
    assert_eq!(
        s.comments("doc-1").unwrap(),
        vec!["alice: same", "alice: same"]
    );
}

#[test]
fn should_store_text_verbatim() {
    let s = sut!(service(&["doc-1"]));
    s.add_comment(
        "doc-1",
        "alice",
        "  see: line 3\nand \u{fc}\u{e9} \u{4e2d}  ",
    )
    .unwrap();

    assert_eq!(
        s.comments("doc-1").unwrap(),
        vec!["alice:   see: line 3\nand \u{fc}\u{e9} \u{4e2d}  "]
    );
}

#[test]
fn should_reject_adding_to_an_unknown_document() {
    let s = sut!(service(&["doc-1"]));

    assert_eq!(
        s.add_comment("doc-9", "alice", "hi"),
        Err(CommentError::UnknownDocument)
    );
}

#[test]
fn should_reject_reading_an_unknown_document() {
    let s = sut!(service(&["doc-1"]));

    assert_eq!(s.comments("doc-9"), Err(CommentError::UnknownDocument));
}

#[test]
fn should_treat_document_ids_as_exact_strings() {
    let s = sut!(service(&["doc-1"]));

    assert_eq!(s.comments("DOC-1"), Err(CommentError::UnknownDocument));
    assert_eq!(
        s.add_comment(" doc-1", "alice", "hi"),
        Err(CommentError::UnknownDocument)
    );
}

#[test]
fn should_reject_blank_reviewer_and_blank_text() {
    let s = sut!(service(&["doc-1"]));

    for blank in ["", " ", "\t\n"] {
        assert_eq!(
            s.add_comment("doc-1", blank, "text"),
            Err(CommentError::InvalidArgument)
        );
        assert_eq!(
            s.add_comment("doc-1", "alice", blank),
            Err(CommentError::InvalidArgument)
        );
    }
    assert_eq!(s.comments("doc-1"), Ok(vec![]));
}

#[test]
fn should_reject_blank_document_ids() {
    let s = sut!(service(&["doc-1"]));

    for blank in ["", "  "] {
        assert_eq!(
            s.add_comment(blank, "alice", "hi"),
            Err(CommentError::InvalidArgument)
        );
        assert_eq!(s.comments(blank), Err(CommentError::InvalidArgument));
    }
}

#[test]
fn should_not_consume_an_id_or_leave_a_trace_when_a_call_is_rejected() {
    let s = sut!(service(&["doc-1"]));

    assert!(s.add_comment("doc-9", "alice", "hi").is_err());
    assert!(s.add_comment("doc-1", " ", "hi").is_err());
    assert!(s.add_comment("doc-1", "alice", "").is_err());

    assert_eq!(s.comments("doc-1"), Ok(vec![]));
    assert_eq!(s.add_comment("doc-1", "alice", "ok"), Ok("1".to_string()));
    assert_eq!(s.comments("doc-9"), Err(CommentError::UnknownDocument));
}

#[test]
fn should_report_blank_arguments_even_for_an_unknown_document() {
    let s = sut!(service(&["doc-1"]));

    assert_eq!(
        s.add_comment("nope", " ", "x"),
        Err(CommentError::InvalidArgument)
    );
    assert_eq!(
        s.add_comment("nope", "alice", " "),
        Err(CommentError::InvalidArgument)
    );
}

#[test]
fn should_not_change_a_list_that_was_already_returned() {
    let s = sut!(service(&["doc-1"]));
    s.add_comment("doc-1", "alice", "first").unwrap();
    let before = s.comments("doc-1").unwrap();

    s.add_comment("doc-1", "bob", "second").unwrap();

    assert_eq!(before, vec!["alice: first"]);
    assert_eq!(
        s.comments("doc-1").unwrap(),
        vec!["alice: first", "bob: second"]
    );
}

#[test]
fn should_not_let_callers_change_the_service_through_a_returned_list() {
    let s = sut!(service(&["doc-1"]));
    s.add_comment("doc-1", "alice", "first").unwrap();
    let mut returned = s.comments("doc-1").unwrap();

    returned.clear();
    returned.push("mallory: injected".to_string());

    assert_eq!(s.comments("doc-1").unwrap(), vec!["alice: first"]);
}

#[test]
fn should_refuse_everything_when_created_without_documents() {
    let s = sut!(service(&[]));

    assert_eq!(
        s.add_comment("doc-1", "alice", "hi"),
        Err(CommentError::UnknownDocument)
    );
    assert_eq!(s.comments("doc-1"), Err(CommentError::UnknownDocument));
}

#[test]
fn should_lose_no_comment_when_many_threads_add_to_one_document() {
    let s = sut!(service(&["doc-1"]));
    let threads = 16;
    let per_thread = 100;

    let ids = collect_concurrently(threads, |t| {
        (0..per_thread)
            .map(|i| {
                s.add_comment("doc-1", &format!("r{t}"), &format!("c{i}"))
                    .unwrap()
            })
            .collect::<Vec<String>>()
    });

    let all: BTreeSet<usize> = ids
        .iter()
        .flatten()
        .map(|id| id.parse::<usize>().unwrap())
        .collect();
    assert_eq!(all.len(), threads * per_thread);
    assert_eq!(all, (1..=threads * per_thread).collect::<BTreeSet<_>>());
    let list = s.comments("doc-1").unwrap();
    assert_eq!(list.len(), threads * per_thread);
    assert_eq!(list.iter().collect::<BTreeSet<_>>().len(), list.len());
}

#[test]
fn should_list_concurrent_comments_in_id_order_and_keep_each_threads_own_order() {
    let s = sut!(service(&["doc-1"]));
    let threads = 8;
    let per_thread = 200;

    let ids = collect_concurrently(threads, |t| {
        (0..per_thread)
            .map(|i| {
                s.add_comment("doc-1", &format!("r{t}"), &format!("c{i}"))
                    .unwrap()
            })
            .collect::<Vec<String>>()
    });

    let mut by_id = vec![String::new(); threads * per_thread + 1];
    for t in 0..threads {
        for i in 0..per_thread {
            by_id[ids[t][i].parse::<usize>().unwrap()] = format!("r{t}: c{i}");
        }
    }
    assert_eq!(s.comments("doc-1").unwrap(), by_id[1..].to_vec());
}

#[test]
fn should_keep_documents_independent_under_concurrent_writes() {
    let s = sut!(service(&["a", "b", "c", "d"]));
    let docs = ["a", "b", "c", "d"];

    run_concurrently(8, |t| {
        for i in 0..100 {
            s.add_comment(docs[t % 4], &format!("r{t}"), &format!("c{i}"))
                .unwrap();
        }
    });

    for d in docs {
        assert_eq!(s.comments(d).unwrap().len(), 200);
    }
    assert_eq!(s.add_comment("a", "x", "y"), Ok("801".to_string()));
}

#[test]
fn should_allow_readers_while_writers_add() {
    let s = sut!(service(&["doc-1"]));

    run_concurrently(8, |t| {
        if t < 4 {
            for i in 0..250 {
                s.add_comment("doc-1", &format!("w{t}"), &format!("c{i}"))
                    .unwrap();
            }
        } else {
            for _ in 0..250 {
                let seen = s.comments("doc-1").unwrap();
                assert_eq!(seen.iter().collect::<BTreeSet<_>>().len(), seen.len());
            }
        }
    });

    assert_eq!(s.comments("doc-1").unwrap().len(), 1_000);
}
