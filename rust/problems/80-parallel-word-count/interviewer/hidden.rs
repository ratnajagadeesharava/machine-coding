#![allow(unused_imports, clippy::needless_range_loop)]
use p80_parallel_word_count::{contract::*, create};
use playground_support::sut;

#[test]
fn should_count_one_million_words_of_a_very_hot_vocabulary() {
    let c = sut!(create());
    let doc = "the quick brown fox jumps over the lazy dog ".repeat(125); // 1_125 words
    let docs: Vec<String> = (0..1_000).map(|_| doc.clone()).collect();

    let result = c.count(&docs, 16).unwrap();

    assert_eq!(result["the"], 2 * 125 * 1_000);
    assert_eq!(result["dog"], 125 * 1_000);
    assert_eq!(result.values().sum::<u64>(), 9 * 125 * 1_000);
}

#[test]
fn should_count_hundreds_of_thousands_of_distinct_words() {
    let c = sut!(create());
    let mut s = 0x9E37_79B9_7F4A_7C15u64;
    let mut docs = Vec::new();
    for _ in 0..100 {
        let mut d = String::new();
        for _ in 0..5_000 {
            for _ in 0..6 {
                s ^= s << 13;
                s ^= s >> 7;
                s ^= s << 17;
                d.push((b'a' + (s % 26) as u8) as char);
            }
            d.push(' ');
        }
        docs.push(d);
    }

    let result = c.count(&docs, 8).unwrap();

    assert_eq!(result.values().sum::<u64>(), 100 * 5_000);
}
