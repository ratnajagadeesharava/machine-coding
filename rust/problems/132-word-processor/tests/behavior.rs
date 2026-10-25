#![allow(unused_imports, clippy::needless_range_loop)]
use p132_word_processor::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::Arc;
use std::time::Instant;

const DEF: &str = "Calibri|11|-|-|-|black";

macro_rules! doc {
    () => {
        sut!(create())
    };
    ($text:expr) => {{
        let d = sut!(create());
        d.insert(0, $text).unwrap();
        d
    }};
}

fn italic() -> String {
    DEF.replace("-|-|-", "-|I|-")
}

#[test]
fn should_start_empty() {
    let d = doc!();

    assert_eq!(d.text(), "");
    assert_eq!(d.length(), 0);
    assert!(d.runs().is_empty());
    assert_eq!(d.word_count(), 0);
    assert_eq!(d.line_count(), 1);
    assert_eq!(d.alignment(0), Ok("LEFT".to_string()));
    assert_eq!(d.distinct_format_count(), 0);
}

#[test]
fn should_give_inserted_text_the_default_format_in_an_empty_document() {
    let d = doc!("hello");

    assert_eq!(d.text(), "hello");
    assert_eq!(d.length(), 5);
    assert_eq!(d.format_at(0), Ok(DEF.to_string()));
    assert_eq!(d.format_at(4), Ok(DEF.to_string()));
    assert_eq!(d.runs(), vec![format!("0-5:{DEF}")]);
}

#[test]
fn should_insert_at_the_start_in_the_middle_and_at_the_end() {
    let d = doc!("ace");
    d.insert(1, "b").unwrap();
    d.insert(3, "d").unwrap();
    d.insert(0, ">").unwrap();
    d.insert(d.length(), "<").unwrap();
    d.insert(2, "").unwrap();

    assert_eq!(d.text(), ">abcde<");
}

#[test]
fn should_reject_insert_at_an_invalid_position() {
    let d = doc!("abc");

    assert_eq!(d.insert(4, "x"), Err(WordError::OutOfRange));
    assert_eq!(d.text(), "abc");
}

#[test]
fn should_give_inserted_text_the_format_of_the_preceding_character() {
    let d = doc!("abcd");
    d.apply_format(0, 2, &[("bold", "true"), ("color", "red")]).unwrap();
    d.apply_format(2, 4, &[("italic", "true")]).unwrap();

    d.insert(1, "X").unwrap();
    assert_eq!(d.format_at(1), Ok("Calibri|11|B|-|-|red".to_string()));
    d.insert(4, "Y").unwrap();
    assert_eq!(d.format_at(4), Ok(italic()));
    d.insert(d.length(), "Z").unwrap();
    assert_eq!(d.format_at(d.length() - 1), Ok(italic()));
    assert_eq!(d.text(), "aXbcYdZ");
}

#[test]
fn should_use_the_default_format_when_inserting_at_position_zero_even_if_the_first_character_is_formatted() {
    let d = doc!("abc");
    d.apply_format(0, 3, &[("font", "Arial"), ("size", "20"), ("underline", "true")]).unwrap();

    d.insert(0, "X").unwrap();

    assert_eq!(d.format_at(0), Ok(DEF.to_string()));
    assert_eq!(d.format_at(1), Ok("Arial|20|-|-|U|black".to_string()));
    assert_eq!(d.runs(), vec![format!("0-1:{DEF}"), "1-4:Arial|20|-|-|U|black".to_string()]);
}

#[test]
fn should_use_the_default_format_again_after_everything_was_deleted() {
    let d = doc!("abc");
    d.apply_format(0, 3, &[("bold", "true")]).unwrap();
    d.delete(0, 3).unwrap();
    assert_eq!(d.length(), 0);
    assert!(d.runs().is_empty());

    d.insert(0, "new").unwrap();

    assert_eq!(d.runs(), vec![format!("0-3:{DEF}")]);
}

#[test]
fn should_delete_a_half_open_range() {
    let d = doc!("0123456789");
    d.delete(2, 5).unwrap();
    assert_eq!(d.text(), "0156789");
    d.delete(0, 1).unwrap();
    assert_eq!(d.text(), "156789");
    d.delete(4, 6).unwrap();
    assert_eq!(d.text(), "1567");
    d.delete(2, 2).unwrap();
    assert_eq!(d.text(), "1567");
    d.delete(0, 4).unwrap();
    assert_eq!(d.text(), "");
}

