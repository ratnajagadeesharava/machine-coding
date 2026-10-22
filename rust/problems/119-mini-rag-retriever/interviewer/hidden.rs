#![allow(unused_imports, clippy::needless_range_loop)]
#[path = "../tests/common/mod.rs"]
mod common;

use common::*;
use p119_mini_rag_retriever::{contract::*, create, create_with_store};
use playground_support::{run_concurrently, sut};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::time::Duration;

fn ids(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

/// Blocks inside `embed` for any text containing "SLOW" until released.
struct GatedEmbedder {
    entered: Mutex<bool>,
    entered_cv: Condvar,
    released: Mutex<bool>,
    released_cv: Condvar,
}

impl GatedEmbedder {
    fn new() -> Self {
        GatedEmbedder {
            entered: Mutex::new(false),
            entered_cv: Condvar::new(),
            released: Mutex::new(false),
            released_cv: Condvar::new(),
        }
    }
    fn wait_entered(&self) {
        let g = self.entered.lock().unwrap();
        let (g, t) = self
            .entered_cv
            .wait_timeout_while(g, Duration::from_secs(10), |e| !*e)
            .unwrap();
        assert!(!t.timed_out() && *g, "embedder was never called");
    }
    fn release(&self) {
        *self.released.lock().unwrap() = true;
        self.released_cv.notify_all();
    }
}

impl Embedder for GatedEmbedder {
    fn embed(&self, text: &str) -> Result<Vec<f64>, EmbedError> {
        if text.contains("SLOW") {
            *self.entered.lock().unwrap() = true;
            self.entered_cv.notify_all();
            let g = self.released.lock().unwrap();
            let (_g, t) = self
                .released_cv
                .wait_timeout_while(g, Duration::from_secs(20), |r| !*r)
                .unwrap();
            if t.timed_out() {
                return Err(EmbedError("never released".into()));
            }
        }
        Ok(VocabEmbedder::vector(text))
    }
}

#[test]
fn should_keep_serving_other_documents_while_one_embedding_is_slow() {
    let gate = Arc::new(GatedEmbedder::new());
    let e = sut!(create(gate.clone()));
    e.ingest("fast0", "alpha beta").unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::scope(|s| {
        let slow = {
            let e = e.clone();
            s.spawn(move || e.ingest("slow", "SLOW alpha gamma"))
        };
        gate.wait_entered();
        {
            let e = e.clone();
            s.spawn(move || {
                e.ingest("fast1", "alpha").unwrap();
                e.delete_document("fast0").unwrap();
                e.build_prompt("alpha", 3, 100).unwrap();
                tx.send(e.retrieve("alpha", 5).unwrap()).ok();
            });
        }
        let seen = rx.recv_timeout(Duration::from_secs(5));
        gate.release();
        let seen = seen.expect("other operations were blocked by a slow embedding of one document");
        assert_eq!(seen, ids(&["fast1#0"]));
        assert_eq!(slow.join().unwrap(), Ok(1));
    });
    assert_eq!(e.retrieve("alpha", 5).unwrap(), ids(&["fast1#0", "slow#0"]));
}

#[test]
fn should_never_show_a_half_replaced_document() {
    let e = sut!(create(Arc::new(VocabEmbedder::default())));
    e.set_chunking(1, 0).unwrap();
    let a = "alpha alpha alpha alpha";
    let b = "beta beta";
    e.ingest("d", a).unwrap();
    let done = AtomicBool::new(false);

    run_concurrently(6, |t| {
        if t < 2 {
            for i in 0..400 {
                e.ingest("d", if (i + t) % 2 == 0 { a } else { b }).unwrap();
            }
            if t == 0 {
                done.store(true, Ordering::SeqCst);
            }
            return;
        }
        while !done.load(Ordering::SeqCst) {
            let n = e.retrieve("mu", 100).unwrap().len();
            assert!(n == 2 || n == 4, "saw {n} chunks");
            let p = e.build_prompt("mu", 100, 10_000).unwrap();
            assert!(
                p.contains("alpha") ^ p.contains("beta"),
                "prompt mixes versions: {p}"
            );
        }
    });
    let n = e.chunk_count();
    assert!(n == 2 || n == 4);
}

#[test]
fn should_not_confuse_documents_whose_ids_share_a_prefix() {
    let e = sut!(create(Arc::new(VocabEmbedder::default())));
    e.set_chunking(1, 0).unwrap();
    e.ingest("a", "alpha beta").unwrap();
    e.ingest("ab", "alpha").unwrap();
    e.ingest("a#1", "alpha").unwrap();

    assert_eq!(
        e.retrieve("alpha", 10).unwrap(),
        ids(&["a#0", "a#1#0", "ab#0", "a#1"])
    );

    assert_eq!(e.delete_document("a"), Ok(true));

    assert_eq!(e.retrieve("alpha", 10).unwrap(), ids(&["a#1#0", "ab#0"]));
    assert_eq!(e.chunk_text("a#1#0").as_deref(), Some("alpha"));
    assert_eq!(e.delete_document("a#1"), Ok(true));
    assert_eq!(e.retrieve("alpha", 10).unwrap(), ids(&["ab#0"]));
}

#[test]
fn should_ingest_from_many_threads_without_losing_anything() {
    let e = sut!(create(Arc::new(VocabEmbedder::default())));
    e.set_chunking(3, 0).unwrap();

    run_concurrently(16, |t| {
        for d in 0..50 {
            e.ingest(
                &format!("t{t}-{d}"),
                "alpha beta gamma delta epsilon zeta eta",
            )
            .unwrap();
        }
    });

    assert_eq!(e.chunk_count(), 16 * 50 * 3);
    let all = e.retrieve("alpha", 100_000).unwrap();
    assert_eq!(all.len(), 2_400);
    let unique: std::collections::BTreeSet<_> = all.iter().collect();
    assert_eq!(unique.len(), 2_400);
    assert_eq!(e.chunk_text("t15-49#2").as_deref(), Some("eta"));
}

#[test]
fn should_rank_a_large_corpus_like_a_brute_force_scan() {
    let e = sut!(create(Arc::new(VocabEmbedder::default())));
    let mut vectors: HashMap<String, Vec<f64>> = HashMap::new();
    let mut seed: u64 = 12345;
    let next = |seed: &mut u64| {
        *seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *seed >> 33
    };
    for d in 0..4_000 {
        let n = 2 + (next(&mut seed) % 7) as usize;
        let text: String = (0..n)
            .map(|_| WORDS[(next(&mut seed) % 16) as usize])
            .collect::<Vec<_>>()
            .join(" ");
        e.ingest(&format!("doc{d}"), &text).unwrap();
        vectors.insert(format!("doc{d}#0"), VocabEmbedder::vector(&text));
    }

    for _ in 0..60 {
        let query = format!(
            "{} {}",
            WORDS[(next(&mut seed) % 16) as usize],
            WORDS[(next(&mut seed) % 16) as usize]
        );
        let qv = VocabEmbedder::vector(&query);
        let mut expected: Vec<f64> = vectors.values().map(|v| cosine(&qv, v)).collect();
        expected.sort_by(|a, b| b.partial_cmp(a).unwrap());

        let got = e.retrieve(&query, 10).unwrap();

        assert_eq!(got.len(), 10);
        for i in 0..10 {
            assert!((cosine(&qv, &vectors[&got[i]]) - expected[i]).abs() < 1e-9);
        }
    }
}

#[test]
fn should_leave_no_trace_when_the_first_ingest_of_a_document_fails_halfway() {
    let store = Arc::new(BruteForceStore::default());
    let e = sut!(create_with_store(
        Arc::new(FailingEmbedder("zeta")),
        store.clone()
    ));
    e.set_chunking(2, 0).unwrap();

    assert!(matches!(
        e.ingest("doc", "alpha beta gamma delta zeta eta"),
        Err(RagError::Embedding(_))
    ));

    assert_eq!(e.chunk_count(), 0);
    assert!(store.ids().is_empty());
    assert_eq!(e.retrieve("alpha", 5), Ok(vec![]));
    assert_eq!(e.ingest("doc", "alpha beta"), Ok(1));
    assert!(matches!(e.retrieve("zeta", 1), Err(RagError::Embedding(_))));
    assert_eq!(e.chunk_count(), 1);
}

#[test]
fn should_order_everything_by_id_when_the_query_matches_nothing() {
    let e = sut!(create(Arc::new(VocabEmbedder::default())));
    e.ingest("m", "alpha").unwrap();
    e.ingest("k", "beta").unwrap();
    e.ingest("z", "unknown words only").unwrap();

    assert_eq!(
        e.retrieve("totally unrelated", 5).unwrap(),
        ids(&["k#0", "m#0", "z#0"])
    );
    assert_eq!(
        e.build_prompt("totally unrelated", 2, 100).unwrap(),
        "Answer using only the context.\n\nContext:\n[1] beta\n[2] alpha\n\nQuestion: totally unrelated\nAnswer:"
    );
}

#[test]
fn should_handle_a_very_long_document() {
    let e = sut!(create(Arc::new(VocabEmbedder::default())));

    let chunks = e.ingest("big", &words("w", 100_000)).unwrap();

    assert_eq!(chunks, 1_250);
    assert_eq!(e.chunk_count(), 1_250);
    let last = e.chunk_text("big#1249").unwrap();
    assert!(last.starts_with("w99920 ") && last.ends_with(" w99999"));
}

#[test]
fn should_give_the_same_state_when_the_same_document_is_ingested_twice() {
    let store = Arc::new(BruteForceStore::default());
    let e = sut!(create_with_store(
        Arc::new(VocabEmbedder::default()),
        store.clone()
    ));
    e.set_chunking(2, 1).unwrap();

    let first = e.ingest("doc", "alpha beta gamma delta epsilon").unwrap();
    let before = e.retrieve("gamma", 10).unwrap();
    let prompt_before = e.build_prompt("gamma", 10, 500).unwrap();
    let second = e.ingest("doc", "alpha beta gamma delta epsilon").unwrap();

    assert_eq!((first, second), (4, 4));
    assert_eq!(e.chunk_count(), 4);
    assert_eq!(
        store.ids().into_iter().collect::<Vec<_>>(),
        ids(&["doc#0", "doc#1", "doc#2", "doc#3"])
    );
    assert_eq!(e.retrieve("gamma", 10).unwrap(), before);
    assert_eq!(e.build_prompt("gamma", 10, 500).unwrap(), prompt_before);
}
