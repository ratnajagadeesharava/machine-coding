#![allow(unused_imports, clippy::needless_range_loop)]
use p116_inverted_index_search::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const EPS: f64 = 1e-9;

fn idf(n: usize, df: usize) -> f64 {
    (1.0 + n as f64 / df as f64).ln()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < EPS
}

fn ids(v: Result<Vec<String>, SearchError>) -> Vec<String> {
    v.unwrap()
}

/// d1 "the quick brown fox", d2 "the lazy dog", d3 "quick quick fox".
fn load(e: &Arc<dyn SearchEngine>) {
    e.add_document("d1", "the quick brown fox").unwrap();
    e.add_document("d2", "the lazy dog").unwrap();
    e.add_document("d3", "quick quick fox").unwrap();
}

fn sorted(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v
}

#[test]
fn should_find_documents_containing_a_single_term() {
    let e = sut!(create());
    load(&e);

    assert_eq!(ids(e.search("lazy", "OR", 10)), vec!["d2"]);
    assert_eq!(ids(e.search("lazy", "AND", 10)), vec!["d2"]);
    assert_eq!(sorted(ids(e.search("fox", "OR", 10))), vec!["d1", "d3"]);
    assert_eq!(e.document_count(), 3);
}

#[test]
fn should_tokenise_case_insensitively_on_non_alphanumerics() {
    let e = sut!(create());
    e.add_document("a", "Hello, WORLD!  Foo_bar-baz;qux2024...")
        .unwrap();

    assert_eq!(ids(e.search("hello", "AND", 5)), vec!["a"]);
    assert_eq!(ids(e.search("WoRlD", "AND", 5)), vec!["a"]);
    assert_eq!(ids(e.search("foo bar baz", "AND", 5)), vec!["a"]);
    assert_eq!(ids(e.search("qux2024", "AND", 5)), vec!["a"]);
    assert!(ids(e.search("qux", "AND", 5)).is_empty());
    assert_eq!(ids(e.search("foo_bar", "AND", 5)), vec!["a"]);
    assert!(ids(e.search("helloworld", "OR", 5)).is_empty());
}

#[test]
fn should_score_with_the_documented_tf_idf_formula() {
    let e = sut!(create());
    load(&e);

    assert!(close(e.score_of("fox", "d1").unwrap(), 0.25 * idf(3, 2)));
    assert!(close(
        e.score_of("fox", "d3").unwrap(),
        1.0 / 3.0 * idf(3, 2)
    ));
    assert!(close(
        e.score_of("quick", "d3").unwrap(),
        2.0 / 3.0 * idf(3, 2)
    ));
    assert!(close(
        e.score_of("lazy", "d2").unwrap(),
        1.0 / 3.0 * idf(3, 1)
    ));
    assert!(close(e.score_of("quick fox", "d3").unwrap(), idf(3, 2)));
    assert!(close(
        e.score_of("quick fox", "d1").unwrap(),
        0.5 * idf(3, 2)
    ));
}

#[test]
fn should_rank_by_score_descending() {
    let e = sut!(create());
    load(&e);

    assert_eq!(ids(e.search("quick fox", "OR", 10)), vec!["d3", "d1"]);
    assert_eq!(ids(e.search("the", "OR", 10)), vec!["d2", "d1"]);
    assert_eq!(ids(e.search("lazy fox", "OR", 10)), vec!["d2", "d3", "d1"]);
}

#[test]
fn should_normalise_term_frequency_by_document_length() {
    let e = sut!(create());
    e.add_document("short", "apple pie").unwrap();
    e.add_document("long", "apple pie with a lot of other words around it")
        .unwrap();
    e.add_document("other", "banana").unwrap();

    assert_eq!(ids(e.search("apple", "OR", 10)), vec!["short", "long"]);
    assert!(close(
        e.score_of("apple", "short").unwrap(),
        0.5 * idf(3, 2)
    ));
    assert!(close(e.score_of("apple", "long").unwrap(), 0.1 * idf(3, 2)));
}