#[test]
fn should_reject_invalid_delete_ranges() {
    let d = doc!("abc");

    assert_eq!(d.delete(2, 1), Err(WordError::OutOfRange));
    assert_eq!(d.delete(1, 4), Err(WordError::OutOfRange));
    assert_eq!(d.delete(4, 4), Err(WordError::OutOfRange));
    assert_eq!(d.text(), "abc");
}

#[test]
fn should_change_only_the_fields_that_are_given() {
    let d = doc!("abcdef");
    d.apply_format(1, 4, &[("font", "Arial"), ("size", "14")]).unwrap();
    d.apply_format(2, 3, &[("italic", "true")]).unwrap();

    assert_eq!(d.format_at(0), Ok(DEF.to_string()));
    assert_eq!(d.format_at(1), Ok("Arial|14|-|-|-|black".to_string()));
    assert_eq!(d.format_at(2), Ok("Arial|14|-|I|-|black".to_string()));
    assert_eq!(d.format_at(3), Ok("Arial|14|-|-|-|black".to_string()));
    assert_eq!(d.format_at(4), Ok(DEF.to_string()));
}

#[test]
fn should_show_every_flag_in_the_format_string() {
    let d = doc!("x");
    d.apply_format(
        0,
        1,
        &[
            ("font", "Times New Roman"),
            ("size", "72"),
            ("bold", "true"),
            ("italic", "true"),
            ("underline", "true"),
            ("color", "dark red"),
        ],
    )
    .unwrap();

    assert_eq!(d.format_at(0), Ok("Times New Roman|72|B|I|U|dark red".to_string()));

    d.apply_format(0, 1, &[("bold", "false"), ("underline", "false")]).unwrap();
    assert_eq!(d.format_at(0), Ok("Times New Roman|72|-|I|-|dark red".to_string()));
}

#[test]
fn should_reject_an_invalid_format_change_without_applying_any_part_of_it() {
    let d = doc!("abcdef");

    assert_eq!(d.apply_format(0, 3, &[("bold", "true"), ("size", "0")]), Err(WordError::InvalidArgument));
    assert_eq!(d.apply_format(0, 3, &[("size", "1001")]), Err(WordError::InvalidArgument));
    assert_eq!(d.apply_format(0, 3, &[("size", "big")]), Err(WordError::InvalidArgument));
    assert_eq!(d.apply_format(0, 3, &[("size", "-4")]), Err(WordError::InvalidArgument));
    assert_eq!(d.apply_format(0, 3, &[("bold", "yes")]), Err(WordError::InvalidArgument));
    assert_eq!(d.apply_format(0, 3, &[("font", " ")]), Err(WordError::InvalidArgument));
    assert_eq!(d.apply_format(0, 3, &[("color", "a|b")]), Err(WordError::InvalidArgument));
    assert_eq!(d.apply_format(0, 3, &[("strike", "true")]), Err(WordError::InvalidArgument));
    assert_eq!(d.apply_format(0, 3, &[("bold", "true"), ("bold", "false")]), Err(WordError::InvalidArgument));
    assert_eq!(d.apply_format(0, 7, &[("bold", "true")]), Err(WordError::OutOfRange));
    assert_eq!(d.apply_format(4, 2, &[("bold", "true")]), Err(WordError::OutOfRange));

    assert_eq!(d.runs(), vec![format!("0-6:{DEF}")]);
}

#[test]
fn should_treat_an_empty_change_or_an_empty_range_as_a_no_op() {
    let d = doc!("abc");
    d.apply_format(0, 3, &[]).unwrap();
    d.apply_format(2, 2, &[("bold", "true")]).unwrap();
    d.apply_format(3, 3, &[("bold", "true")]).unwrap();

    assert_eq!(d.runs(), vec![format!("0-3:{DEF}")]);
}

#[test]
fn should_reject_format_at_outside_the_text() {
    let d = doc!("abc");

    assert_eq!(d.format_at(3), Err(WordError::OutOfRange));
    assert_eq!(doc!().format_at(0), Err(WordError::OutOfRange));
}

