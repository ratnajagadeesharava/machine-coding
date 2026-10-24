#![allow(unused_imports, clippy::needless_range_loop)]
use p143_excel_formulas::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::sync::Arc;
use std::time::Instant;

type Sheet = Arc<dyn FormulaSheet>;

fn put(s: &Sheet, cells: &[(&str, &str)]) {
    for (c, i) in cells {
        s.set(c, i).unwrap();
    }
}

fn get(s: &Sheet, c: &str) -> String {
    s.get(c).unwrap()
}

fn cell_name(i: usize) -> String {
    format!("{}{}", (b'A' + (i % 26) as u8) as char, i / 26 + 1)
}

fn list(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

/// Runs the body on a thread with the default stack size.
fn on_default_stack(body: fn()) {
    std::thread::spawn(body)
        .join()
        .expect("default-stack thread failed");
}

#[test]
fn should_store_numbers_text_and_empty_cells() {
    let s = sut!(create());
    put(
        &s,
        &[
            ("A1", "5"),
            ("B1", "hello"),
            ("C1", "3.50"),
            ("D1", "-0"),
            ("E1", "+7"),
            ("F1", "007"),
            ("G1", "-2.25"),
        ],
    );

    assert_eq!(get(&s, "A1"), "5");
    assert_eq!(get(&s, "B1"), "hello");
    assert_eq!(get(&s, "C1"), "3.5");
    assert_eq!(get(&s, "D1"), "0");
    assert_eq!(get(&s, "E1"), "7");
    assert_eq!(get(&s, "F1"), "7");
    assert_eq!(get(&s, "G1"), "-2.25");
    assert_eq!(get(&s, "H1"), "");
    assert_eq!(s.formula_of("A1").unwrap(), "");
}

#[test]
fn should_keep_input_that_is_not_an_exact_number_literal_as_text() {
    let s = sut!(create());
    let texts = [
        "1e3", "5.", ".5", " 5", "5 ", "--5", "1,000", "1.2.3", "+", "-", "12abc", "A1+1", "'5",
    ];
    for (i, t) in texts.iter().enumerate() {
        s.set(&cell_name(i), t).unwrap();
    }
    for (i, t) in texts.iter().enumerate() {
        assert_eq!(get(&s, &cell_name(i)), *t, "{t}");
    }
    s.set("A2", "= not a formula").unwrap();
    assert_eq!(get(&s, "A2"), "#ERR!");
    s.set("A3", " =1+1").unwrap();
    assert_eq!(get(&s, "A3"), " =1+1");
    assert_eq!(s.formula_of("A3").unwrap(), "");
}

#[test]
fn should_normalise_addresses_to_upper_case() {
    let s = sut!(create());
    s.set("b2", "7").unwrap();
    s.set("c3", "=b2*2").unwrap();

    assert_eq!(get(&s, "B2"), "7");
    assert_eq!(get(&s, "c3"), "14");
    assert_eq!(s.used_cells(), list(&["B2", "C3"]));
    assert_eq!(s.dependencies("c3").unwrap(), list(&["B2"]));
    assert_eq!(s.dependents("b2").unwrap(), list(&["C3"]));
}

#[test]
fn should_reject_invalid_addresses() {
    let s = sut!(create());
    s.set("A1", "1").unwrap();
    let bad = [
        "", " ", "A0", "A101", "AA1", "1A", "A", "A 1", "A01", " A1", "A1 ", "@1", "A-1", "A1.5",
        "Z999",
    ];
    for a in bad {
        let e = SheetError::InvalidAddress;
        assert_eq!(s.set(a, "1"), Err(SheetError::InvalidAddress), "set {a}");
        assert_eq!(s.get(a), Err(SheetError::InvalidAddress), "get {a}");
        assert_eq!(
            s.formula_of(a),
            Err(SheetError::InvalidAddress),
            "formula_of {a}"
        );
        assert_eq!(
            s.dependencies(a),
            Err(SheetError::InvalidAddress),
            "dependencies {a}"
        );
        assert_eq!(s.dependents(a), Err(e), "dependents {a}");
    }
    assert_eq!(get(&s, "A1"), "1");
    assert_eq!(s.used_cells(), list(&["A1"]));
    assert_eq!(get(&s, "Z100"), "");
    assert_eq!(get(&s, "a1"), "1");
}

#[test]
fn should_clear_a_cell_with_an_empty_string() {
    let s = sut!(create());
    put(&s, &[("A1", "5"), ("B1", "=A1+1")]);
    assert_eq!(get(&s, "B1"), "6");

    s.set("A1", "").unwrap();
    assert_eq!(get(&s, "A1"), "");
    assert_eq!(get(&s, "B1"), "1");
    assert_eq!(s.used_cells(), list(&["B1"]));

    s.set("B1", "").unwrap();
    assert_eq!(get(&s, "B1"), "");
    assert_eq!(s.formula_of("B1").unwrap(), "");
    assert!(s.dependencies("B1").unwrap().is_empty());
    assert!(s.dependents("A1").unwrap().is_empty());
    assert!(s.used_cells().is_empty());
}

#[test]
fn should_render_numbers_without_trailing_zeros_and_at_most_ten_decimals() {
    let s = sut!(create());
    let cases = [
        ("=1/3", "0.3333333333"),
        ("=2/3", "0.6666666667"),
        ("=1/8", "0.125"),
        ("=10/4", "2.5"),
        ("=0.1+0.2", "0.3"),
        ("=1/3*3", "1"),
        ("=0-0", "0"),
        ("=-0", "0"),
        ("=0*-1", "0"),
        ("=0.00000000004", "0"),
        ("=0.00000000006", "0.0000000001"),
        ("=-0.00000000004", "0"),
        ("=100000*100000", "10000000000"),
        ("=123456789012*10", "1234567890120"),
        ("=-7/2", "-3.5"),
        ("=2.50", "2.5"),
        ("=1.0", "1"),
    ];
    for (f, want) in cases {
        s.set("A1", f).unwrap();
        assert_eq!(get(&s, "A1"), want, "{f}");
    }
}

#[test]
fn should_evaluate_arithmetic_with_precedence_and_left_associativity() {
    let s = sut!(create());
    let cases = [
        ("=1+2*3", "7"),
        ("=(1+2)*3", "9"),
        ("=10-4-3", "3"),
        ("=100/10/5", "2"),
        ("=2*3+4*5", "26"),
        ("=-3+5", "2"),
        ("=--3", "3"),
        ("=2*-3", "-6"),
        ("=2--3", "5"),
        ("=-(1+2)*2", "-6"),
        ("=((((7))))", "7"),
        ("= 1 +  2 ", "3"),
        ("=\t8 / 4\t", "2"),
        ("=1.5*4", "6"),
        ("=2-3", "-1"),
        ("=8/2*3", "12"),
        ("=2*(3+4)-5", "9"),
    ];
    for (f, want) in cases {
        s.set("A1", f).unwrap();
        assert_eq!(get(&s, "A1"), want, "{f}");
    }
}

#[test]
fn should_read_cells_and_treat_empty_cells_as_zero() {
    let s = sut!(create());
    put(
        &s,
        &[
            ("A1", "5"),
            ("A2", "=A1*2"),
            ("A3", "=A2+B9"),
            ("A4", "=B9"),
            ("A5", "=B9*7"),
        ],
    );

    assert_eq!(get(&s, "A2"), "10");
    assert_eq!(get(&s, "A3"), "10");
    assert_eq!(get(&s, "A4"), "0");
    assert_eq!(get(&s, "A5"), "0");
    assert_eq!(s.used_cells(), list(&["A1", "A2", "A3", "A4", "A5"]));
}

#[test]
fn should_sum_a_rectangle_in_any_corner_order() {
    let s = sut!(create());
    put(
        &s,
        &[
            ("A1", "1"),
            ("B1", "2"),
            ("A2", "3"),
            ("B2", "4"),
            ("A3", "5"),
            ("B3", "6"),
            ("C1", "100"),
        ],
    );
    for r in ["A1:B3", "B3:A1", "A3:B1", "B1:A3"] {
        s.set("D1", &format!("=SUM({r})")).unwrap();
        assert_eq!(get(&s, "D1"), "21", "{r}");
    }
    s.set("D1", "=SUM(A2:B2)").unwrap();
    assert_eq!(get(&s, "D1"), "7");
    s.set("D1", "=SUM(B2:B2)").unwrap();
    assert_eq!(get(&s, "D1"), "4");
    s.set("D1", "=SUM(A1:A3)+SUM(B1:B3)*2").unwrap();
    assert_eq!(get(&s, "D1"), "33");
    s.set("D1", "=sum(a1:b2)").unwrap();
    assert_eq!(get(&s, "D1"), "10");
}

#[test]
fn should_sum_a_list_of_ranges_cells_and_numbers() {
    let s = sut!(create());
    put(&s, &[("A1", "1"), ("A2", "2"), ("B1", "10"), ("C5", "0.5")]);

    s.set("D1", "=SUM(A1:A2, B1, 10, -2.5)").unwrap();
    assert_eq!(get(&s, "D1"), "20.5");
    s.set("D1", "=SUM(7)").unwrap();
    assert_eq!(get(&s, "D1"), "7");
    s.set("D1", "= SUM ( A1 , A2 : B1 , C5 ) * 2").unwrap();
    assert_eq!(get(&s, "D1"), "29");
    s.set("D1", "=SUM(A1,A1,A1)").unwrap();
    assert_eq!(get(&s, "D1"), "3");
    assert_eq!(s.dependencies("D1").unwrap(), list(&["A1"]));
}

#[test]
fn should_ignore_text_and_empty_cells_in_sum_but_not_in_arithmetic() {
    let s = sut!(create());
    put(
        &s,
        &[
            ("A1", "4"),
            ("A2", "words"),
            ("A3", ""),
            ("A4", "6"),
            ("B1", "=SUM(A1:A4)"),
            ("B2", "=SUM(A2)"),
            ("B3", "=SUM(A2:A3)"),
        ],
    );
    assert_eq!(get(&s, "B1"), "10");
    assert_eq!(get(&s, "B2"), "0");
    assert_eq!(get(&s, "B3"), "0");

    put(
        &s,
        &[
            ("C1", "=A1+A2"),
            ("C2", "=A2*2"),
            ("C3", "=-A2"),
            ("C4", "=SUM(A1:A4)+A2"),
        ],
    );
    for c in ["C1", "C2", "C3", "C4"] {
        assert_eq!(get(&s, c), "#VALUE!", "{c}");
    }
}

#[test]
fn should_copy_text_through_a_bare_reference_but_fail_on_arithmetic() {
    let s = sut!(create());
    put(
        &s,
        &[
            ("A1", "hello"),
            ("B1", "=A1"),
            ("B2", "=(A1)"),
            ("B3", "=B1+1"),
            ("B4", "=SUM(B1:B2)"),
            ("B5", "=Z9"),
        ],
    );
    assert_eq!(get(&s, "B1"), "hello");
    assert_eq!(get(&s, "B2"), "hello");
    assert_eq!(get(&s, "B3"), "#VALUE!");
    assert_eq!(get(&s, "B4"), "0");
    assert_eq!(get(&s, "B5"), "0");
    s.set("A1", "12").unwrap();
    assert_eq!(get(&s, "B1"), "12");
    assert_eq!(get(&s, "B3"), "13");
    assert_eq!(get(&s, "B4"), "24");
}

#[test]
fn should_report_division_by_zero_and_propagate_errors() {
    let s = sut!(create());
    put(
        &s,
        &[
            ("A1", "=1/0"),
            ("A2", "=0/0"),
            ("A3", "=1/B9"),
            ("A4", "=5/(2-2)"),
            ("A5", "=A1+1"),
            ("A6", "=SUM(A1:A2)"),
            ("A7", "=-A1"),
            ("A8", "=2*A5"),
            ("A9", "=1/(A1+1)"),
        ],
    );
    for i in 1..=9 {
        assert_eq!(get(&s, &format!("A{i}")), "#DIV/0!", "A{i}");
    }
    s.set("B1", "2").unwrap();
    s.set("A1", "=1/B1").unwrap();
    assert_eq!(get(&s, "A1"), "0.5");
    assert_eq!(get(&s, "A5"), "1.5");
    assert_eq!(get(&s, "A6"), "#DIV/0!");
    assert_eq!(get(&s, "A7"), "-0.5");
    assert_eq!(get(&s, "A9"), "0.6666666667");
}

#[test]
fn should_report_ref_error_for_invalid_addresses_inside_formulas() {
    let s = sut!(create());
    put(
        &s,
        &[
            ("B1", "5"),
            ("A1", "=Z101"),
            ("A2", "=AA1"),
            ("A3", "=A0+1"),
            ("A4", "=SUM(B1:Z101)"),
            ("A5", "=SUM(B1, AA1)"),
            ("A6", "=B1+A01"),
            ("A7", "=SUM(A0:B1)"),
            ("A8", "=ABC12"),
        ],
    );
    for i in 1..=8 {
        assert_eq!(get(&s, &format!("A{i}")), "#REF!", "A{i}");
    }
    assert!(s.dependencies("A1").unwrap().is_empty());
    assert_eq!(s.dependencies("A5").unwrap(), list(&["B1"]));
    assert_eq!(s.dependencies("A6").unwrap(), list(&["B1"]));
    assert!(s.dependencies("A4").unwrap().is_empty());
    assert_eq!(s.formula_of("A2").unwrap(), "=AA1");
}

#[test]
fn should_report_syntax_errors() {
    let s = sut!(create());
    let bad = [
        "=",
        "=1+",
        "=(1",
        "=1)",
        "=()",
        "=SUM()",
        "=SUM(A1:B2",
        "=SUM(A1,)",
        "=SUM(,A1)",
        "=A1:B2",
        "=foo",
        "=AVG(1)",
        "=1 2",
        "=A1 B1",
        "=1..2",
        "=.5",
        "=5.",
        "=+5",
        "=SUM(1+2)",
        "=SUM(A1+1)",
        "=A",
        "=1+*2",
        "=1**2",
        "=2 3",
        "=SUM(SUM(1))",
        "=SUM(A1:B2:C3)",
        "=SUM",
        "=SUM 1",
        "=1+2)",
        "=$A$1",
        "=A1;B1",
        "=\"x\"",
        "=SUM(--1)",
        "=1%",
    ];
    for f in bad {
        s.set("C3", f).unwrap();
        assert_eq!(get(&s, "C3"), "#ERR!", "{f}");
        assert!(s.dependencies("C3").unwrap().is_empty(), "{f}");
    }
    s.set("C3", "=a1 +  b2 ").unwrap();
    assert_eq!(s.formula_of("C3").unwrap(), "=A1 +  B2 ");
    s.set("C3", "=1+").unwrap();
    assert_eq!(s.formula_of("C3").unwrap(), "=1+");
    assert_eq!(s.used_cells(), list(&["C3"]));
}

#[test]
fn should_propagate_the_first_error_in_row_major_order_and_then_argument_order() {
    let s = sut!(create());
    put(
        &s,
        &[("A1", "1"), ("B1", "=1/0"), ("A2", "=Z101"), ("B2", "5")],
    );

    let cases = [
        ("D1", "=SUM(A1:B2)", "#DIV/0!"),
        ("D1", "=SUM(B2:A1)", "#DIV/0!"),
        ("D1", "=SUM(A2, B1)", "#REF!"),
        ("D1", "=SUM(B1, A2)", "#DIV/0!"),
    ];
    for (c, f, want) in cases {
        s.set(c, f).unwrap();
        assert_eq!(get(&s, c), want, "{f}");
    }

    s.set("E1", "text").unwrap();
    let cases = [
        ("=A2+B1", "#REF!"),
        ("=B1+A2", "#DIV/0!"),
        ("=E1+B1", "#DIV/0!"),
        ("=B1+E1", "#DIV/0!"),
        ("=E1/0", "#VALUE!"),
        ("=E1+1/0", "#DIV/0!"),
    ];
    for (f, want) in cases {
        s.set("D2", f).unwrap();
        assert_eq!(get(&s, "D2"), want, "{f}");
    }
}

#[test]
fn should_recompute_dependents_after_any_change() {
    let s = sut!(create());
    put(
        &s,
        &[
            ("A1", "5"),
            ("A2", "7"),
            ("A3", "=A1+A2*2"),
            ("B1", "=SUM(A1:A3)"),
        ],
    );
    assert_eq!(get(&s, "A3"), "19");
    assert_eq!(get(&s, "B1"), "31");

    s.set("A1", "10").unwrap();
    assert_eq!(get(&s, "A3"), "24");
    assert_eq!(get(&s, "B1"), "41");

    s.set("A2", "text").unwrap();
    assert_eq!(get(&s, "A3"), "#VALUE!");
    assert_eq!(get(&s, "B1"), "#VALUE!");

    s.set("A2", "1").unwrap();
    assert_eq!(get(&s, "A3"), "12");
    assert_eq!(get(&s, "B1"), "23");

    s.set("A3", "100").unwrap();
    assert_eq!(get(&s, "B1"), "111");
    s.set("A3", "=A1").unwrap();
    assert_eq!(get(&s, "B1"), "21");
}

#[test]
fn should_report_formula_dependencies_and_dependents() {
    let s = sut!(create());
    put(
        &s,
        &[
            ("C1", "=a1 + B2 + SUM(A3:B4) + a1"),
            ("D5", "=C1*2"),
            ("A2", "=SUM(C1, B2)"),
        ],
    );

    assert_eq!(s.formula_of("C1").unwrap(), "=A1 + B2 + SUM(A3:B4) + A1");
    assert_eq!(s.formula_of("D5").unwrap(), "=C1*2");
    assert_eq!(s.formula_of("A1").unwrap(), "");
    assert_eq!(
        s.dependencies("C1").unwrap(),
        list(&["A1", "B2", "A3", "B3", "A4", "B4"])
    );
    assert_eq!(s.dependencies("D5").unwrap(), list(&["C1"]));
    assert!(s.dependencies("A1").unwrap().is_empty());
    assert_eq!(s.dependents("A1").unwrap(), list(&["C1"]));
    assert_eq!(s.dependents("B2").unwrap(), list(&["C1", "A2"]));
    assert_eq!(s.dependents("B3").unwrap(), list(&["C1"]));
    assert_eq!(s.dependents("C1").unwrap(), list(&["A2", "D5"]));
    assert!(s.dependents("D5").unwrap().is_empty());
    assert!(s.dependents("Z100").unwrap().is_empty());
}

#[test]
fn should_update_dependencies_when_a_formula_is_replaced() {
    let s = sut!(create());
    put(&s, &[("A1", "1"), ("B1", "2"), ("C1", "=A1+1")]);
    assert_eq!(s.dependents("A1").unwrap(), list(&["C1"]));

    s.set("C1", "=B1+1").unwrap();
    assert_eq!(s.dependencies("C1").unwrap(), list(&["B1"]));
    assert!(s.dependents("A1").unwrap().is_empty());
    assert_eq!(s.dependents("B1").unwrap(), list(&["C1"]));
    let before = s.recompute_count();
    s.set("A1", "50").unwrap();
    assert_eq!(s.recompute_count(), before);
    assert_eq!(get(&s, "C1"), "3");

    s.set("C1", "7").unwrap();
    assert!(s.dependencies("C1").unwrap().is_empty());
    assert!(s.dependents("B1").unwrap().is_empty());
    assert_eq!(s.formula_of("C1").unwrap(), "");
    s.set("C1", "=1/0").unwrap();
    assert!(s.dependencies("C1").unwrap().is_empty());
}

#[test]
fn should_list_used_cells_row_by_row() {
    let s = sut!(create());
    put(
        &s,
        &[
            ("C2", "x"),
            ("A3", "1"),
            ("B1", "=A3"),
            ("Z100", "9"),
            ("A1", "t"),
            ("A2", "=Q9"),
        ],
    );
    assert_eq!(
        s.used_cells(),
        list(&["A1", "B1", "A2", "C2", "A3", "Z100"])
    );
    s.set("A2", "").unwrap();
    assert_eq!(s.used_cells(), list(&["A1", "B1", "C2", "A3", "Z100"]));
}

#[test]
fn should_mark_every_cell_on_a_cycle_and_every_dependent_of_it() {
    let s = sut!(create());
    put(
        &s,
        &[
            ("A1", "=B1+1"),
            ("B1", "=A1+1"),
            ("C1", "=A1*2"),
            ("D1", "=SUM(C1:C2)"),
            ("E1", "=1+1"),
        ],
    );
    for c in ["A1", "B1", "C1", "D1"] {
        assert_eq!(get(&s, c), "#CYCLE!", "{c}");
    }
    assert_eq!(get(&s, "E1"), "2");

    s.set("B1", "4").unwrap();
    assert_eq!(get(&s, "A1"), "5");
    assert_eq!(get(&s, "C1"), "10");
    assert_eq!(get(&s, "D1"), "10");
}

#[test]
fn should_detect_self_and_range_and_long_cycles() {
    let s = sut!(create());
    s.set("A1", "=A1").unwrap();
    assert_eq!(get(&s, "A1"), "#CYCLE!");
    s.set("A1", "=1+A1*2").unwrap();
    assert_eq!(get(&s, "A1"), "#CYCLE!");
    s.set("A1", "3").unwrap();
    assert_eq!(get(&s, "A1"), "3");

    s.set("A2", "=SUM(A1:A3)").unwrap();
    assert_eq!(get(&s, "A2"), "#CYCLE!");
    s.set("A2", "=SUM(A1:A1)").unwrap();
    assert_eq!(get(&s, "A2"), "3");

    put(
        &s,
        &[("B1", "=B2"), ("B2", "=B3"), ("B3", "=B4"), ("B4", "=B1")],
    );
    for c in ["B1", "B2", "B3", "B4"] {
        assert_eq!(get(&s, c), "#CYCLE!", "{c}");
    }
    s.set("B3", "8").unwrap();
    assert_eq!(get(&s, "B1"), "8");
    assert_eq!(get(&s, "B2"), "8");
    assert_eq!(get(&s, "B4"), "8");
}

#[test]
fn should_not_recompute_anything_when_an_unrelated_cell_changes() {
    let s = sut!(create());
    put(&s, &[("A1", "1"), ("B1", "=A1+1"), ("C1", "=SUM(A1:B1)")]);
    let before = s.recompute_count();

    s.set("F9", "123").unwrap();
    s.set("F10", "text").unwrap();
    s.set("G9", "").unwrap();
    s.set("C2", "5").unwrap();

    assert_eq!(s.recompute_count(), before);
    assert_eq!(get(&s, "C1"), "3");
}

#[test]
fn should_count_each_formula_evaluation_at_set_time() {
    let s = sut!(create());
    assert_eq!(s.recompute_count(), 0);
    s.set("A1", "1").unwrap();
    s.set("A2", "text").unwrap();
    assert_eq!(s.recompute_count(), 0);
    s.set("B1", "=A1+1").unwrap();
    assert_eq!(s.recompute_count(), 1);
    s.set("B2", "=SUM(A1:A2)").unwrap();
    assert_eq!(s.recompute_count(), 2);
    s.set("B3", "=1/0").unwrap();
    assert_eq!(s.recompute_count(), 3);
    s.set("B1", "=A1+2").unwrap();
    assert_eq!(s.recompute_count(), 4);
    assert_eq!(s.set("a0", "=1"), Err(SheetError::InvalidAddress));
    assert_eq!(s.recompute_count(), 4);
}

#[test]
fn should_evaluate_each_dependent_of_a_diamond_exactly_once() {
    let s = sut!(create());
    put(
        &s,
        &[
            ("A1", "1"),
            ("B1", "=A1+1"),
            ("C1", "=A1+2"),
            ("D1", "=B1+C1"),
        ],
    );
    assert_eq!(get(&s, "D1"), "5");
    assert_eq!(s.recompute_count(), 3);

    s.set("A1", "10").unwrap();
    assert_eq!(get(&s, "D1"), "23");
    assert_eq!(s.recompute_count(), 6);

    put(&s, &[("E1", "=D1+B1+A1"), ("F1", "=SUM(A1:E1)")]);
    let before = s.recompute_count();
    s.set("A1", "0").unwrap();
    assert_eq!(s.recompute_count() - before, 5);
    assert_eq!(get(&s, "F1"), "10");
}

#[test]
fn should_evaluate_a_chain_of_one_hundred_dependents_exactly_once_per_change() {
    let s = sut!(create());
    s.set("A1", "1").unwrap();
    for r in 1..=100 {
        let f = if r == 1 {
            "=A1+1".to_string()
        } else {
            format!("=B{}+1", r - 1)
        };
        s.set(&format!("B{r}"), &f).unwrap();
    }
    assert_eq!(get(&s, "B100"), "101");
    let before = s.recompute_count();

    s.set("A1", "50").unwrap();
    assert_eq!(s.recompute_count() - before, 100);
    assert_eq!(get(&s, "B100"), "150");

    let before = s.recompute_count();
    s.set("B50", "=A1+1000").unwrap();
    assert_eq!(s.recompute_count() - before, 51);
    assert_eq!(get(&s, "B100"), "1100");

    let before = s.recompute_count();
    s.set("A1", "").unwrap();
    assert_eq!(s.recompute_count() - before, 100);
    assert_eq!(get(&s, "B100"), "1050");
    assert_eq!(get(&s, "B49"), "49");
}

#[test]
fn should_count_cycle_members_only_when_they_are_actually_evaluated() {
    let s = sut!(create());
    s.set("A1", "=B1+1").unwrap();
    let start = s.recompute_count();
    assert_eq!(start, 1);

    s.set("B1", "=A1+1").unwrap();
    assert_eq!(s.recompute_count(), start);

    s.set("C1", "=A1*2").unwrap();
    assert_eq!(s.recompute_count(), start + 1);

    s.set("B1", "4").unwrap();
    assert_eq!(s.recompute_count(), start + 3);
    assert_eq!(get(&s, "C1"), "10");
}

#[test]
fn should_survive_a_chain_through_all_twenty_six_hundred_cells_on_a_default_stack() {
    on_default_stack(|| {
        let s = sut!(create());
        s.set("A1", "1").unwrap();
        for i in 1..2600 {
            s.set(&cell_name(i), &format!("={}+1", cell_name(i - 1)))
                .unwrap();
        }
        assert_eq!(get(&s, "Z100"), "2600");
        let before = s.recompute_count();
        let start = Instant::now();
        for k in 1..=100u64 {
            s.set("A1", &(k * 10).to_string()).unwrap();
            assert_eq!(get(&s, "Z100"), (k * 10 + 2599).to_string());
        }
        assert_eq!(s.recompute_count() - before, 100 * 2599);
        assert!(start.elapsed().as_millis() < 20_000);
    });
}

#[test]
fn should_keep_all_cells_consistent_under_concurrent_sets_and_gets() {
    let s = sut!(create());
    run_concurrently(8, |t| {
        let col = (b'A' + t as u8) as char;
        s.set(&format!("{col}1"), "1").unwrap();
        for r in 2..=100 {
            s.set(&format!("{col}{r}"), &format!("={col}{}+1", r - 1))
                .unwrap();
            s.get(&format!("{col}{r}")).unwrap();
            s.used_cells();
        }
    });
    for t in 0..8u8 {
        assert_eq!(get(&s, &format!("{}100", (b'A' + t) as char)), "100");
    }
    assert_eq!(s.recompute_count(), 8 * 99);
    assert_eq!(s.used_cells().len(), 800);
}

#[test]
fn should_never_expose_a_half_updated_chain_to_readers() {
    let s = sut!(create());
    s.set("A1", "1000").unwrap();
    for r in 1..=99 {
        let f = if r == 1 {
            "=A1+1".to_string()
        } else {
            format!("=B{}+1", r - 1)
        };
        s.set(&format!("B{r}"), &f).unwrap();
    }
    let seen: Vec<Vec<String>> = collect_concurrently(6, |t| {
        let mut out = vec![];
        if t < 2 {
            for i in 0..300 {
                s.set("A1", if i % 2 == 0 { "2000" } else { "1000" })
                    .unwrap();
            }
        } else {
            for _ in 0..2_000 {
                out.push(s.get("B99").unwrap());
            }
        }
        out
    });
    for reads in seen {
        for v in reads {
            assert!(v == "1099" || v == "2099", "saw {v}");
        }
    }
    let a1: i64 = get(&s, "A1").parse().unwrap();
    assert_eq!(get(&s, "B99"), (a1 + 99).to_string());
}