#[test]
fn should_require_every_term_in_and_mode() {
    let e = sut!(create());
    load(&e);

    assert_eq!(ids(e.search("quick fox", "AND", 10)), vec!["d3", "d1"]);
    assert_eq!(ids(e.search("quick brown", "AND", 10)), vec!["d1"]);
    assert!(ids(e.search("lazy fox", "AND", 10)).is_empty());
}

#[test]
fn should_match_any_term_in_or_mode() {
    let e = sut!(create());
    load(&e);

    assert_eq!(
        sorted(ids(e.search("brown dog", "OR", 10))),
        vec!["d1", "d2"]
    );
}

#[test]
fn should_return_nothing_for_an_and_query_with_an_unknown_term() {
    let e = sut!(create());
    load(&e);

    assert!(ids(e.search("fox zebra", "AND", 10)).is_empty());
    assert!(ids(e.search("zebra", "AND", 10)).is_empty());
    assert!(ids(e.search("zebra", "OR", 10)).is_empty());
}

#[test]
fn should_let_unknown_terms_contribute_nothing_in_or_mode() {
    let e = sut!(create());
    load(&e);

    assert_eq!(
        ids(e.search("fox zebra", "OR", 10)),
        ids(e.search("fox", "OR", 10))
    );
    assert!(close(
        e.score_of("fox zebra", "d1").unwrap(),
        e.score_of("fox", "d1").unwrap()
    ));
}

#[test]
fn should_break_score_ties_by_doc_id_ascending() {
    let e = sut!(create());
    e.add_document("c", "red blue").unwrap();
    e.add_document("a", "red blue").unwrap();
    e.add_document("b", "blue red").unwrap();
    e.add_document("z", "green").unwrap();
    e.add_document("m", "red blue green green green").unwrap();

    assert_eq!(ids(e.search("red", "OR", 10)), vec!["a", "b", "c", "m"]);
    assert_eq!(
        ids(e.search("red blue", "AND", 10)),
        vec!["a", "b", "c", "m"]
    );
    assert_eq!(ids(e.search("red", "OR", 2)), vec!["a", "b"]);
}

#[test]
fn should_return_at_most_k_results() {
    let e = sut!(create());
    for i in 0..10 {
        e.add_document(&format!("doc{i}"), &format!("common word{i}"))
            .unwrap();
    }

    assert_eq!(
        ids(e.search("common", "OR", 3)),
        vec!["doc0", "doc1", "doc2"]
    );
    assert_eq!(ids(e.search("common", "OR", 100)).len(), 10);
    assert!(ids(e.search("common", "OR", 0)).is_empty());
}

#[test]
fn should_paginate_the_ranking_with_offset_and_limit() {
    let e = sut!(create());
    for i in 0..10 {
        e.add_document(
            &format!("doc{i:02}"),
            &format!("common {}", "pad ".repeat(i)),
        )
        .unwrap();
    }
    let full = ids(e.search("common", "OR", 100));
    assert_eq!(full.len(), 10);
    let mut expected = full.clone();
    expected.sort();
    assert_eq!(full, expected);

    let mut glued = vec![];
    let mut offset = 0;
    while offset < 10 {
        glued.extend(ids(e.search_page("common", "OR", offset, 4)));
        offset += 4;
    }
    assert_eq!(glued, full);
    assert_eq!(
        ids(e.search_page("common", "OR", 4, 3)),
        full[4..7].to_vec()
    );
    assert_eq!(
        ids(e.search_page("common", "OR", 8, 5)),
        full[8..10].to_vec()
    );
    assert!(ids(e.search_page("common", "OR", 10, 5)).is_empty());
    assert!(ids(e.search_page("common", "OR", 99, 5)).is_empty());
    assert!(ids(e.search_page("common", "OR", 2, 0)).is_empty());
}

