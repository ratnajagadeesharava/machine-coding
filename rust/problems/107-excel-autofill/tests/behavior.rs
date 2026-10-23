#![allow(unused_imports, clippy::needless_range_loop)]
use p107_excel_autofill::{contract::*, create};
use playground_support::{run_concurrently, sut};

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn should_continue_an_increasing_integer_series() {
    let s = sut!(create());
    assert_eq!(s.autofill(&["1", "3"], 4), Ok(strs(&["5", "7", "9", "11"])));
}

#[test]
fn should_continue_a_decreasing_series_through_zero_into_negatives() {
    let s = sut!(create());
    assert_eq!(
        s.autofill(&["5", "3"], 4),
        Ok(strs(&["1", "-1", "-3", "-5"]))
    );
}

#[test]
fn should_continue_series_whose_seeds_are_negative() {
    let s = sut!(create());
    assert_eq!(s.autofill(&["-10", "-7"], 3), Ok(strs(&["-4", "-1", "2"])));
}

#[test]
fn should_continue_using_the_step_shared_by_all_seeds() {
    let s = sut!(create());
    assert_eq!(s.autofill(&["10", "20", "30"], 2), Ok(strs(&["40", "50"])));
}

#[test]
fn should_repeat_a_constant_series() {
    let s = sut!(create());
    assert_eq!(s.autofill(&["4", "4"], 3), Ok(strs(&["4", "4", "4"])));
}

#[test]
fn should_add_decimals_exactly() {
    let s = sut!(create());
    assert_eq!(
        s.autofill(&["0.1", "0.2"], 3),
        Ok(strs(&["0.3", "0.4", "0.5"]))
    );
}

#[test]
fn should_drop_trailing_zeros_and_never_use_scientific_notation() {
    let s = sut!(create());
    assert_eq!(
        s.autofill(&["0.5", "1.0"], 3),
        Ok(strs(&["1.5", "2", "2.5"]))
    );
    assert_eq!(
        s.autofill(&["1000000", "2000000"], 2),
        Ok(strs(&["3000000", "4000000"]))
    );
    assert_eq!(
        s.autofill(&["0.0000001", "0.0000002"], 2),
        Ok(strs(&["0.0000003", "0.0000004"]))
    );
    assert_eq!(s.autofill(&["1", "0.5"], 3), Ok(strs(&["0", "-0.5", "-1"])));
}

#[test]
fn should_step_by_one_for_a_single_numeric_seed() {
    let s = sut!(create());
    assert_eq!(s.autofill(&["7"], 3), Ok(strs(&["8", "9", "10"])));
    assert_eq!(s.autofill(&["-1"], 3), Ok(strs(&["0", "1", "2"])));
    assert_eq!(s.autofill(&["1.5"], 2), Ok(strs(&["2.5", "3.5"])));
}

#[test]
fn should_continue_an_uppercase_letter_series() {
    let s = sut!(create());
    assert_eq!(s.autofill(&["A", "B"], 3), Ok(strs(&["C", "D", "E"])));
}

#[test]
fn should_preserve_lowercase() {
    let s = sut!(create());
    assert_eq!(s.autofill(&["a", "b"], 2), Ok(strs(&["c", "d"])));
}

#[test]
fn should_wrap_past_z_into_two_letter_labels() {
    let s = sut!(create());
    assert_eq!(
        s.autofill(&["X", "Y"], 4),
        Ok(strs(&["Z", "AA", "AB", "AC"]))
    );
    assert_eq!(s.autofill(&["Z"], 2), Ok(strs(&["AA", "AB"])));
    assert_eq!(s.autofill(&["z"], 2), Ok(strs(&["aa", "ab"])));
    assert_eq!(s.autofill(&["AZ"], 2), Ok(strs(&["BA", "BB"])));
}

#[test]
fn should_continue_a_letter_series_with_a_step() {
    let s = sut!(create());
    assert_eq!(s.autofill(&["A", "C"], 3), Ok(strs(&["E", "G", "I"])));
}

