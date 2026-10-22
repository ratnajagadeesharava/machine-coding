#![allow(unused_imports, clippy::needless_range_loop)]
use p116_inverted_index_search::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::collections::{BTreeMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

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

#[test]
fn should_shift_idf_as_unrelated_documents_are_added() {
    let e = sut!(create());
    e.add_document("a", "rare words here").unwrap();
    assert!(close(e.score_of("rare", "a").unwrap(), idf(1, 1) / 3.0));

    for i in 0..9 {
        e.add_document(&format!("filler{i}"), "nothing relevant")
            .unwrap();
    }

    assert!(close(e.score_of("rare", "a").unwrap(), idf(10, 1) / 3.0));
}

#[test]
fn should_flip_the_ranking_when_an_update_changes_term_frequencies() {
    let e = sut!(create());
    e.add_document("a", "cat dog").unwrap();
    e.add_document("b", "cat dog bird fish").unwrap();
    e.add_document("c", "bird").unwrap();
    assert_eq!(ids(e.search("cat", "OR", 5)), vec!["a", "b"]);

    e.update_document("a", "x y z w cat").unwrap();
    e.update_document("b", "cat").unwrap();

    assert_eq!(ids(e.search("cat", "OR", 5)), vec!["b", "a"]);
    assert!(ids(e.search("dog", "OR", 5)).is_empty());
    assert_eq!(ids(e.search("bird", "OR", 5)), vec!["c"]);
    assert_eq!(e.document_count(), 3);
}

#[test]
fn should_work_from_an_empty_collection_and_after_everything_was_deleted() {
    let e = sut!(create());
    assert!(ids(e.search("anything", "OR", 5)).is_empty());
    assert_eq!(e.document_count(), 0);

    e.add_document("a", "one two").unwrap();
    e.add_document("b", "two three").unwrap();
    e.delete_document("a").unwrap();
    e.delete_document("b").unwrap();

    assert_eq!(e.document_count(), 0);
    assert!(ids(e.search("two", "OR", 5)).is_empty());
    e.add_document("a", "two").unwrap();
    assert_eq!(ids(e.search("two", "AND", 5)), vec!["a"]);
    assert!(close(e.score_of("two", "a").unwrap(), idf(1, 1)));
}

#[test]
fn should_treat_tokens_with_digits_and_mixed_case_as_single_terms() {
    let e = sut!(create());
    e.add_document("a", "Model X100 vs model-X200").unwrap();
    e.add_document("b", "x100x200").unwrap();

    assert_eq!(ids(e.search("x100", "OR", 5)), vec!["a"]);
    assert_eq!(ids(e.search("X200 MODEL", "AND", 5)), vec!["a"]);
    assert_eq!(ids(e.search("x100x200", "AND", 5)), vec!["b"]);
    assert!(close(
        e.score_of("model", "a").unwrap(),
        2.0 / 5.0 * idf(2, 1)
    ));
}

struct Lcg(u64);
impl Lcg {
    fn next(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) as usize) % n
    }
}

fn tokens(s: &str) -> Vec<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(String::from)
        .collect()
}

fn distinct(q: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    tokens(q)
        .into_iter()
        .filter(|t| seen.insert(t.clone()))
        .collect()
}

fn score(docs: &BTreeMap<String, String>, q: &str, id: &str) -> f64 {
    let doc = tokens(&docs[id]);
    let mut s = 0.0;
    for t in distinct(q) {
        let tf = doc.iter().filter(|x| **x == t).count();
        if tf == 0 {
            continue;
        }
        let df = docs.values().filter(|d| tokens(d).contains(&t)).count();
        s += (tf as f64 / doc.len() as f64) * idf(docs.len(), df);
    }
    s
}

fn rank(docs: &BTreeMap<String, String>, q: &str, and: bool) -> Vec<String> {
    let terms = distinct(q);
    if terms.is_empty() {
        return vec![];
    }
    let mut v: Vec<(String, f64)> = docs
        .keys()
        .filter(|id| {
            let d = tokens(&docs[*id]);
            if and {
                terms.iter().all(|t| d.contains(t))
            } else {
                terms.iter().any(|t| d.contains(t))
            }
        })
        .map(|id| (id.clone(), score(docs, q, id)))
        .collect();
    v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then_with(|| a.0.cmp(&b.0)));
    v.into_iter().map(|(id, _)| id).collect()
}