#[test]
fn should_reject_adding_an_existing_document_id() {
    let e = sut!(create());
    load(&e);

    assert_eq!(
        e.add_document("d1", "something else"),
        Err(SearchError::DuplicateDocument)
    );
    assert_eq!(e.document_count(), 3);
    assert!(ids(e.search("something", "OR", 5)).is_empty());
    assert_eq!(ids(e.search("brown", "OR", 5)), vec!["d1"]);
}

#[test]
fn should_reject_invalid_arguments() {
    let e = sut!(create());
    load(&e);

    assert_eq!(e.add_document(" ", "x"), Err(SearchError::InvalidArgument));
    assert_eq!(e.add_document("", "x"), Err(SearchError::InvalidArgument));
    assert_eq!(e.search("fox", "XOR", 5), Err(SearchError::InvalidArgument));
    assert_eq!(e.search("fox", "and", 5), Err(SearchError::InvalidArgument));
    assert_eq!(e.search("fox", "", 5), Err(SearchError::InvalidArgument));
    assert_eq!(
        e.search_page("fox", "or", 0, 5),
        Err(SearchError::InvalidArgument)
    );
    assert_eq!(e.document_count(), 3);
}

#[test]
fn should_return_nothing_for_a_query_without_tokens() {
    let e = sut!(create());
    load(&e);

    assert!(ids(e.search("", "OR", 10)).is_empty());
    assert!(ids(e.search("  ,.! ", "AND", 10)).is_empty());
    assert!(ids(e.search("", "AND", 10)).is_empty());
    assert_eq!(e.score_of("", "d1"), Ok(0.0));
}

#[test]
fn should_treat_repeated_query_terms_as_a_single_term() {
    let e = sut!(create());
    load(&e);

    assert!(close(
        e.score_of("fox fox FOX", "d1").unwrap(),
        e.score_of("fox", "d1").unwrap()
    ));
    assert_eq!(
        ids(e.search("fox fox", "AND", 10)),
        ids(e.search("fox", "AND", 10))
    );
    assert_eq!(
        ids(e.search("quick quick fox", "OR", 10)),
        ids(e.search("quick fox", "OR", 10))
    );
}

#[test]
fn should_stop_returning_a_deleted_document() {
    let e = sut!(create());
    load(&e);

    e.delete_document("d2").unwrap();

    assert!(ids(e.search("lazy", "OR", 10)).is_empty());
    assert_eq!(ids(e.search("the", "OR", 10)), vec!["d1"]);
    assert_eq!(e.document_count(), 2);
    assert_eq!(e.score_of("lazy", "d2"), Err(SearchError::UnknownDocument));
}

#[test]
fn should_recompute_idf_with_the_new_document_count_after_a_delete() {
    let e = sut!(create());
    load(&e);
    assert!(close(e.score_of("fox", "d1").unwrap(), 0.25 * idf(3, 2)));

    e.delete_document("d2").unwrap(); // N = 2, df(fox) = 2
    assert!(close(e.score_of("fox", "d1").unwrap(), 0.25 * idf(2, 2)));

    e.delete_document("d3").unwrap(); // N = 1, df(fox) = 1
    assert!(close(e.score_of("fox", "d1").unwrap(), 0.25 * idf(1, 1)));
}

#[test]
fn should_reflect_an_updated_text_immediately() {
    let e = sut!(create());
    load(&e);

    e.update_document("d2", "quick quick quick dog").unwrap();

    assert!(ids(e.search("lazy", "OR", 10)).is_empty());
    assert_eq!(ids(e.search("quick", "OR", 10)), vec!["d2", "d3", "d1"]);
    assert_eq!(ids(e.search("the", "OR", 10)), vec!["d1"]);
    assert_eq!(e.document_count(), 3);
    assert!(close(e.score_of("quick", "d2").unwrap(), 0.75 * idf(3, 3)));
    assert!(close(e.score_of("quick", "d1").unwrap(), 0.25 * idf(3, 3)));
}

