//! Deterministic collaborators implemented by the tests.
#![allow(dead_code)]
use p119_mini_rag_retriever::contract::*;
use std::collections::{BTreeSet, HashMap};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

pub const WORDS: [&str; 16] = [
    "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "iota", "kappa",
    "lambda", "mu", "nu", "xi", "omicron", "pi",
];

/// Counts occurrences of 16 known words (lower-cased, split on whitespace); every other word is ignored.
#[derive(Default)]
pub struct VocabEmbedder {
    pub calls: AtomicUsize,
}

impl VocabEmbedder {
    pub fn vector(text: &str) -> Vec<f64> {
        let mut v = vec![0.0; WORDS.len()];
        for w in text.to_lowercase().split_whitespace() {
            if let Some(i) = WORDS.iter().position(|x| *x == w) {
                v[i] += 1.0;
            }
        }
        v
    }
}

impl Embedder for VocabEmbedder {
    fn embed(&self, text: &str) -> Result<Vec<f64>, EmbedError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(Self::vector(text))
    }
}

/// A completely different embedding: a 26-letter histogram.
pub struct LetterEmbedder;

impl Embedder for LetterEmbedder {
    fn embed(&self, text: &str) -> Result<Vec<f64>, EmbedError> {
        let mut v = vec![0.0; 26];
        for c in text.to_lowercase().chars() {
            if c.is_ascii_lowercase() {
                v[(c as u8 - b'a') as usize] += 1.0;
            }
        }
        Ok(v)
    }
}

/// Fails for any text containing the marker, otherwise behaves like `VocabEmbedder`.
pub struct FailingEmbedder(pub &'static str);

impl Embedder for FailingEmbedder {
    fn embed(&self, text: &str) -> Result<Vec<f64>, EmbedError> {
        if text.contains(self.0) {
            Err(EmbedError("embedding service down".into()))
        } else {
            Ok(VocabEmbedder::vector(text))
        }
    }
}

pub fn cosine(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f64 = a.iter().map(|x| x * x).sum();
    let nb: f64 = b.iter().map(|x| x * x).sum();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na.sqrt() * nb.sqrt())
    }
}

/// Plain scan over everything stored; records how it was used.
#[derive(Default)]
pub struct BruteForceStore {
    entries: Mutex<HashMap<String, (Vec<f64>, String)>>,
    pub upserts: AtomicUsize,
    pub removes: AtomicUsize,
    pub searches: AtomicUsize,
}

impl BruteForceStore {
    pub fn ids(&self) -> BTreeSet<String> {
        self.entries.lock().unwrap().keys().cloned().collect()
    }
}

impl VectorStore for BruteForceStore {
    fn upsert(&self, id: &str, vector: &[f64], payload: &str) {
        self.upserts.fetch_add(1, Ordering::SeqCst);
        self.entries
            .lock()
            .unwrap()
            .insert(id.to_string(), (vector.to_vec(), payload.to_string()));
    }

    fn remove(&self, id: &str) -> bool {
        self.removes.fetch_add(1, Ordering::SeqCst);
        self.entries.lock().unwrap().remove(id).is_some()
    }

    fn top_k(&self, query: &[f64], k: usize) -> Vec<Hit> {
        self.searches.fetch_add(1, Ordering::SeqCst);
        let mut all: Vec<Hit> = self
            .entries
            .lock()
            .unwrap()
            .iter()
            .map(|(id, (v, p))| Hit {
                id: id.clone(),
                score: cosine(query, v),
                payload: p.clone(),
            })
            .collect();
        all.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap()
                .then_with(|| a.id.cmp(&b.id))
        });
        all.truncate(k);
        all
    }
}

/// "prefix0 prefix1 ... prefix{n-1}"
pub fn words(prefix: &str, n: usize) -> String {
    (0..n)
        .map(|i| format!("{prefix}{i}"))
        .collect::<Vec<_>>()
        .join(" ")
}
