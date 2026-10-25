#![allow(unused_imports, clippy::needless_range_loop)]
use p134_dictionary_app::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn v(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn name(mut n: usize) -> String {
    let mut s = String::from("w");
    loop {
        s.push((b'a' + (n % 26) as u8) as char);
        n /= 26;
        if n == 0 {
            return s;
        }
    }
}

fn some(s: &str) -> Option<String> {
    Some(s.to_string())
}

#[test]
fn should_cover_every_kind_of_edit_distance_one() {
    let d = sut!(create());
    for w in ["stone", "tone", "stoned", "store", "atone", "stove", "scone", "ston", "o'ne"] {
        d.add_word(w, "noun", "x").unwrap();
    }

    // insertion, deletion and substitution all count as one edit; a transposition or two edits does not
    assert_eq!(d.suggest("stne", 20), Ok(v(&["stone"])));
    assert_eq!(d.suggest("stoe", 20), Ok(v(&["ston", "stone", "store", "stove"])));
    assert_eq!(d.suggest("tone", 20), Ok(vec![]));
    assert_eq!(d.suggest("one", 20), Ok(v(&["o'ne", "tone"])));
    assert_eq!(d.suggest("stonee", 20), Ok(v(&["stone", "stoned"])));
    assert_eq!(d.suggest("sstonee", 20), Ok(vec![]));
}

#[test]
fn should_return_the_first_k_suggestions_in_alphabetical_order_not_insertion_order() {
    let d = sut!(create());
    for w in ["bat", "cat", "aat", "eat", "dat"] {
        d.add_word(w, "noun", "x").unwrap();
    }

    assert_eq!(d.suggest("xat", 3), Ok(v(&["aat", "bat", "cat"])));
    assert_eq!(d.suggest("at", 2), Ok(v(&["aat", "bat"])));
}

#[test]
fn should_follow_the_word_of_the_day_when_words_come_and_go() {
    let d = sut!(create());
    for w in ["delta", "alpha", "charlie", "bravo"] {
        d.add_word(w, "noun", "x").unwrap();
    }
    assert_eq!(d.word_of_the_day(i64::MAX), some("delta")); // MAX mod 4 = 3
    assert_eq!(d.word_of_the_day(i64::MIN), some("alpha")); // rem_euclid(MIN, 4) = 0
    assert_eq!(d.word_of_the_day(-3), some("bravo"));

    d.remove_word("alpha").unwrap();
    assert_eq!(d.word_of_the_day(0), some("bravo"));
    assert_eq!(d.word_of_the_day(3), some("bravo"));
    assert_eq!(d.word_of_the_day(i64::MIN), some("charlie")); // rem_euclid(MIN, 3) = 1

    d.add_word("aardvark", "noun", "x").unwrap();
    assert_eq!(d.word_of_the_day(0), some("aardvark"));
    for w in ["aardvark", "bravo", "charlie", "delta"] {
        d.remove_word(w).unwrap();
    }
    assert_eq!(d.word_of_the_day(0), None);
}

#[test]
fn should_keep_update_in_place_and_reject_only_real_duplicates() {
    let d = sut!(create());
    d.add_word("set", "noun", "a group").unwrap();
    d.add_word("set", "noun", "a tennis unit").unwrap();
    d.add_word("set", "verb", "to put down").unwrap();

    d.update_meaning("set", 0, "a collection").unwrap();
    d.update_meaning("set", 2, "a group").unwrap();

    assert_eq!(
        d.lookup("set"),
        Ok(v(&["NOUN: a collection", "NOUN: a tennis unit", "VERB: a group"]))
    );
    // the old text is free again
    assert_eq!(d.add_word("set", "noun", "a group"), Ok(true));
    // an update must not count as an add: still ignored when equal to an existing pair
    assert_eq!(d.add_word("set", "noun", "a collection"), Ok(false));
    assert_eq!(d.meaning_count(), 4);
}

#[test]
fn should_search_across_all_meanings_without_matching_across_meaning_boundaries() {
    let d = sut!(create());
    d.add_word("one", "noun", "alpha").unwrap();
    d.add_word("one", "noun", "beta").unwrap();
    d.add_word("two", "noun", "ALPHABET soup").unwrap();
    d.add_word("three", "adverb", "Gamma ray").unwrap();

    assert_eq!(d.search("alpha", 10), Ok(v(&["one", "two"])));
    assert_eq!(d.search("alphabeta", 10), Ok(vec![]));
    assert_eq!(d.search("alpha beta", 10), Ok(vec![]));
    assert_eq!(d.search("ray", 10), Ok(v(&["three"])));
    assert_eq!(d.search("gamma ray", 10), Ok(v(&["three"])));
    assert_eq!(d.search("a", 2), Ok(v(&["one", "three"])));
}

#[test]
fn should_stay_consistent_when_meanings_are_removed_and_the_word_disappears() {
    let d = sut!(create());
    d.add_word("a", "noun", "x").unwrap();
    d.add_word("b", "noun", "y").unwrap();
    d.add_word("b", "verb", "z").unwrap();
    d.add_synonym("a", "b").unwrap();

    d.remove_meaning("b", 0).unwrap();
    assert_eq!(d.synonyms("a"), Ok(v(&["b"])));
    d.remove_meaning("b", 0).unwrap();

    assert_eq!(d.size(), 1);
    assert_eq!(d.meaning_count(), 1);
    assert_eq!(d.synonyms("a"), Ok(vec![]));
    assert_eq!(d.prefix("b", 5), Ok(vec![]));
    assert_eq!(d.add_synonym("a", "b"), Err(DictError::UnknownWord));
}

#[test]
fn should_reject_a_synonym_link_to_a_removed_word() {
    let d = sut!(create());
    d.add_word("a", "noun", "x").unwrap();
    d.add_word("b", "noun", "y").unwrap();
    d.remove_word("b").unwrap();

    assert_eq!(d.add_synonym("a", "b"), Err(DictError::UnknownWord));
}

#[test]
fn should_link_the_same_two_words_in_opposite_directions_concurrently_without_deadlock() {
    let d = sut!(create());
    for i in 0..20 {
        d.add_word(&name(i), "noun", "x").unwrap();
    }

    run_concurrently(8, |t| {
        for _round in 0..500 {
            for i in 0..20 {
                let j = (i + 1 + t) % 20;
                if i == j {
                    continue;
                }
                if t % 2 == 0 {
                    d.add_synonym(&name(i), &name(j)).unwrap();
                } else {
                    d.add_synonym(&name(j), &name(i)).unwrap();
                }
            }
        }
    });

    for i in 0..20 {
        for s in d.synonyms(&name(i)).unwrap() {
            assert!(d.synonyms(&s).unwrap().contains(&name(i)));
        }
    }
}

#[test]
fn should_never_show_a_half_applied_word_removal_to_concurrent_readers() {
    let d = sut!(create());
    d.add_word("hub", "noun", "x").unwrap();
    d.add_word("spoke", "noun", "x").unwrap();

    run_concurrently(5, |t| {
        if t == 0 {
            for _ in 0..3_000 {
                d.remove_word("spoke").unwrap();
                d.add_word("spoke", "noun", "x").unwrap();
                d.add_synonym("hub", "spoke").unwrap();
            }
            return;
        }
        for _ in 0..6_000 {
            assert!(d.synonyms("hub").unwrap().len() <= 1);
            let n = d.size();
            assert!((1..=2).contains(&n));
        }
    });

    assert_eq!(d.synonyms("hub"), Ok(v(&["spoke"])));
    assert_eq!(d.synonyms("spoke"), Ok(v(&["hub"])));
}

#[test]
fn should_keep_insertion_order_visible_to_readers_while_writers_append() {
    let d = sut!(create());
    d.add_word("bank", "noun", "m0").unwrap();

    run_concurrently(5, |t| {
        if t == 0 {
            for i in 1..2_000 {
                d.add_word("bank", "noun", &format!("m{i}")).unwrap();
            }
            return;
        }
        for _ in 0..1_000 {
            let l = d.lookup("bank").unwrap();
            for (k, item) in l.iter().enumerate() {
                assert_eq!(item, &format!("NOUN: m{k}"));
            }
        }
    });

    assert_eq!(d.lookup("bank").unwrap().len(), 2_000);
}

#[test]
fn should_count_meanings_exactly_under_concurrent_adds_and_word_removals() {
    let d = sut!(create());

    run_concurrently(8, |t| {
        for i in 0..300 {
            let w = name(i % 30);
            if t % 4 == 3 {
                d.remove_word(&w).unwrap();
            } else {
                d.add_word(&w, "noun", &format!("t{t}i{i}")).unwrap();
            }
        }
    });

    let words = d.prefix("w", 1_000).unwrap();
    let total: usize = words.iter().map(|w| d.lookup(w).unwrap().len()).sum();
    assert_eq!(d.meaning_count(), total);
    assert_eq!(d.size(), words.len());
}

#[test]
fn should_answer_prefix_and_suggest_quickly_on_a_large_dictionary() {
    let d = sut!(create());
    for i in 0..30_000 {
        d.add_word(&name(i), "noun", &format!("meaning {i}")).unwrap();
    }

    let start = Instant::now();
    for i in 0..3_000 {
        assert_eq!(d.prefix("wa", 5).unwrap().len(), 5);
        assert_eq!(d.lookup(&name(i)).unwrap().len(), 1);
    }
    for _ in 0..20 {
        assert!(d.suggest("wqqqqx", 3).is_ok());
    }
    assert!(start.elapsed() < Duration::from_secs(20));
    assert_eq!(d.size(), 30_000);
}

#[test]
fn should_treat_different_cases_and_padding_as_the_same_word_everywhere() {
    let d = sut!(create());
    d.add_word("Apple", "noun", "a fruit").unwrap();

    assert_eq!(d.add_word(" APPLE\t", "noun", "a fruit"), Ok(false));
    d.add_word("apple", "noun", "a company").unwrap();
    d.add_word("pear", "noun", "a fruit").unwrap();
    d.add_synonym(" APPLE ", "Pear").unwrap();

    assert_eq!(d.size(), 2);
    assert_eq!(d.synonyms("pear"), Ok(v(&["apple"])));
    assert_eq!(d.remove_word("ApPlE"), Ok(true));
    assert_eq!(d.synonyms("pear"), Ok(vec![]));
    assert_eq!(d.search("fruit", 5), Ok(v(&["pear"])));
}
