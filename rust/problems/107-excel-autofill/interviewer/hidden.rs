#![allow(unused_imports, clippy::needless_range_loop)]
use p107_excel_autofill::{contract::*, create};
use playground_support::{run_concurrently, sut};

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn should_treat_letter_labels_as_bijective_base_twenty_six() {
    let s = sut!(create());
    assert_eq!(s.autofill(&["Z", "AA"], 3), Ok(strs(&["AB", "AC", "AD"])));
    assert_eq!(
        s.autofill(&["ZY", "ZZ"], 3),
        Ok(strs(&["AAA", "AAB", "AAC"]))
    );
    assert_eq!(s.autofill(&["zz"], 2), Ok(strs(&["aaa", "aab"])));
    assert_eq!(s.autofill(&["Y", "AA"], 3), Ok(strs(&["AC", "AE", "AG"])));
}

#[test]
fn should_shrink_labels_when_a_decreasing_letter_series_crosses_a_length_boundary() {
    let s = sut!(create());
    assert_eq!(
        s.autofill(&["AB", "AA"], 4),
        Ok(strs(&["Z", "Y", "X", "W"]))
    );
    assert_eq!(s.autofill(&["BA", "AZ"], 2), Ok(strs(&["AY", "AX"])));
}

#[test]
fn should_stop_immediately_when_the_first_step_leaves_the_alphabet() {
    let s = sut!(create());
    assert_eq!(s.autofill(&["C", "A"], 3), Ok(vec![]));
    assert_eq!(s.autofill(&["e", "c"], 5), Ok(strs(&["a"])));
}

#[test]
fn should_repeat_a_letter_when_the_step_is_zero() {
    let s = sut!(create());
    assert_eq!(s.autofill(&["B", "B", "B"], 2), Ok(strs(&["B", "B"])));
}

#[test]
fn should_keep_decimal_arithmetic_exact_across_scales_and_signs() {
    let s = sut!(create());
    assert_eq!(
        s.autofill(&["0.1", "0.2", "0.3"], 2),
        Ok(strs(&["0.4", "0.5"]))
    );
    assert_eq!(s.autofill(&["1.10", "1.20"], 2), Ok(strs(&["1.3", "1.4"])));
    assert_eq!(s.autofill(&["1", "1.5", "2.0"], 2), Ok(strs(&["2.5", "3"])));
    assert_eq!(
        s.autofill(&["0.5", "0.25"], 3),
        Ok(strs(&["0", "-0.25", "-0.5"]))
    );
    assert_eq!(s.autofill(&["-0.5", "0.5"], 2), Ok(strs(&["1.5", "2.5"])));
    assert_eq!(
        s.autofill(&["0.001", "0.002"], 2),
        Ok(strs(&["0.003", "0.004"]))
    );
}

#[test]
fn should_never_emit_negative_zero() {
    let s = sut!(create());
    assert_eq!(s.autofill(&["-2", "-1"], 3), Ok(strs(&["0", "1", "2"])));
    assert_eq!(
        s.autofill(&["-0.5", "-0.25"], 3),
        Ok(strs(&["0", "0.25", "0.5"]))
    );
    assert_eq!(s.autofill(&["-0"], 2), Ok(strs(&["1", "2"])));
}

#[test]
fn should_accept_leading_zeros_but_not_preserve_padding() {
    let s = sut!(create());
    assert_eq!(s.autofill(&["007", "008"], 3), Ok(strs(&["9", "10", "11"])));
    assert_eq!(s.autofill(&["00.50", "01.00"], 2), Ok(strs(&["1.5", "2"])));
}

#[test]
fn should_accept_fifteen_digits_and_reject_sixteen() {
    let s = sut!(create());
    assert_eq!(
        s.autofill(&["999999999999998", "999999999999999"], 2),
        Ok(strs(&["1000000000000000", "1000000000000001"]))
    );
    assert_eq!(
        s.autofill(&["0.12345678901234"], 1),
        Ok(strs(&["1.12345678901234"]))
    );
    assert_eq!(
        s.autofill(&["1234567890123456"], 1),
        Err(AutofillError::UnrecognizedSeries)
    );
    assert_eq!(
        s.autofill(&["0.1234567890123456"], 1),
        Err(AutofillError::UnrecognizedSeries)
    );
}

#[test]
fn should_accept_ten_letters_and_reject_eleven() {
    let s = sut!(create());
    assert_eq!(s.autofill(&["AAAAAAAAAA"], 1), Ok(strs(&["AAAAAAAAAB"])));
    assert_eq!(
        s.autofill(&["AAAAAAAAAAA"], 1),
        Err(AutofillError::UnrecognizedSeries)
    );
}

#[test]
fn should_reject_any_bad_seed_even_when_it_is_last_or_the_count_is_zero() {
    let s = sut!(create());
    let bad = Err(AutofillError::UnrecognizedSeries);
    assert_eq!(s.autofill(&["1", "2", "3", "x"], 0), bad);
    assert_eq!(s.autofill(&["1", "2", "3", "5"], 0), bad);
    assert_eq!(s.autofill(&["1", "2", "2"], 1), bad);
    assert_eq!(s.autofill(&["a", "B"], 0), bad);
}

#[test]
fn should_check_every_pair_not_just_the_first_two_seeds() {
    let s = sut!(create());
    let bad = Err(AutofillError::UnrecognizedSeries);
    assert_eq!(s.autofill(&["2", "4", "6", "9"], 1), bad);
    assert_eq!(s.autofill(&["A", "C", "E", "H"], 1), bad);
}

#[test]
fn should_handle_large_steps_and_large_counts() {
    let s = sut!(create());
    assert_eq!(
        s.autofill(&["1000000000000", "2000000000000"], 2),
        Ok(strs(&["3000000000000", "4000000000000"]))
    );
    let big = s.autofill(&["1", "2"], 100_000).unwrap();
    assert_eq!(big.len(), 100_000);
    assert_eq!(big[99_999], "100002");
    let letters = s.autofill(&["A"], 703).unwrap();
    assert_eq!(letters[25], "AA");
    assert_eq!(letters[701], "AAA");
}

#[test]
fn should_stay_correct_when_many_threads_mix_kinds() {
    let s = sut!(create());
    run_concurrently(8, |_| {
        for _ in 0..5_000 {
            assert_eq!(s.autofill(&["a", "c"], 2), Ok(strs(&["e", "g"])));
            assert_eq!(s.autofill(&["0.1", "0.2"], 1), Ok(strs(&["0.3"])));
            assert_eq!(
                s.autofill(&["1", "x"], 1),
                Err(AutofillError::UnrecognizedSeries)
            );
        }
    });
}