#[test]
fn should_step_by_one_for_a_single_letter_seed() {
    let s = sut!(create());
    assert_eq!(s.autofill(&["M"], 2), Ok(strs(&["N", "O"])));
}

#[test]
fn should_stop_a_decreasing_letter_series_before_a() {
    let s = sut!(create());
    assert_eq!(s.autofill(&["D", "C"], 5), Ok(strs(&["B", "A"])));
    assert_eq!(s.autofill(&["c", "b"], 1), Ok(strs(&["a"])));
    assert_eq!(s.autofill(&["B", "A"], 3), Ok(vec![]));
}

#[test]
fn should_return_exactly_count_values_and_nothing_for_zero() {
    let s = sut!(create());
    assert_eq!(s.autofill(&["1", "2"], 0), Ok(vec![]));
    assert_eq!(s.autofill(&["A", "B"], 0), Ok(vec![]));
    let many = s.autofill(&["1", "2"], 1000).unwrap();
    assert_eq!(many.len(), 1000);
    assert_eq!(many[0], "3");
    assert_eq!(many[999], "1002");
}

#[test]
fn should_reject_mixed_kinds_of_seeds() {
    let s = sut!(create());
    assert_eq!(
        s.autofill(&["1", "A"], 2),
        Err(AutofillError::UnrecognizedSeries)
    );
    assert_eq!(
        s.autofill(&["A", "1"], 2),
        Err(AutofillError::UnrecognizedSeries)
    );
}

#[test]
fn should_reject_seeds_that_are_not_an_arithmetic_progression() {
    let s = sut!(create());
    assert_eq!(
        s.autofill(&["1", "2", "4"], 2),
        Err(AutofillError::UnrecognizedSeries)
    );
    assert_eq!(
        s.autofill(&["A", "B", "D"], 2),
        Err(AutofillError::UnrecognizedSeries)
    );
}

#[test]
fn should_reject_mixed_case_letters() {
    let s = sut!(create());
    assert_eq!(
        s.autofill(&["A", "b"], 2),
        Err(AutofillError::UnrecognizedSeries)
    );
    assert_eq!(
        s.autofill(&["Ab"], 2),
        Err(AutofillError::UnrecognizedSeries)
    );
}

#[test]
fn should_reject_malformed_numbers() {
    let s = sut!(create());
    for bad in [
        "1e3", "+1", ".5", "1.", " 1", "1,000", "0x10", "--1", "1.2.3", "-",
    ] {
        assert_eq!(
            s.autofill(&[bad], 1),
            Err(AutofillError::UnrecognizedSeries),
            "{bad}"
        );
    }
}

#[test]
fn should_reject_text_that_is_neither_number_nor_letters() {
    let s = sut!(create());
    for bad in ["", " ", "Jan", "A1", "?", "A B", "é"] {
        assert_eq!(
            s.autofill(&[bad], 1),
            Err(AutofillError::UnrecognizedSeries),
            "{bad}"
        );
    }
}

#[test]
fn should_reject_an_empty_seed_list() {
    let s = sut!(create());
    assert_eq!(s.autofill(&[], 1), Err(AutofillError::InvalidArgument));
    assert_eq!(s.autofill(&[], 0), Err(AutofillError::InvalidArgument));
}

#[test]
fn should_validate_seeds_even_when_count_is_zero() {
    let s = sut!(create());
    assert_eq!(
        s.autofill(&["1", "A"], 0),
        Err(AutofillError::UnrecognizedSeries)
    );
}

#[test]
fn should_give_the_same_answer_when_called_from_many_threads() {
    let s = sut!(create());
    run_concurrently(16, |t| {
        for _ in 0..500 {
            let a = t.to_string();
            let b = (t + 2).to_string();
            let expected: Vec<String> = (1..=3).map(|k| (t + 2 + 2 * k).to_string()).collect();
            assert_eq!(s.autofill(&[&a, &b], 3), Ok(expected));
            assert_eq!(s.autofill(&["Y", "Z"], 2), Ok(strs(&["AA", "AB"])));
        }
    });
}
