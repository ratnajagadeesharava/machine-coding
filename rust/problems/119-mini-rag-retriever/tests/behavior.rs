#![allow(unused_imports, clippy::needless_range_loop)]
mod common;

use common::*;
use p119_mini_rag_retriever::{contract::*, create, create_with_store};
use playground_support::{run_concurrently, sut};
use std::collections::BTreeSet;
use std::sync::Arc;

fn embedder() -> Arc<VocabEmbedder> {
    Arc::new(VocabEmbedder::default())
}

fn prompt(context: &str, query: &str) -> String {
    format!("Answer using only the context.\n\nContext:\n{context}\n\nQuestion: {query}\nAnswer:")
}

/// a="alpha beta", b="alpha gamma delta", c="alpha": for the query "alpha" the ranking is c, a, b.
fn ingest_prompt_corpus(e: &Arc<dyn RagEngine>) {
    e.ingest("a", "alpha beta").unwrap();
    e.ingest("b", "alpha gamma delta").unwrap();
    e.ingest("c", "alpha").unwrap();
}

fn ids(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn should_cut_text_into_chunks_of_at_most_chunk_size_words() {
    let e = sut!(create(embedder()));
    e.set_chunking(4, 0).unwrap();

    assert_eq!(e.ingest("doc", &words("w", 10)), Ok(3));

    assert_eq!(e.chunk_text("doc#0").as_deref(), Some("w0 w1 w2 w3"));
    assert_eq!(e.chunk_text("doc#1").as_deref(), Some("w4 w5 w6 w7"));
    assert_eq!(e.chunk_text("doc#2").as_deref(), Some("w8 w9"));
    assert_eq!(e.chunk_text("doc#3"), None);
    assert_eq!(e.chunk_count(), 3);
}

#[test]
fn should_repeat_the_overlap_words_at_the_start_of_the_next_chunk() {
    let e = sut!(create(embedder()));
    e.set_chunking(4, 1).unwrap();

    assert_eq!(e.ingest("doc", &words("w", 10)), Ok(3));

    assert_eq!(e.chunk_text("doc#0").as_deref(), Some("w0 w1 w2 w3"));
    assert_eq!(e.chunk_text("doc#1").as_deref(), Some("w3 w4 w5 w6"));
    assert_eq!(e.chunk_text("doc#2").as_deref(), Some("w6 w7 w8 w9"));
}

#[test]
fn should_not_create_a_trailing_chunk_that_is_already_covered() {
    let e = sut!(create(embedder()));
    e.set_chunking(4, 1).unwrap();

    assert_eq!(e.ingest("seven", &words("w", 7)), Ok(2));
    assert_eq!(e.ingest("four", &words("w", 4)), Ok(1));
    assert_eq!(e.ingest("one", "w0"), Ok(1));
    assert_eq!(e.ingest("five", &words("w", 5)), Ok(2));
    assert_eq!(e.chunk_text("five#1").as_deref(), Some("w3 w4"));
}

#[test]
fn should_create_no_chunks_for_empty_or_blank_text() {
    let e = sut!(create(embedder()));

    assert_eq!(e.ingest("doc", ""), Ok(0));
    assert_eq!(e.ingest("doc", "  \n\t "), Ok(0));
    assert_eq!(e.chunk_count(), 0);
    assert_eq!(e.retrieve("alpha", 5), Ok(vec![]));
}

#[test]
fn should_join_chunk_words_with_single_spaces() {
    let e = sut!(create(embedder()));
    e.set_chunking(3, 0).unwrap();

    e.ingest("doc", "  alpha\n beta\t\tgamma   delta  ")
        .unwrap();

    assert_eq!(e.chunk_text("doc#0").as_deref(), Some("alpha beta gamma"));
    assert_eq!(e.chunk_text("doc#1").as_deref(), Some("delta"));
}

#[test]
fn should_replace_all_chunks_when_a_document_is_ingested_again() {
    let e = sut!(create(embedder()));
    e.set_chunking(2, 0).unwrap();
    e.ingest("doc", "alpha beta gamma delta epsilon zeta")
        .unwrap();
    e.ingest("other", "eta theta").unwrap();
    assert_eq!(e.chunk_count(), 4);

    assert_eq!(e.ingest("doc", "iota kappa"), Ok(1));

    assert_eq!(e.chunk_count(), 2);
    assert_eq!(e.chunk_text("doc#0").as_deref(), Some("iota kappa"));
    assert_eq!(e.chunk_text("doc#1"), None);
    assert_eq!(e.chunk_text("doc#2"), None);
    assert_eq!(e.retrieve("alpha", 10).unwrap(), ids(&["doc#0", "other#0"]));
    assert_eq!(e.ingest("doc", ""), Ok(0));
    assert_eq!(e.retrieve("alpha", 10).unwrap(), ids(&["other#0"]));
}

#[test]
fn should_use_the_configured_chunking_only_for_later_ingests() {
    let e = sut!(create(embedder()));
    e.set_chunking(2, 0).unwrap();
    e.ingest("old", "alpha beta gamma delta").unwrap();

    e.set_chunking(4, 0).unwrap();
    e.ingest("new", "alpha beta gamma delta").unwrap();

    assert_eq!(e.chunk_text("old#1").as_deref(), Some("gamma delta"));
    assert_eq!(
        e.chunk_text("new#0").as_deref(),
        Some("alpha beta gamma delta")
    );
    assert_eq!(e.chunk_count(), 3);
}

#[test]
fn should_default_to_chunks_of_a_hundred_words_with_an_overlap_of_twenty() {
    let e = sut!(create(embedder()));

    assert_eq!(e.ingest("doc", &words("w", 250)), Ok(3));
    let c1 = e.chunk_text("doc#1").unwrap();
    assert!(c1.starts_with("w80 w81") && c1.ends_with("w178 w179"));
    let c2 = e.chunk_text("doc#2").unwrap();
    assert!(c2.starts_with("w160 w161") && c2.ends_with("w248 w249"));
}

#[test]
fn should_reject_invalid_chunking_and_keep_the_old_setting() {
    let e = sut!(create(embedder()));
    e.set_chunking(3, 0).unwrap();

    assert_eq!(e.set_chunking(0, 0), Err(RagError::InvalidArgument));
    assert_eq!(e.set_chunking(4, 4), Err(RagError::InvalidArgument));
    assert_eq!(e.set_chunking(4, 7), Err(RagError::InvalidArgument));

    assert_eq!(e.ingest("doc", "w0 w1 w2 w3 w4 w5"), Ok(2));
    e.set_chunking(1, 0).unwrap();
    assert_eq!(e.ingest("single", "w0 w1 w2"), Ok(3));
}

#[test]
fn should_reject_blank_document_ids() {
    let e = sut!(create(embedder()));

    assert_eq!(e.ingest("", "alpha"), Err(RagError::InvalidArgument));
    assert_eq!(e.ingest("  ", "alpha"), Err(RagError::InvalidArgument));
    assert_eq!(e.delete_document(" "), Err(RagError::InvalidArgument));
    assert_eq!(e.chunk_count(), 0);
}

#[test]
fn should_rank_chunks_by_cosine_similarity() {
    let e = sut!(create(embedder()));
    e.ingest("d1", "alpha alpha beta").unwrap();
    e.ingest("d2", "alpha gamma gamma").unwrap();
    e.ingest("d3", "delta delta delta").unwrap();

    assert_eq!(
        e.retrieve("alpha", 3).unwrap(),
        ids(&["d1#0", "d2#0", "d3#0"])
    );
    assert_eq!(
        e.retrieve("gamma beta", 3).unwrap(),
        ids(&["d2#0", "d1#0", "d3#0"])
    );
    assert_eq!(e.retrieve("delta", 3).unwrap()[0], "d3#0");
}

#[test]
fn should_ignore_the_length_of_a_chunk_when_ranking() {
    let e = sut!(create(embedder()));
    e.ingest("short", "alpha").unwrap();
    e.ingest("long", "alpha alpha alpha alpha").unwrap();
    e.ingest("mixed", "alpha beta").unwrap();

    assert_eq!(
        e.retrieve("alpha alpha", 3).unwrap(),
        ids(&["long#0", "short#0", "mixed#0"])
    );
}

#[test]
fn should_break_ties_by_chunk_id_in_ascending_string_order() {
    let e = sut!(create(embedder()));
    e.ingest("b", "alpha").unwrap();
    e.ingest("a", "alpha").unwrap();
    e.ingest("c", "alpha").unwrap();

    assert_eq!(e.retrieve("alpha", 3).unwrap(), ids(&["a#0", "b#0", "c#0"]));
}

#[test]
fn should_order_equal_scores_of_numbered_chunks_as_strings() {
    let e = sut!(create(embedder()));
    e.set_chunking(1, 0).unwrap();
    e.ingest("n", &["alpha"; 12].join(" ")).unwrap();

    assert_eq!(
        e.retrieve("alpha", 12).unwrap(),
        ids(&[
            "n#0", "n#1", "n#10", "n#11", "n#2", "n#3", "n#4", "n#5", "n#6", "n#7", "n#8", "n#9"
        ])
    );
}

#[test]
fn should_return_at_most_k_results_and_all_chunks_when_there_are_fewer() {
    let e = sut!(create(embedder()));
    e.ingest("d1", "alpha").unwrap();
    e.ingest("d2", "beta").unwrap();
    e.ingest("d3", "gamma").unwrap();

    assert_eq!(e.retrieve("alpha", 1).unwrap(), ids(&["d1#0"]));
    assert_eq!(e.retrieve("alpha", 2).unwrap().len(), 2);
    assert_eq!(e.retrieve("alpha", 3).unwrap().len(), 3);
    assert_eq!(e.retrieve("alpha", 1_000_000).unwrap().len(), 3);
}

#[test]
fn should_still_return_chunks_with_zero_similarity_after_the_related_ones() {
    let e = sut!(create(embedder()));
    e.ingest("x", "gamma").unwrap();
    e.ingest("y", "alpha").unwrap();
    e.ingest("z", "unknownword").unwrap();

    assert_eq!(e.retrieve("alpha", 3).unwrap(), ids(&["y#0", "x#0", "z#0"]));
}

#[test]
fn should_return_nothing_when_nothing_is_ingested() {
    let e = sut!(create(embedder()));

    assert_eq!(e.retrieve("alpha", 3), Ok(vec![]));
    assert_eq!(
        e.build_prompt("alpha", 3, 1_000),
        Ok(prompt("(none)", "alpha"))
    );
}

#[test]
fn should_reject_bad_retrieve_arguments() {
    let e = sut!(create(embedder()));
    e.ingest("d", "alpha").unwrap();

    assert_eq!(e.retrieve("alpha", 0), Err(RagError::InvalidArgument));
    assert_eq!(e.retrieve("  ", 1), Err(RagError::InvalidArgument));
    assert_eq!(e.retrieve("", 1), Err(RagError::InvalidArgument));
    assert_eq!(
        e.build_prompt("alpha", 0, 100),
        Err(RagError::InvalidArgument)
    );
    assert_eq!(e.build_prompt(" ", 1, 100), Err(RagError::InvalidArgument));
}

#[test]
fn should_build_the_prompt_from_the_exact_template() {
    let e = sut!(create(embedder()));
    ingest_prompt_corpus(&e);

    assert_eq!(
        e.build_prompt("alpha", 3, 1_000).unwrap(),
        "Answer using only the context.\n\nContext:\n[1] alpha\n[2] alpha beta\n[3] alpha gamma delta\n\nQuestion: alpha\nAnswer:"
    );
    assert_eq!(
        e.build_prompt("alpha", 1, 1_000).unwrap(),
        prompt("[1] alpha", "alpha")
    );
    assert!(e
        .build_prompt("alpha", 1_000, 1_000)
        .unwrap()
        .contains("[3] alpha gamma delta"));
}

#[test]
fn should_include_whole_chunks_only_while_the_context_fits_the_budget() {
    let e = sut!(create(embedder()));
    ingest_prompt_corpus(&e);
    // context lengths: "[1] alpha" = 9, + "\n[2] alpha beta" = 24, + "\n[3] alpha gamma delta" = 46

    assert!(e
        .build_prompt("alpha", 3, 46)
        .unwrap()
        .contains("[3] alpha gamma delta"));
    let two = prompt("[1] alpha\n[2] alpha beta", "alpha");
    assert_eq!(e.build_prompt("alpha", 3, 45).unwrap(), two);
    assert_eq!(e.build_prompt("alpha", 3, 24).unwrap(), two);
    let one = prompt("[1] alpha", "alpha");
    assert_eq!(e.build_prompt("alpha", 3, 23).unwrap(), one);
    assert_eq!(e.build_prompt("alpha", 3, 9).unwrap(), one);
}

#[test]
fn should_render_no_context_when_even_the_first_chunk_does_not_fit() {
    let e = sut!(create(embedder()));
    ingest_prompt_corpus(&e);

    assert_eq!(
        e.build_prompt("alpha", 3, 8).unwrap(),
        prompt("(none)", "alpha")
    );
    assert_eq!(
        e.build_prompt("alpha", 3, 0).unwrap(),
        prompt("(none)", "alpha")
    );
}

#[test]
fn should_stop_at_the_first_chunk_that_does_not_fit_even_if_a_later_one_would() {
    let e = sut!(create(embedder()));
    e.ingest("c", "alpha").unwrap();
    e.ingest("big", "alpha alpha alpha beta").unwrap();
    e.ingest("small", "alpha beta beta").unwrap();
    // ranking for "alpha": c (1.0), big, small; entries are 9, 26 and 19 characters long

    assert_eq!(
        e.retrieve("alpha", 3).unwrap(),
        ids(&["c#0", "big#0", "small#0"])
    );
    assert_eq!(
        e.build_prompt("alpha", 3, 30).unwrap(),
        prompt("[1] alpha", "alpha")
    );
}

#[test]
fn should_count_the_width_of_multi_digit_ranks_in_the_budget() {
    let e = sut!(create(embedder()));
    e.set_chunking(1, 0).unwrap();
    e.ingest("n", &["alpha"; 11].join(" ")).unwrap();
    // 11 chunks "alpha": entries "[1] alpha".."[9] alpha" are 9 chars, "[10] alpha" and "[11] alpha" are 10
    // 9 entries = 9*9 + 8 newlines = 89; the 10th adds 1 + 10 = 11 -> 100; the 11th -> 111

    let p100 = e.build_prompt("alpha", 11, 100).unwrap();
    assert!(p100.contains("[10] alpha") && !p100.contains("[11]"));
    let p99 = e.build_prompt("alpha", 11, 99).unwrap();
    assert!(p99.contains("[9] alpha") && !p99.contains("[10]"));
    assert!(e
        .build_prompt("alpha", 11, 111)
        .unwrap()
        .contains("[11] alpha"));
}

#[test]
fn should_put_the_query_verbatim_into_the_prompt() {
    let e = sut!(create(embedder()));
    e.ingest("d", "alpha").unwrap();

    assert_eq!(
        e.build_prompt("  what is alpha?\nexplain ", 1, 100)
            .unwrap(),
        prompt("[1] alpha", "  what is alpha?\nexplain ")
    );
}

#[test]
fn should_delete_only_the_chunks_of_that_document() {
    let e = sut!(create(embedder()));
    e.set_chunking(2, 0).unwrap();
    e.ingest("a", "alpha beta gamma delta").unwrap();
    e.ingest("b", "alpha beta").unwrap();

    assert_eq!(e.delete_document("a"), Ok(true));

    assert_eq!(e.chunk_count(), 1);
    assert_eq!(e.chunk_text("a#0"), None);
    assert_eq!(e.retrieve("alpha", 10).unwrap(), ids(&["b#0"]));
    assert_eq!(e.delete_document("a"), Ok(false));
    assert_eq!(e.delete_document("never-ingested"), Ok(false));
}

#[test]
fn should_allow_an_ingest_after_a_delete() {
    let e = sut!(create(embedder()));
    e.ingest("a", "alpha").unwrap();
    e.delete_document("a").unwrap();

    assert_eq!(e.ingest("a", "beta"), Ok(1));

    assert_eq!(e.chunk_text("a#0").as_deref(), Some("beta"));
    assert_eq!(e.retrieve("beta", 5).unwrap(), ids(&["a#0"]));
}

#[test]
fn should_work_with_another_embedder() {
    let e = sut!(create(Arc::new(LetterEmbedder)));
    e.ingest("a", "aaa").unwrap();
    e.ingest("b", "bbb").unwrap();
    e.ingest("c", "abab").unwrap();

    assert_eq!(e.retrieve("bbb", 3).unwrap(), ids(&["b#0", "c#0", "a#0"]));
    assert_eq!(e.retrieve("aaa", 3).unwrap(), ids(&["a#0", "c#0", "b#0"]));
}

#[test]
fn should_call_the_embedder_once_per_chunk_and_once_per_query() {
    let emb = embedder();
    let e = sut!(create(emb.clone()));
    let calls = || emb.calls.load(std::sync::atomic::Ordering::SeqCst);
    e.set_chunking(2, 0).unwrap();

    e.ingest("doc", "alpha beta gamma delta epsilon").unwrap();
    assert_eq!(calls(), 3);

    e.retrieve("alpha", 2).unwrap();
    assert_eq!(calls(), 4);
    e.build_prompt("alpha", 2, 100).unwrap();
    assert_eq!(calls(), 5);

    e.chunk_count();
    e.chunk_text("doc#0");
    e.delete_document("doc").unwrap();
    assert_eq!(calls(), 5);
}

#[test]
fn should_store_vectors_in_the_supplied_vector_store() {
    let store = Arc::new(BruteForceStore::default());
    let e = sut!(create_with_store(embedder(), store.clone()));
    e.set_chunking(2, 0).unwrap();

    e.ingest("a", "alpha beta gamma delta").unwrap();
    assert_eq!(
        store.ids().into_iter().collect::<Vec<_>>(),
        ids(&["a#0", "a#1"])
    );
    assert_eq!(e.retrieve("gamma", 2).unwrap(), ids(&["a#1", "a#0"]));
    assert!(store.searches.load(std::sync::atomic::Ordering::SeqCst) > 0);

    e.ingest("a", "alpha beta").unwrap();
    assert_eq!(store.ids().into_iter().collect::<Vec<_>>(), ids(&["a#0"]));

    e.ingest("b", "gamma").unwrap();
    e.delete_document("a").unwrap();
    assert_eq!(store.ids().into_iter().collect::<Vec<_>>(), ids(&["b#0"]));
    e.delete_document("b").unwrap();
    assert!(store.ids().is_empty());
}

#[test]
fn should_give_the_same_answers_with_the_default_store_and_a_supplied_store() {
    let plain = sut!(create(embedder()));
    let custom = sut!(create_with_store(
        embedder(),
        Arc::new(BruteForceStore::default())
    ));
    let docs = [
        ("d1", "alpha beta gamma alpha"),
        ("d2", "delta epsilon alpha"),
        ("d3", "zeta eta theta iota"),
        ("d4", "alpha"),
        ("d5", "gamma gamma gamma beta"),
        ("d6", "nothing known here"),
    ];
    for e in [&plain, &custom] {
        e.set_chunking(3, 1).unwrap();
        for (id, text) in docs {
            e.ingest(id, text).unwrap();
        }
    }

    for q in [
        "alpha",
        "gamma beta",
        "zeta iota",
        "mu nu",
        "alpha delta epsilon",
    ] {
        assert_eq!(custom.retrieve(q, 4), plain.retrieve(q, 4));
        assert_eq!(custom.build_prompt(q, 4, 80), plain.build_prompt(q, 4, 80));
    }
}

#[test]
fn should_report_embedder_failures_and_keep_the_previous_version_of_the_document() {
    let e = sut!(create(Arc::new(FailingEmbedder("boom"))));
    e.set_chunking(2, 0).unwrap();
    e.ingest("doc", "alpha beta gamma delta").unwrap();

    assert!(matches!(
        e.ingest("doc", "iota kappa lambda boom"),
        Err(RagError::Embedding(_))
    ));

    assert_eq!(e.chunk_count(), 2);
    assert_eq!(e.chunk_text("doc#0").as_deref(), Some("alpha beta"));
    assert_eq!(e.chunk_text("doc#1").as_deref(), Some("gamma delta"));
}

#[test]
fn should_handle_concurrent_ingests_and_retrievals() {
    let e = sut!(create(embedder()));
    e.set_chunking(2, 0).unwrap();

    run_concurrently(8, |t| {
        for d in 0..20 {
            e.ingest(&format!("t{t}-d{d}"), "alpha beta gamma delta epsilon zeta")
                .unwrap();
            let hits = e.retrieve("alpha", 5).unwrap();
            assert!((1..=5).contains(&hits.len()));
            e.build_prompt("beta", 3, 200).unwrap();
        }
    });

    assert_eq!(e.chunk_count(), 8 * 20 * 3);
    let all = e.retrieve("alpha", 1_000).unwrap();
    assert_eq!(all.len(), 480);
    assert_eq!(all.iter().collect::<BTreeSet<_>>().len(), 480);
}

#[test]
fn should_not_lose_documents_when_some_are_deleted_concurrently() {
    let e = sut!(create(embedder()));
    e.set_chunking(2, 0).unwrap();

    run_concurrently(8, |t| {
        for d in 0..25 {
            let id = format!("t{t}-{d}");
            e.ingest(&id, "alpha beta gamma").unwrap();
            if d % 2 == 0 {
                e.delete_document(&id).unwrap();
            }
        }
    });

    let all = e.retrieve("alpha", 10_000).unwrap();
    assert_eq!(all.len(), 8 * 12 * 2);
    assert_eq!(all.iter().collect::<BTreeSet<_>>().len(), all.len());
    assert_eq!(e.chunk_count(), all.len());
}