fn random_text(r: &mut Lcg, vocab: &[&str]) -> String {
    let n = r.next(7);
    let mut s = String::new();
    for i in 0..n {
        s.push_str(vocab[r.next(vocab.len())]);
        s.push_str(if i % 2 == 0 { " " } else { ", " });
    }
    s
}

#[test]
fn should_match_a_brute_force_oracle_after_many_random_operations() {
    let e = sut!(create());
    let mut oracle: BTreeMap<String, String> = BTreeMap::new();
    let mut r = Lcg(42);
    let vocab = [
        "red", "green", "blue", "cat", "dog", "fish", "sun", "moon", "star", "tree", "rock", "sand",
    ];

    for step in 0..1_500 {
        let id = format!("doc{}", r.next(120));
        let op = r.next(10);
        let text = random_text(&mut r, &vocab);
        if op < 5 {
            if let std::collections::btree_map::Entry::Vacant(v) = oracle.entry(id.clone()) {
                e.add_document(&id, &text).unwrap();
                v.insert(text);
            }
        } else if op < 7 {
            if oracle.contains_key(&id) {
                e.update_document(&id, &text).unwrap();
                oracle.insert(id, text);
            }
        } else if oracle.contains_key(&id) {
            e.delete_document(&id).unwrap();
            oracle.remove(&id);
        }
        if step % 25 == 0 {
            let query = random_text(&mut r, &vocab);
            for (mode, and) in [("AND", true), ("OR", false)] {
                assert_eq!(
                    ids(e.search(&query, mode, 1_000)),
                    rank(&oracle, &query, and),
                    "step {step} query '{query}' {mode}"
                );
            }
        }
    }
    assert_eq!(e.document_count(), oracle.len());
    for id in oracle.keys() {
        assert!(close(
            e.score_of("red cat star", id).unwrap(),
            score(&oracle, "red cat star", id)
        ));
    }
}

#[test]
fn should_never_show_a_half_applied_update_to_concurrent_searches() {
    let e = sut!(create());
    e.add_document("flip", "alpha beta").unwrap();
    e.add_document("other", "gamma delta").unwrap();
    let done = AtomicBool::new(false);

    run_concurrently(5, |t| {
        if t == 0 {
            for i in 0..20_000 {
                e.update_document(
                    "flip",
                    if i % 2 == 0 {
                        "gamma delta"
                    } else {
                        "alpha beta"
                    },
                )
                .unwrap();
            }
            done.store(true, Ordering::SeqCst);
            return;
        }
        while !done.load(Ordering::SeqCst) {
            // the document is always exactly one of the two versions; never a mixture, never absent
            assert!(ids(e.search("alpha delta", "AND", 5)).is_empty());
            let mut r = ids(e.search("alpha gamma", "OR", 5));
            r.sort();
            assert_eq!(r, vec!["flip", "other"]);
            assert_eq!(e.document_count(), 2);
        }
    });
}

#[test]
fn should_stay_fast_on_a_large_collection_with_selective_queries() {
    let e = sut!(create());
    for i in 0..30_000 {
        e.add_document(
            &format!("d{i}"),
            &format!("common filler words w{} more text here", i % 3_000),
        )
        .unwrap();
    }

    let start = Instant::now();
    for q in 0..20_000 {
        let r = ids(e.search(&format!("w{} nothing", q % 3_000), "OR", 5));
        assert_eq!(r.len(), 5);
    }
    assert_eq!(ids(e.search("common w7", "AND", 100)).len(), 10);
    assert!(
        start.elapsed() < Duration::from_secs(30),
        "searches are too slow: {:?}",
        start.elapsed()
    );
}

#[test]
fn should_page_consistently_through_a_shrinking_collection() {
    let e = sut!(create());
    for i in 0..20 {
        e.add_document(&format!("p{i:02}"), "topic").unwrap();
    }
    assert_eq!(
        ids(e.search_page("topic", "OR", 0, 5)),
        vec!["p00", "p01", "p02", "p03", "p04"]
    );

    e.delete_document("p00").unwrap();
    e.delete_document("p07").unwrap();

    assert_eq!(
        ids(e.search_page("topic", "OR", 5, 5)),
        vec!["p06", "p08", "p09", "p10", "p11"]
    );
    assert_eq!(ids(e.search_page("topic", "OR", 16, 5)), vec!["p18", "p19"]);
}
