#![allow(unused_imports)]
use p134_dictionary_app::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::sync::Arc;

fn v(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

/// cat, car, cart, dog, hound: the dictionary used in the README examples.
fn sample() -> Option<Arc<dyn DictionaryApp>> {
    create().map(|d| {
        d.add_word("cat", "noun", "small domesticated feline").unwrap();
        d.add_word("car", "noun", "a road vehicle").unwrap();
        d.add_word("cart", "noun", "a vehicle pulled by an animal").unwrap();
        d.add_word("dog", "noun", "a domesticated canine").unwrap();
        d.add_word("hound", "noun", "a dog used for hunting").unwrap();
        d
    })
}

fn letters(mut n: usize) -> String {
    let mut s = String::new();
    loop {
        s.push((b'a' + (n % 26) as u8) as char);
        n /= 26;
        if n == 0 {
            return s;
        }
    }
}

#[test]
fn should_store_a_meaning_and_look_it_up() {
    let d = sut!(create());

    assert_eq!(d.add_word("bank", "NOUN", "a financial institution"), Ok(true));

    assert_eq!(d.lookup("bank"), Ok(v(&["NOUN: a financial institution"])));
    assert_eq!(d.size(), 1);
    assert_eq!(d.meaning_count(), 1);
}

#[test]
fn should_keep_several_meanings_in_insertion_order() {
    let d = sut!(create());
    d.add_word("bank", "NOUN", "a financial institution").unwrap();
    d.add_word("bank", "VERB", "to tilt an aircraft").unwrap();
    d.add_word("bank", "NOUN", "the land beside a river").unwrap();

    assert_eq!(
        d.lookup("bank"),
        Ok(v(&[
            "NOUN: a financial institution",
            "VERB: to tilt an aircraft",
            "NOUN: the land beside a river"
        ]))
    );
    assert_eq!(d.size(), 1);
    assert_eq!(d.meaning_count(), 3);
}

#[test]
fn should_ignore_the_same_part_of_speech_and_meaning_twice() {
    let d = sut!(create());
    assert_eq!(d.add_word("bank", "NOUN", "a financial institution"), Ok(true));

    assert_eq!(d.add_word("bank", "noun", "  a financial institution "), Ok(false));
    assert_eq!(d.add_word("bank", "VERB", "a financial institution"), Ok(true));
    assert_eq!(d.add_word("bank", "NOUN", "A financial institution"), Ok(true));

    assert_eq!(d.meaning_count(), 3);
}

#[test]
fn should_lower_case_and_trim_words_on_every_call() {
    let d = sut!(create());
    d.add_word("  Bank ", "noun", "a financial institution").unwrap();

    assert_eq!(d.lookup("BANK"), Ok(v(&["NOUN: a financial institution"])));
    assert_eq!(d.lookup(" bank").unwrap().len(), 1);
    assert_eq!(d.prefix("BA", 5), Ok(v(&["bank"])));
    assert_eq!(d.size(), 1);
}

#[test]
fn should_normalise_part_of_speech_and_trim_the_meaning() {
    let d = sut!(create());
    d.add_word("run", " verb ", "  to move fast on foot  ").unwrap();

    assert_eq!(d.lookup("run"), Ok(v(&["VERB: to move fast on foot"])));
}

#[test]
fn should_accept_hyphens_and_apostrophes_in_words() {
    let d = sut!(create());

    assert_eq!(d.add_word("Well-known", "adjective", "famous"), Ok(true));
    assert_eq!(d.add_word("o'clock", "adverb", "by the clock"), Ok(true));

    assert_eq!(d.lookup("well-known"), Ok(v(&["ADJECTIVE: famous"])));
    assert_eq!(d.lookup("O'Clock"), Ok(v(&["ADVERB: by the clock"])));
}

#[test]
fn should_reject_invalid_words_parts_of_speech_and_meanings() {
    let d = sut!(create());
    let bad = DictError::InvalidArgument;

    assert_eq!(d.add_word("  ", "NOUN", "x"), Err(DictError::InvalidArgument));
    assert_eq!(d.add_word("two words", "NOUN", "x"), Err(DictError::InvalidArgument));
    assert_eq!(d.add_word("abc1", "NOUN", "x"), Err(DictError::InvalidArgument));
    assert_eq!(d.add_word("caf\u{e9}", "NOUN", "x"), Err(DictError::InvalidArgument));
    assert_eq!(d.add_word("under_score", "NOUN", "x"), Err(DictError::InvalidArgument));
    assert_eq!(d.add_word("word", "PREPOSITION", "x"), Err(DictError::InvalidArgument));
    assert_eq!(d.add_word("word", "", "x"), Err(DictError::InvalidArgument));
    assert_eq!(d.add_word("word", "NOUN", " "), Err(bad));

    assert_eq!(d.size(), 0);
    assert_eq!(d.meaning_count(), 0);
    assert_eq!(d.lookup("bad word"), Err(DictError::InvalidArgument));
    assert_eq!(d.lookup(" "), Err(DictError::InvalidArgument));
}

#[test]
fn should_return_an_empty_list_for_an_unknown_word() {
    let d = sut!(sample());

    assert_eq!(d.lookup("zebra"), Ok(vec![]));
    assert_eq!(d.synonyms("zebra"), Ok(vec![]));
}

#[test]
fn should_return_a_lookup_that_later_changes_do_not_affect() {
    let d = sut!(create());
    d.add_word("bank", "NOUN", "a financial institution").unwrap();
    let returned = d.lookup("bank").unwrap();

    d.add_word("bank", "VERB", "to tilt an aircraft").unwrap();

    assert_eq!(returned, v(&["NOUN: a financial institution"]));
}

#[test]
fn should_update_a_meaning_in_place() {
    let d = sut!(create());
    d.add_word("bank", "NOUN", "a financial institution").unwrap();
    d.add_word("bank", "VERB", "to tilt an aircraft").unwrap();

    d.update_meaning("Bank", 1, "  to lean into a turn ").unwrap();

    assert_eq!(
        d.lookup("bank"),
        Ok(v(&["NOUN: a financial institution", "VERB: to lean into a turn"]))
    );
    assert_eq!(d.meaning_count(), 2);
}

#[test]
fn should_reject_bad_updates() {
    let d = sut!(create());
    d.add_word("bank", "NOUN", "a financial institution").unwrap();
    d.add_word("bank", "NOUN", "the land beside a river").unwrap();
    d.add_word("bank", "VERB", "the land beside a river").unwrap();

    assert_eq!(d.update_meaning("nope", 0, "x"), Err(DictError::UnknownWord));
    assert_eq!(d.update_meaning("bank", 3, "x"), Err(DictError::InvalidIndex));
    assert_eq!(d.update_meaning("bank", -1, "x"), Err(DictError::InvalidIndex));
    assert_eq!(d.update_meaning("bank", 0, "  "), Err(DictError::InvalidArgument));
    assert_eq!(
        d.update_meaning("bank", 0, "the land beside a river"),
        Err(DictError::DuplicateMeaning)
    );

    assert_eq!(
        d.lookup("bank"),
        Ok(v(&[
            "NOUN: a financial institution",
            "NOUN: the land beside a river",
            "VERB: the land beside a river"
        ]))
    );
}

#[test]
fn should_allow_updating_a_meaning_to_its_own_text_or_to_another_part_of_speechs_text() {
    let d = sut!(create());
    d.add_word("bank", "NOUN", "a financial institution").unwrap();
    d.add_word("bank", "VERB", "to tilt an aircraft").unwrap();

    d.update_meaning("bank", 0, "a financial institution").unwrap();
    d.update_meaning("bank", 1, "a financial institution").unwrap();

    assert_eq!(
        d.lookup("bank"),
        Ok(v(&["NOUN: a financial institution", "VERB: a financial institution"]))
    );
}

#[test]
fn should_shift_later_meanings_down_when_one_is_removed() {
    let d = sut!(create());
    d.add_word("bank", "NOUN", "first").unwrap();
    d.add_word("bank", "NOUN", "second").unwrap();
    d.add_word("bank", "NOUN", "third").unwrap();

    d.remove_meaning("bank", 0).unwrap();

    assert_eq!(d.lookup("bank"), Ok(v(&["NOUN: second", "NOUN: third"])));
    d.remove_meaning("bank", 1).unwrap();
    assert_eq!(d.lookup("bank"), Ok(v(&["NOUN: second"])));
    assert_eq!(d.meaning_count(), 1);
}

#[test]
fn should_remove_the_word_when_its_last_meaning_is_removed() {
    let d = sut!(sample());

    d.remove_meaning("dog", 0).unwrap();

    assert_eq!(d.lookup("dog"), Ok(vec![]));
    assert_eq!(d.size(), 4);
    assert_eq!(d.remove_word("dog"), Ok(false));
}

#[test]
fn should_reject_bad_meaning_removals() {
    let d = sut!(sample());

    assert_eq!(d.remove_meaning("nope", 0), Err(DictError::UnknownWord));
    assert_eq!(d.remove_meaning("dog", 1), Err(DictError::InvalidIndex));
    assert_eq!(d.remove_meaning("dog", -1), Err(DictError::InvalidIndex));
    assert_eq!(d.lookup("dog").unwrap().len(), 1);
}

#[test]
fn should_remove_a_word_and_report_whether_it_existed() {
    let d = sut!(sample());

    assert_eq!(d.remove_word(" DOG "), Ok(true));
    assert_eq!(d.remove_word("dog"), Ok(false));
    assert_eq!(d.remove_word("never-added"), Ok(false));

    assert_eq!(d.lookup("dog"), Ok(vec![]));
    assert_eq!(d.size(), 4);
    assert_eq!(d.meaning_count(), 4);
    assert_eq!(d.remove_word(""), Err(DictError::InvalidArgument));
}

#[test]
fn should_link_synonyms_both_ways_sorted_and_without_transitivity() {
    let d = sut!(sample());
    d.add_word("canine", "noun", "a dog or related animal").unwrap();

    assert_eq!(d.add_synonym("dog", "hound"), Ok(true));
    assert_eq!(d.add_synonym("hound", "dog"), Ok(false));
    assert_eq!(d.add_synonym("Canine", "hound"), Ok(true));
    assert_eq!(d.add_synonym("dog", "canine"), Ok(true));

    assert_eq!(d.synonyms("dog"), Ok(v(&["canine", "hound"])));
    assert_eq!(d.synonyms("hound"), Ok(v(&["canine", "dog"])));
    assert_eq!(d.synonyms("cat"), Ok(vec![]));
}

#[test]
fn should_not_make_synonyms_transitive() {
    let d = sut!(sample());
    d.add_synonym("cat", "dog").unwrap();
    d.add_synonym("dog", "hound").unwrap();

    assert_eq!(d.synonyms("cat"), Ok(v(&["dog"])));
    assert_eq!(d.synonyms("hound"), Ok(v(&["dog"])));
}

#[test]
fn should_reject_bad_synonym_links() {
    let d = sut!(sample());

    assert_eq!(d.add_synonym("dog", "dog"), Err(DictError::InvalidArgument));
    assert_eq!(d.add_synonym("dog", " DOG "), Err(DictError::InvalidArgument));
    assert_eq!(d.add_synonym("dog", "wolf"), Err(DictError::UnknownWord));
    assert_eq!(d.add_synonym("wolf", "dog"), Err(DictError::UnknownWord));
    assert_eq!(d.add_synonym("dog", ""), Err(DictError::InvalidArgument));

    assert_eq!(d.synonyms("dog"), Ok(vec![]));
}

#[test]
fn should_drop_synonym_links_when_a_word_is_removed() {
    let d = sut!(sample());
    d.add_synonym("dog", "hound").unwrap();
    d.add_synonym("cat", "dog").unwrap();

    d.remove_word("dog").unwrap();

    assert_eq!(d.synonyms("hound"), Ok(vec![]));
    assert_eq!(d.synonyms("cat"), Ok(vec![]));
    d.add_word("dog", "noun", "a domesticated canine").unwrap();
    assert_eq!(d.synonyms("dog"), Ok(vec![]));
    assert_eq!(d.synonyms("hound"), Ok(vec![]));
}

#[test]
fn should_list_words_by_prefix_sorted_and_limited() {
    let d = sut!(sample());

    assert_eq!(d.prefix("ca", 10), Ok(v(&["car", "cart", "cat"])));
    assert_eq!(d.prefix("ca", 2), Ok(v(&["car", "cart"])));
    assert_eq!(d.prefix("CART", 5), Ok(v(&["cart"])));
    assert_eq!(d.prefix("z", 5), Ok(vec![]));
    assert_eq!(d.prefix("d", 1), Ok(v(&["dog"])));
}

#[test]
fn should_reject_bad_prefix_queries() {
    let d = sut!(sample());

    assert_eq!(d.prefix("ca", 0), Err(DictError::InvalidArgument));
    assert_eq!(d.prefix("ca", -3), Err(DictError::InvalidArgument));
    assert_eq!(d.prefix(" ", 3), Err(DictError::InvalidArgument));
    assert_eq!(d.prefix("c4", 3), Err(DictError::InvalidArgument));
}

#[test]
fn should_search_meanings_case_insensitively_sorted_by_word() {
    let d = sut!(sample());

    assert_eq!(d.search("DOMESTICATED", 10), Ok(v(&["cat", "dog"])));
    assert_eq!(d.search("vehicle", 10), Ok(v(&["car", "cart"])));
    assert_eq!(d.search("vehicle", 1), Ok(v(&["car"])));
    assert_eq!(d.search("  an animal ", 10), Ok(v(&["cart"])));
    assert_eq!(d.search("quantum", 10), Ok(vec![]));
}

#[test]
fn should_search_only_meanings_and_list_each_word_once() {
    let d = sut!(sample());
    d.add_word("dog", "verb", "to follow closely").unwrap();
    d.add_word("dog", "verb", "to pursue like a dog").unwrap();

    assert_eq!(d.search("dog", 10), Ok(v(&["dog", "hound"])));
    assert_eq!(d.search("hound", 10), Ok(vec![]));
    assert_eq!(d.search("NOUN", 10), Ok(vec![]));
}

#[test]
fn should_reject_bad_search_queries() {
    let d = sut!(sample());

    assert_eq!(d.search("dog", 0), Err(DictError::InvalidArgument));
    assert_eq!(d.search("  ", 5), Err(DictError::InvalidArgument));
}

#[test]
fn should_suggest_words_at_edit_distance_one_for_an_unknown_word() {
    let d = sut!(sample());

    assert_eq!(d.suggest("cag", 5), Ok(v(&["car", "cat"])));
    assert_eq!(d.suggest("do", 5), Ok(v(&["dog"])));
    assert_eq!(d.suggest("cats", 5), Ok(v(&["cat"])));
    assert_eq!(d.suggest("hund", 5), Ok(v(&["hound"])));
    assert_eq!(d.suggest("CAG", 1), Ok(v(&["car"])));
    assert_eq!(d.suggest("zzz", 5), Ok(vec![]));
}

#[test]
fn should_not_suggest_for_a_known_word_and_reject_bad_suggest_input() {
    let d = sut!(sample());

    assert_eq!(d.suggest("cat", 5), Ok(vec![]));
    assert_eq!(d.suggest("cag", 0), Err(DictError::InvalidArgument));
    assert_eq!(d.suggest(" ", 3), Err(DictError::InvalidArgument));
    assert_eq!(d.suggest("c4g", 3), Err(DictError::InvalidArgument));
}

#[test]
fn should_pick_the_word_of_the_day_from_the_sorted_words() {
    let d = sut!(sample());

    assert_eq!(d.word_of_the_day(0), Some("car".to_string()));
    assert_eq!(d.word_of_the_day(2), Some("cat".to_string()));
    assert_eq!(d.word_of_the_day(7), Some("cat".to_string()));
    assert_eq!(d.word_of_the_day(5), Some("car".to_string()));
    assert_eq!(d.word_of_the_day(-1), Some("hound".to_string()));
    assert_eq!(d.word_of_the_day(-5), Some("car".to_string()));
}

#[test]
fn should_have_no_word_of_the_day_in_an_empty_dictionary() {
    let d = sut!(create());

    assert_eq!(d.word_of_the_day(0), None);
    assert_eq!(d.word_of_the_day(-12), None);
    assert_eq!(d.size(), 0);
    assert_eq!(d.meaning_count(), 0);
}

#[test]
fn should_sort_words_by_plain_character_code() {
    let d = sut!(create());
    d.add_word("a-b", "noun", "x").unwrap();
    d.add_word("a'b", "noun", "x").unwrap();
    d.add_word("ab", "noun", "x").unwrap();
    d.add_word("a", "noun", "x").unwrap();

    assert_eq!(d.prefix("a", 10), Ok(v(&["a", "a'b", "a-b", "ab"])));
    assert_eq!(d.word_of_the_day(1), Some("a'b".to_string()));
}

#[test]
fn should_lose_no_meaning_when_many_threads_add_different_meanings_to_the_same_word() {
    let d = sut!(create());

    run_concurrently(16, |t| {
        for i in 0..100 {
            d.add_word("bank", "NOUN", &format!("meaning {t}-{i}")).unwrap();
        }
    });

    assert_eq!(d.lookup("bank").unwrap().len(), 1_600);
    assert_eq!(d.meaning_count(), 1_600);
    assert_eq!(d.size(), 1);
}

#[test]
fn should_accept_the_same_meaning_from_many_threads_exactly_once() {
    let d = sut!(create());

    let results = collect_concurrently(16, |_| {
        d.add_word("bank", "noun", "a financial institution").unwrap()
    });

    assert_eq!(results.iter().filter(|r| **r).count(), 1);
    assert_eq!(d.lookup("bank"), Ok(v(&["NOUN: a financial institution"])));
}

#[test]
fn should_create_each_word_once_when_threads_add_different_words_and_meanings() {
    let d = sut!(create());

    run_concurrently(8, |t| {
        for i in 0..200 {
            d.add_word(&format!("w{}", letters(i)), "verb", &format!("by thread {t}"))
                .unwrap();
        }
    });

    assert_eq!(d.size(), 200);
    assert_eq!(d.meaning_count(), 1_600);
}