#[test]
fn should_report_maximal_runs_with_exclusive_ends() {
    let d = doc!("0123456789");
    d.apply_format(2, 5, &[("bold", "true")]).unwrap();
    d.apply_format(7, 9, &[("color", "red")]).unwrap();

    assert_eq!(
        d.runs(),
        vec![
            format!("0-2:{DEF}"),
            "2-5:Calibri|11|B|-|-|black".to_string(),
            format!("5-7:{DEF}"),
            "7-9:Calibri|11|-|-|-|red".to_string(),
            format!("9-10:{DEF}"),
        ]
    );
}

#[test]
fn should_merge_adjacent_runs_that_become_equal() {
    let d = doc!("0123456789");
    d.apply_format(0, 5, &[("bold", "true")]).unwrap();
    d.apply_format(5, 10, &[("bold", "true")]).unwrap();
    assert_eq!(d.runs(), vec!["0-10:Calibri|11|B|-|-|black".to_string()]);

    d.apply_format(3, 6, &[("bold", "false")]).unwrap();
    assert_eq!(d.runs().len(), 3);

    d.apply_format(0, 10, &[("bold", "false")]).unwrap();
    assert_eq!(d.runs(), vec![format!("0-10:{DEF}")]);
}

#[test]
fn should_merge_runs_across_a_deleted_middle() {
    let d = doc!("aaXbb");
    d.apply_format(2, 3, &[("underline", "true")]).unwrap();
    assert_eq!(d.runs().len(), 3);

    d.delete(2, 3).unwrap();

    assert_eq!(d.text(), "aabb");
    assert_eq!(d.runs(), vec![format!("0-4:{DEF}")]);
}

#[test]
fn should_keep_a_single_run_when_inserting_inside_a_formatted_run() {
    let d = doc!("abcdef");
    d.apply_format(2, 5, &[("bold", "true")]).unwrap();

    d.insert(3, "ZZ").unwrap();

    assert_eq!(
        d.runs(),
        vec![format!("0-2:{DEF}"), "2-7:Calibri|11|B|-|-|black".to_string(), format!("7-8:{DEF}")]
    );
}

#[test]
fn should_find_overlapping_matches_in_ascending_order() {
    let d = doc!("aaaa abab");

    assert_eq!(d.find("aa", true), Ok(vec![0, 1, 2]));
    assert_eq!(d.find("aba", true), Ok(vec![5]));
    assert_eq!(d.find("ab", true), Ok(vec![5, 7]));
    assert_eq!(d.find("zz", true), Ok(vec![]));
    assert_eq!(d.find("aaaa abab", true), Ok(vec![0]));
    assert_eq!(d.find("aaaa abab!", true), Ok(vec![]));
}

#[test]
fn should_find_ignoring_case_only_when_asked() {
    let d = doc!("Cat cAT CAT cat");

    assert_eq!(d.find("cat", true), Ok(vec![12]));
    assert_eq!(d.find("cat", false), Ok(vec![0, 4, 8, 12]));
    assert_eq!(d.find("CAT", false), Ok(vec![0, 4, 8, 12]));
}

#[test]
fn should_find_across_format_boundaries() {
    let d = doc!("hello world");
    d.apply_format(3, 8, &[("bold", "true")]).unwrap();

    assert_eq!(d.find("lo wo", true), Ok(vec![3]));
}

#[test]
fn should_reject_an_empty_pattern() {
    let d = doc!("abc");

    assert_eq!(d.find("", true), Err(WordError::InvalidArgument));
    assert_eq!(d.replace_all("", "x", true), Err(WordError::InvalidArgument));
    assert_eq!(d.text(), "abc");
}

#[test]
fn should_replace_all_non_overlapping_matches_and_return_the_count() {
    let d = doc!("the cat and the hat");

    assert_eq!(d.replace_all("the", "a", true), Ok(2));
    assert_eq!(d.text(), "a cat and a hat");
    assert_eq!(d.replace_all("zzz", "q", true), Ok(0));
    assert_eq!(d.text(), "a cat and a hat");

    let e = doc!("aaaa");
    assert_eq!(e.replace_all("aa", "b", true), Ok(2));
    assert_eq!(e.text(), "bb");

    let g = doc!("aaa");
    assert_eq!(g.replace_all("aa", "b", true), Ok(1));
    assert_eq!(g.text(), "ba");
}

#[test]
fn should_not_scan_the_replacement_text_again() {
    let d = doc!("ab ab");

    assert_eq!(d.replace_all("ab", "abab", true), Ok(2));
    assert_eq!(d.text(), "abab abab");
}