#[test]
fn should_reject_deleting_or_updating_an_unknown_document() {
    let e = sut!(create());
    load(&e);

    assert_eq!(e.delete_document("nope"), Err(SearchError::UnknownDocument));
    assert_eq!(
        e.update_document("nope", "x"),
        Err(SearchError::UnknownDocument)
    );
    e.delete_document("d1").unwrap();
    assert_eq!(e.delete_document("d1"), Err(SearchError::UnknownDocument));
    assert_eq!(
        e.update_document("d1", "back"),
        Err(SearchError::UnknownDocument)
    );
    assert_eq!(e.document_count(), 2);
}

#[test]
fn should_allow_adding_a_deleted_id_again_with_fresh_content() {
    let e = sut!(create());
    load(&e);

    e.delete_document("d1").unwrap();
    e.add_document("d1", "completely different").unwrap();

    assert!(ids(e.search("brown", "OR", 10)).is_empty());
    assert_eq!(ids(e.search("different", "OR", 10)), vec!["d1"]);
}

#[test]
fn should_count_documents_without_tokens_towards_the_collection_size() {
    let e = sut!(create());
    e.add_document("empty", "").unwrap();
    e.add_document("punct", "?! ... --").unwrap();
    e.add_document("a", "alpha beta").unwrap();

    assert_eq!(e.document_count(), 3);
    assert!(close(e.score_of("alpha", "a").unwrap(), 0.5 * idf(3, 1)));
    assert_eq!(e.score_of("alpha", "empty"), Ok(0.0));
    assert_eq!(ids(e.search("alpha", "OR", 10)), vec!["a"]);
    e.update_document("empty", "alpha").unwrap();
    assert_eq!(ids(e.search("alpha", "OR", 10)), vec!["empty", "a"]);
}

#[test]
fn should_score_an_existing_document_without_query_terms_as_zero() {
    let e = sut!(create());
    load(&e);

    assert_eq!(e.score_of("lazy", "d1"), Ok(0.0));
    assert_eq!(
        e.score_of("lazy", "ghost"),
        Err(SearchError::UnknownDocument)
    );
}

#[test]
fn should_not_lose_documents_added_concurrently() {
    let e = sut!(create());

    run_concurrently(16, |t| {
        for i in 0..200 {
            e.add_document(&format!("t{t}-{i}"), &format!("shared token{t} n{i}"))
                .unwrap();
        }
    });

    assert_eq!(e.document_count(), 16 * 200);
    assert_eq!(ids(e.search("shared", "OR", 10_000)).len(), 3_200);
    assert_eq!(ids(e.search("token3 n7", "AND", 10)), vec!["t3-7"]);
}

#[test]
fn should_accept_exactly_one_of_many_concurrent_adds_with_the_same_id() {
    let e = sut!(create());
    let accepted = AtomicUsize::new(0);

    run_concurrently(12, |t| {
        match e.add_document("same", &format!("payload from {t}")) {
            Ok(()) => {
                accepted.fetch_add(1, Ordering::SeqCst);
            }
            Err(err) => assert_eq!(err, SearchError::DuplicateDocument),
        }
    });

    assert_eq!(accepted.load(Ordering::SeqCst), 1);
    assert_eq!(e.document_count(), 1);
    assert_eq!(ids(e.search("payload", "OR", 5)), vec!["same"]);
}

#[test]
fn should_answer_searches_while_documents_are_added_and_deleted() {
    let e = sut!(create());
    for i in 0..100 {
        e.add_document(&format!("base{i}"), &format!("stable content {i}"))
            .unwrap();
    }

    run_concurrently(8, |t| {
        for i in 0..300 {
            if t < 4 {
                let id = format!("tmp{t}-{i}");
                e.add_document(&id, "volatile content").unwrap();
                e.delete_document(&id).unwrap();
            } else {
                assert_eq!(ids(e.search("stable content", "AND", 1_000)).len(), 100);
            }
        }
    });

    assert_eq!(e.document_count(), 100);
    assert!(ids(e.search("volatile", "OR", 10)).is_empty());
}
