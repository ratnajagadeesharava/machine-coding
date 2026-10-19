#![allow(unused_imports, clippy::needless_range_loop)]
use p80_parallel_word_count::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::collections::HashMap;
use std::sync::Arc;

struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Rng(0x9E37_79B9_7F4A_7C15 ^ (seed + 1).wrapping_mul(0xD1B5_4A32_D192_ED03))
    }
    fn next(&mut self, n: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % n as u64) as usize
    }
}

/// Straightforward single-threaded reference used to compute the expected answer.
fn expected(docs: &[String]) -> HashMap<String, u64> {
    let mut out = HashMap::new();
    for d in docs {
        let mut word = String::new();
        for c in d.chars().chain(std::iter::once(' ')) {
            if c.is_ascii_alphabetic() {
                word.push(c.to_ascii_lowercase());
            } else if !word.is_empty() {
                *out.entry(std::mem::take(&mut word)).or_insert(0) += 1;
            }
        }
    }
    out
}

fn vocabulary(n: usize) -> Vec<String> {
    let mut r = Rng::new(11);
    (0..n)
        .map(|i| {
            let len = 3 + r.next(10);
            let mut w: String = (0..len)
                .map(|_| (b'a' + r.next(26) as u8) as char)
                .collect();
            w.push((b'a' + (i % 26) as u8) as char);
            w
        })
        .collect()
}

fn document(vocab: &[String], r: &mut Rng, words: usize) -> String {
    let seps = [" ", ", ", ".\n", " - ", "\t", "! ", "123 ", "_"];
    let mut s = String::new();
    for _ in 0..words {
        let w = &vocab[r.next(vocab.len())];
        if r.next(2) == 0 {
            s.push_str(w);
        } else {
            s.push_str(&w.to_uppercase());
        }
        s.push_str(seps[r.next(seps.len())]);
    }
    s
}

fn documents(docs: usize, words_each: usize, seed: u64) -> Vec<String> {
    let vocab = vocabulary(64);
    let mut r = Rng::new(seed);
    (0..docs)
        .map(|_| document(&vocab, &mut r, words_each))
        .collect()
}

fn strings(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

fn map(pairs: &[(&str, u64)]) -> HashMap<String, u64> {
    pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

#[test]
fn should_count_words_case_insensitively() {
    let c = sut!(create());

    let result = c
        .count(&strings(&["Hello, hello WORLD!", "world of Hello"]), 2)
        .unwrap();

    assert_eq!(result, map(&[("hello", 3), ("world", 2), ("of", 1)]));
}

#[test]
fn should_treat_everything_but_ascii_letters_as_separators() {
    let c = sut!(create());

    let result = c
        .count(&strings(&["abc123def a-b_c\nd\te", "x.y,z;é"]), 1)
        .unwrap();

    assert_eq!(
        result,
        map(&[
            ("abc", 1),
            ("def", 1),
            ("a", 1),
            ("b", 1),
            ("c", 1),
            ("d", 1),
            ("e", 1),
            ("x", 1),
            ("y", 1),
            ("z", 1)
        ])
    );
}

#[test]
fn should_handle_empty_input() {
    let c = sut!(create());

    assert!(c.count(&[], 4).unwrap().is_empty());
    assert!(c
        .count(&strings(&["", "   ", "123 !!"]), 4)
        .unwrap()
        .is_empty());
}

#[test]
fn should_reject_zero_parallelism() {
    let c = sut!(create());

    assert_eq!(c.count(&strings(&["a"]), 0), Err(InvalidParallelism));
}

#[test]
fn should_match_the_sequential_answer_for_every_parallelism() {
    let c = sut!(create());
    let docs = documents(200, 500, 1);
    let expected = expected(&docs);

    for p in [1, 2, 3, 4, 8, 16, 64] {
        assert!(c.count(&docs, p).unwrap() == expected, "parallelism {p}");
    }
}

#[test]
fn should_count_one_huge_document_correctly() {
    let c = sut!(create());
    // one document: a split inside it must never cut a word in half
    let docs = vec![document(&vocabulary(64), &mut Rng::new(5), 300_000)];
    let expected = expected(&docs);

    for p in [1, 4, 8] {
        assert!(c.count(&docs, p).unwrap() == expected, "parallelism {p}");
    }
}

#[test]
fn should_handle_fewer_documents_than_threads() {
    let c = sut!(create());
    let docs = documents(3, 1_000, 9);

    assert!(c.count(&docs, 32).unwrap() == expected(&docs));
}

#[test]
fn should_add_up_to_the_total_number_of_words() {
    let c = sut!(create());
    let docs = documents(100, 300, 21);

    let total: u64 = c.count(&docs, 8).unwrap().values().sum();

    assert_eq!(total, 100 * 300);
}

#[test]
fn should_be_usable_from_several_threads_at_once() {
    let c = sut!(create());

    run_concurrently(6, |t| {
        let docs = documents(60, 200, 100 + t as u64);
        assert!(c.count(&docs, 4).unwrap() == expected(&docs));
    });
}

#[test]
fn should_lose_no_counts_over_100_runs() {
    let c = sut!(create());
    let mut r = Rng::new(77);
    for run in 0..100u64 {
        let docs = documents(1 + r.next(80), 1 + r.next(300), run);
        let p = 1 + r.next(12);

        assert!(
            c.count(&docs, p).unwrap() == expected(&docs),
            "run {run}, parallelism {p}"
        );
    }
}