#[test]
fn should_replace_ignoring_case_when_asked() {
    let d = doc!("Foo foo FOO");

    assert_eq!(d.replace_all("foo", "x", true), Ok(1));
    assert_eq!(d.text(), "Foo x FOO");
    assert_eq!(d.replace_all("foo", "y", false), Ok(2));
    assert_eq!(d.text(), "y x y");
}

#[test]
fn should_delete_matches_when_the_replacement_is_empty() {
    let d = doc!("a-b-c");

    assert_eq!(d.replace_all("-", "", true), Ok(2));
    assert_eq!(d.text(), "abc");
    assert_eq!(d.runs(), vec![format!("0-3:{DEF}")]);
}

#[test]
fn should_give_replacement_text_the_format_of_the_first_character_of_each_match() {
    let d = doc!("one two one");
    d.apply_format(0, 1, &[("bold", "true")]).unwrap();
    d.apply_format(1, 3, &[("italic", "true")]).unwrap();
    d.apply_format(8, 11, &[("color", "red")]).unwrap();

    assert_eq!(d.replace_all("one", "1234", true), Ok(2));

    assert_eq!(d.text(), "1234 two 1234");
    assert_eq!(
        d.runs(),
        vec![
            "0-4:Calibri|11|B|-|-|black".to_string(),
            format!("4-9:{DEF}"),
            "9-13:Calibri|11|-|-|-|red".to_string(),
        ]
    );
}

#[test]
fn should_count_words_as_maximal_non_whitespace_sequences() {
    assert_eq!(doc!().word_count(), 0);
    assert_eq!(doc!("   \n\t ").word_count(), 0);
    assert_eq!(doc!("hello").word_count(), 1);
    assert_eq!(doc!("  hello   world  ").word_count(), 2);
    assert_eq!(doc!("a\nb\tc\r\nd").word_count(), 4);
    assert_eq!(doc!("don't stop-me now.").word_count(), 3);
}

#[test]
fn should_count_lines_as_newlines_plus_one() {
    assert_eq!(doc!().line_count(), 1);
    assert_eq!(doc!("abc").line_count(), 1);
    assert_eq!(doc!("a\nb\nc").line_count(), 3);
    assert_eq!(doc!("a\n").line_count(), 2);
    assert_eq!(doc!("\n\n").line_count(), 3);

    let d = doc!("a\nb\nc");
    d.delete(1, 4).unwrap();
    assert_eq!(d.text(), "ac");
    assert_eq!(d.line_count(), 1);
}

#[test]
fn should_set_and_read_paragraph_alignment() {
    let d = doc!("one\ntwo\nthree");

    assert_eq!(d.alignment(0), Ok("LEFT".to_string()));
    d.set_alignment(1, "CENTER").unwrap();
    d.set_alignment(2, "JUSTIFY").unwrap();
    d.set_alignment(0, "RIGHT").unwrap();

    assert_eq!(d.alignment(0), Ok("RIGHT".to_string()));
    assert_eq!(d.alignment(1), Ok("CENTER".to_string()));
    assert_eq!(d.alignment(2), Ok("JUSTIFY".to_string()));
}

#[test]
fn should_reject_invalid_alignment_requests() {
    let d = doc!("one\ntwo");

    assert_eq!(d.set_alignment(2, "LEFT"), Err(WordError::OutOfRange));
    assert_eq!(d.set_alignment(0, "left"), Err(WordError::InvalidArgument));
    assert_eq!(d.set_alignment(0, "MIDDLE"), Err(WordError::InvalidArgument));
    assert_eq!(d.alignment(2), Err(WordError::OutOfRange));
    assert_eq!(d.alignment(0), Ok("LEFT".to_string()));
}

#[test]
fn should_give_new_lines_the_alignment_of_the_line_they_were_created_in() {
    let d = doc!("one\ntwo");
    d.set_alignment(0, "CENTER").unwrap();
    d.set_alignment(1, "RIGHT").unwrap();

    d.insert(3, "\nnew").unwrap();

    assert_eq!(d.text(), "one\nnew\ntwo");
    assert_eq!(d.alignment(0), Ok("CENTER".to_string()));
    assert_eq!(d.alignment(1), Ok("CENTER".to_string()));
    assert_eq!(d.alignment(2), Ok("RIGHT".to_string()));

    d.insert(9, "x\ny\nz").unwrap();
    assert_eq!(d.line_count(), 5);
    assert_eq!(d.alignment(2), Ok("RIGHT".to_string()));
    assert_eq!(d.alignment(3), Ok("RIGHT".to_string()));
    assert_eq!(d.alignment(4), Ok("RIGHT".to_string()));
}

