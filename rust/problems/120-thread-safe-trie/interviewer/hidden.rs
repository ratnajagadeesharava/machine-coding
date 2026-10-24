#![allow(unused_imports, clippy::needless_range_loop)]
use p120_thread_safe_trie::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

fn word(i: usize) -> String {
    let digits = b"0123456789abcdefghijklmnopqrstuvwxyz";
    if i == 0 {
        return "0".into();
    }
    let mut n = i;
    let mut out = Vec::new();
    while n > 0 {
        out.push(digits[n % 36]);
        n /= 36;
    }
    out.reverse();
    String::from_utf8(out).unwrap()
}

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

#[test]
fn should_match_a_brute_force_model_on_random_single_threaded_operations() {
    let t = sut!(create());
    let mut model: BTreeSet<String> = BTreeSet::new();
    let mut rnd = Lcg(42);
    let alphabet: Vec<char> = "ab-1".chars().collect();

    for _ in 0..20_000 {
        let len = 1 + rnd.below(5);
        let w: String = (0..len).map(|_| alphabet[rnd.below(alphabet.len())]).collect();
        let given = if rnd.below(2) == 0 { w.clone() } else { w.to_uppercase() };
        match rnd.below(4) {
            0 => assert_eq!(t.insert(&given), Ok(model.insert(w))),
            1 => assert_eq!(t.delete(&given), Ok(model.remove(&w))),
            2 => assert_eq!(t.contains(&given), Ok(model.contains(&w))),
            _ => {
                let p = &w[..rnd.below(w.len() + 1)];
                let expected: Vec<String> =
                    model.iter().filter(|m| m.starts_with(p)).cloned().collect();
                let limit = 1 + rnd.below(8);
                assert_eq!(t.starts_with(p), Ok(!expected.is_empty()));
                assert_eq!(t.count_with_prefix(p), Ok(expected.len()));
                let cut = limit.min(expected.len());
                assert_eq!(t.words_with_prefix(p, limit), Ok(expected[..cut].to_vec()));
            }
        }
        assert_eq!(t.size(), model.len());
    }
}

#[test]
fn should_release_everything_when_all_words_are_deleted_in_any_order() {
    let t = sut!(create());
    let mut words: Vec<String> = (0..3_000).map(word).collect();
    for w in &words {
        t.insert(w).unwrap();
    }
    let mut rnd = Lcg(7);
    for i in (1..words.len()).rev() {
        words.swap(i, rnd.below(i + 1));
    }
    for w in &words {
        assert_eq!(t.delete(w), Ok(true));
        assert_eq!(t.contains(w), Ok(false));
    }

    assert_eq!(t.size(), 0);
    assert_eq!(t.starts_with(""), Ok(false));
    assert_eq!(t.count_with_prefix("1"), Ok(0));
    assert_eq!(t.words_with_prefix("", 10), Ok(vec![]));
    assert_eq!(t.insert("again"), Ok(true));
    assert_eq!(t.words_with_prefix("", 10), Ok(vec!["again".to_string()]));
}

#[test]
fn should_handle_two_hundred_thousand_words_with_many_threads_in_bounded_time() {
    let t = sut!(create());

    run_concurrently(8, |id| {
        for i in (id..200_000).step_by(8) {
            t.insert(&word(i)).unwrap();
        }
    });

    assert_eq!(t.size(), 200_000);
    for i in 0..20_000 {
        assert!(!t.words_with_prefix(&word(i * 5), 3).unwrap().is_empty());
    }
}

#[test]
fn should_keep_count_with_prefix_consistent_with_the_listing_while_writers_run() {
    let t = sut!(create());
    let done = AtomicBool::new(false);
    let writers_left = AtomicUsize::new(2);

    run_concurrently(6, |id| {
        if id < 2 {
            for _ in 0..30 {
                for i in (id..1_000).step_by(2) {
                    t.insert(&format!("k{}", word(i))).unwrap();
                }
                for i in (id..1_000).step_by(2) {
                    t.delete(&format!("k{}", word(i))).unwrap();
                }
            }
            if writers_left.fetch_sub(1, Ordering::SeqCst) == 1 {
                done.store(true, Ordering::SeqCst);
            }
            return;
        }
        while !done.load(Ordering::SeqCst) {
            assert!(t.count_with_prefix("k").unwrap() <= 1_000);
            assert!(t.words_with_prefix("k", 5_000).unwrap().len() <= 1_000);
            assert!(t.size() <= 1_000);
        }
    });

    assert_eq!(t.size(), 0);
}

#[test]
fn should_not_let_a_delete_of_a_word_remove_its_siblings_inserted_at_the_same_time() {
    let t = sut!(create());

    // "car" is repeatedly deleted while "card" / "care" (sharing its path) are inserted and kept
    run_concurrently(8, |id| {
        for i in 0..5_000 {
            match id % 4 {
                0 => {
                    t.insert("car").unwrap();
                    t.delete("car").unwrap();
                }
                1 => {
                    t.insert(&format!("card{}", i % 50)).unwrap();
                }
                2 => {
                    t.insert(&format!("care{}", i % 50)).unwrap();
                }
                _ => {
                    t.insert("ca").unwrap();
                    t.delete("ca").unwrap();
                }
            }
        }
    });

    assert_eq!(t.count_with_prefix("card"), Ok(50));
    assert_eq!(t.count_with_prefix("care"), Ok(50));
    assert_eq!(t.size(), 100);
    assert_eq!(t.starts_with("car"), Ok(true));
}

#[test]
fn should_stay_correct_under_repeated_races_on_the_same_small_word_set() {
    let t = sut!(create());
    let words = ["a", "ab", "abc", "abd", "b", "ba"];

    let net = collect_concurrently(8, |id| {
        let mut balance: i64 = 0;
        let mut rnd = Lcg(id as u64 + 1);
        for _ in 0..30_000 {
            let w = words[rnd.below(words.len())];
            if rnd.below(2) == 0 {
                if t.insert(w).unwrap() {
                    balance += 1;
                }
            } else if t.delete(w).unwrap() {
                balance -= 1;
            }
        }
        balance
    });

    let expected: i64 = net.iter().sum();
    assert!((0..=words.len() as i64).contains(&expected));
    assert_eq!(t.size() as i64, expected);
    assert_eq!(t.words_with_prefix("", 10).unwrap().len() as i64, expected);
    let present = words.iter().filter(|w| t.contains(w).unwrap()).count();
    assert_eq!(present as i64, expected);
}

#[test]
fn should_reject_invalid_input_without_disturbing_concurrent_work() {
    let t = sut!(create());

    run_concurrently(4, |id| {
        for i in 0..2_000 {
            let w = if i % 2 == 0 {
                "bad word".to_string()
            } else {
                format!("good{id}-{i}")
            };
            let r = t.insert(&w);
            if i % 2 == 0 {
                assert_eq!(r, Err(TrieError::InvalidArgument));
            }
        }
    });

    assert_eq!(t.size(), 4_000);
    assert_eq!(t.starts_with("bad"), Ok(false));
}