#[test]
fn should_keep_the_alignment_of_the_first_line_when_lines_are_merged_by_deleting() {
    let d = doc!("aa\nbb\ncc\ndd");
    d.set_alignment(0, "LEFT").unwrap();
    d.set_alignment(1, "CENTER").unwrap();
    d.set_alignment(2, "RIGHT").unwrap();
    d.set_alignment(3, "JUSTIFY").unwrap();

    d.delete(4, 7).unwrap();

    assert_eq!(d.text(), "aa\nbc\ndd");
    assert_eq!(d.line_count(), 3);
    assert_eq!(d.alignment(0), Ok("LEFT".to_string()));
    assert_eq!(d.alignment(1), Ok("CENTER".to_string()));
    assert_eq!(d.alignment(2), Ok("JUSTIFY".to_string()));
}

#[test]
fn should_count_distinct_formats_in_use() {
    let d = doc!("abcdef");
    assert_eq!(d.distinct_format_count(), 1);

    d.apply_format(0, 2, &[("bold", "true")]).unwrap();
    d.apply_format(4, 6, &[("bold", "true")]).unwrap();
    assert_eq!(d.distinct_format_count(), 2);

    d.apply_format(1, 4, &[("color", "red")]).unwrap();
    assert_eq!(d.distinct_format_count(), 3);

    d.apply_format(0, 6, &[("bold", "false"), ("color", "black")]).unwrap();
    assert_eq!(d.distinct_format_count(), 1);

    d.delete(0, 6).unwrap();
    assert_eq!(d.distinct_format_count(), 0);
}

#[test]
fn should_lose_no_characters_when_threads_insert_at_the_end() {
    let d = doc!();
    let threads = 8usize;
    let each = 1_000usize;

    run_concurrently(threads, |t| {
        let ch = ((b'a' + t as u8) as char).to_string();
        for _ in 0..each {
            d.insert(d.length(), &ch).unwrap();
        }
    });

    assert_eq!(d.length(), threads * each);
    let text = d.text();
    for t in 0..threads {
        let c = (b'a' + t as u8) as char;
        assert_eq!(text.chars().filter(|x| *x == c).count(), each);
    }
    assert_eq!(d.runs(), vec![format!("0-{}:{DEF}", threads * each)]);
}

#[test]
fn should_stay_consistent_while_threads_format_and_read() {
    let d = doc!(&"x".repeat(1_000));

    run_concurrently(6, |t| {
        for i in 0..300usize {
            if t < 3 {
                d.apply_format(i, i + 100, &[("size", &(10 + t).to_string())]).unwrap();
            } else {
                let runs = d.runs();
                assert!(runs[0].starts_with("0-"));
                assert!(runs[runs.len() - 1].contains("-1000:"));
                let n = d.distinct_format_count();
                assert!((1..=4).contains(&n));
            }
        }
    });

    assert_eq!(d.text(), "x".repeat(1_000));
}

#[test]
fn should_handle_a_million_characters_in_a_few_runs() {
    let start = Instant::now();
    let d = doc!();
    d.insert(0, &"a".repeat(1_000_000)).unwrap();
    for i in 0..10usize {
        d.apply_format(i * 100_000 + 10_000, i * 100_000 + 20_000, &[("size", &(12 + i).to_string())])
            .unwrap();
    }

    assert_eq!(d.length(), 1_000_000);
    assert_eq!(d.runs().len(), 21);
    assert_eq!(d.distinct_format_count(), 11);
    assert_eq!(d.format_at(15_000), Ok("Calibri|12|-|-|-|black".to_string()));
    assert_eq!(d.format_at(999_999), Ok(DEF.to_string()));

    for i in 0..2_000usize {
        d.insert(500_000, "b").unwrap();
        d.format_at(i * 400).unwrap();
    }
    d.apply_format(0, 1_000_000, &[("size", "11")]).unwrap();
    assert_eq!(d.runs(), vec![format!("0-1002000:{DEF}")]);
    assert_eq!(d.find("b", true).unwrap().len(), 2_000);
    assert_eq!(d.word_count(), 1);

    assert!(start.elapsed().as_millis() < 20_000);
}
