#![allow(unused_imports, clippy::needless_range_loop)]
use p133_spreadsheet_grid::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::Arc;
use std::time::Instant;

const DEFAULT_STYLE: &str = "false|false|11|black|none|GENERAL";

fn bold(b: bool) -> StyleChange {
    StyleChange {
        bold: Some(b),
        ..Default::default()
    }
}

fn format(f: &str) -> StyleChange {
    StyleChange {
        number_format: Some(f.to_string()),
        ..Default::default()
    }
}

fn col_name(c: u32) -> String {
    if c > 26 {
        format!(
            "{}{}",
            (b'A' + ((c - 1) / 26 - 1) as u8) as char,
            (b'A' + ((c - 1) % 26) as u8) as char
        )
    } else {
        ((b'A' + (c - 1) as u8) as char).to_string()
    }
}

fn v(g: &Arc<dyn SpreadsheetGrid>, cell: &str) -> String {
    g.get_value(cell).unwrap()
}

fn column(g: &Arc<dyn SpreadsheetGrid>, col: &str, from: u32, to: u32) -> String {
    (from..=to)
        .map(|r| v(g, &format!("{col}{r}")))
        .collect::<Vec<_>>()
        .join(",")
}

fn invalid<T: std::fmt::Debug>(r: Result<T, GridError>) {
    assert_eq!(r.unwrap_err(), GridError::InvalidArgument);
}

#[test]
fn should_start_empty_with_default_style() {
    let g = sut!(create());

    assert_eq!(v(&g, "A1"), "");
    assert_eq!(g.value_type("A1"), Ok(ValueType::Empty));
    assert_eq!(g.style_of("ZZ1000000").unwrap(), DEFAULT_STYLE);
    assert_eq!(g.used_range(), "");
    assert_eq!(g.row_count(), 0);
    assert_eq!(g.column_count(), 0);
    assert_eq!(g.distinct_style_count(), 0);
}

#[test]
fn should_store_numbers_and_text_with_case_insensitive_addresses() {
    let g = sut!(create());
    g.set_number("b3", 42.0).unwrap();

    assert_eq!(v(&g, "B3"), "42");
    assert_eq!(g.value_type("b3"), Ok(ValueType::Number));

    g.set_text("B3", "hello").unwrap();
    assert_eq!(v(&g, "b3"), "hello");
    assert_eq!(g.value_type("B3"), Ok(ValueType::Text));

    g.set_text("C1", "42").unwrap();
    assert_eq!(g.value_type("C1"), Ok(ValueType::Text));
    assert_eq!(v(&g, "C1"), "42");
}

#[test]
fn should_reject_invalid_addresses() {
    let g = sut!(create());
    for bad in [
        "", "A", "1", "A0", "A01", "AAA1", "A1000001", "A 1", "A1 ", " A1", "1A", "A-1", "A1.5",
        "$A$1", "A1:B2",
    ] {
        invalid(g.set_number(bad, 1.0));
        invalid(g.get_value(bad));
        invalid(g.style_of(bad));
    }
    g.set_number("ZZ1000000", 1.0).unwrap();
    g.set_number("a1", 1.0).unwrap();
    assert_eq!(g.row_count(), 1_000_000);
    assert_eq!(g.column_count(), 702);
}

#[test]
fn should_reject_bad_values_and_keep_the_old_one() {
    let g = sut!(create());
    g.set_number("A1", 5.0).unwrap();

    invalid(g.set_number("A1", f64::NAN));
    invalid(g.set_number("A1", f64::INFINITY));
    invalid(g.set_number("A1", f64::NEG_INFINITY));
    assert_eq!(v(&g, "A1"), "5");
}

#[test]
fn should_display_general_numbers_with_up_to_ten_decimals() {
    let g = sut!(create());
    let cases: [(f64, &str); 13] = [
        (1.5, "1.5"),
        (2.0, "2"),
        (0.1 + 0.2, "0.3"),
        (1e15, "1000000000000000"),
        (1.23456789012, "1.2345678901"),
        (0.00000000004, "0"),
        (0.00000000005, "0.0000000001"),
        (-2.5, "-2.5"),
        (1e-10, "0.0000000001"),
        (123456789.125, "123456789.125"),
        (-0.0, "0"),
        (100.0, "100"),
        (-0.00000000001, "0"),
    ];
    for (i, (n, want)) in cases.iter().enumerate() {
        let cell = format!("A{}", i + 1);
        g.set_number(&cell, *n).unwrap();
        assert_eq!(v(&g, &cell), *want, "value {n}");
    }
}

#[test]
fn should_apply_integer_format_rounding_half_away_from_zero() {
    let g = sut!(create());
    g.set_style_range("A1", "A6", &format("INTEGER")).unwrap();
    let cases = [
        (2.5, "3"),
        (-2.5, "-3"),
        (2.4, "2"),
        (-0.4, "0"),
        (1234.5678, "1235"),
        (0.5, "1"),
    ];
    for (i, (n, want)) in cases.iter().enumerate() {
        let cell = format!("A{}", i + 1);
        g.set_number(&cell, *n).unwrap();
        assert_eq!(v(&g, &cell), *want, "value {n}");
    }
}

#[test]
fn should_apply_two_decimals_and_percent_formats() {
    let g = sut!(create());
    g.set_style_range("A1", "A6", &format("two_decimals"))
        .unwrap();
    g.set_style_range("B1", "B7", &format("PERCENT")).unwrap();
    let two = [
        (3.0, "3.00"),
        (1.005, "1.01"),
        (2.345, "2.35"),
        (-0.001, "0.00"),
        (-1.005, "-1.01"),
        (1234.5, "1234.50"),
    ];
    for (i, (n, want)) in two.iter().enumerate() {
        let cell = format!("A{}", i + 1);
        g.set_number(&cell, *n).unwrap();
        assert_eq!(v(&g, &cell), *want, "two decimals of {n}");
    }
    let pct = [
        (0.256, "26%"),
        (0.5, "50%"),
        (1.0, "100%"),
        (0.005, "1%"),
        (-0.5, "-50%"),
        (0.001, "0%"),
        (12.0, "1200%"),
    ];
    for (i, (n, want)) in pct.iter().enumerate() {
        let cell = format!("B{}", i + 1);
        g.set_number(&cell, *n).unwrap();
        assert_eq!(v(&g, &cell), *want, "percent of {n}");
    }
}

#[test]
fn should_render_stored_numbers_again_when_the_format_changes_and_leave_text_alone() {
    let g = sut!(create());
    g.set_number("A1", 0.256).unwrap();
    g.set_text("A2", "0.256").unwrap();

    g.set_style_range("A1", "A2", &format("PERCENT")).unwrap();
    assert_eq!(v(&g, "A1"), "26%");
    assert_eq!(v(&g, "A2"), "0.256");

    g.set_style("A1", &format("GENERAL")).unwrap();
    assert_eq!(v(&g, "A1"), "0.256");
}

#[test]
fn should_merge_partial_style_changes_and_normalise_names() {
    let g = sut!(create());
    g.set_style("B2", &bold(true)).unwrap();
    assert_eq!(
        g.style_of("B2").unwrap(),
        "true|false|11|black|none|GENERAL"
    );

    g.set_style(
        "b2",
        &StyleChange {
            italic: Some(true),
            font_size: Some(14),
            text_color: Some("RED".into()),
            fill_color: Some("Light-Blue".into()),
            number_format: Some("percent".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        g.style_of("B2").unwrap(),
        "true|true|14|red|light-blue|PERCENT"
    );

    g.set_style(
        "B2",
        &StyleChange {
            bold: Some(false),
            fill_color: Some("none".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(g.style_of("B2").unwrap(), "false|true|14|red|none|PERCENT");

    g.set_style("B2", &StyleChange::default()).unwrap();
    assert_eq!(g.style_of("B2").unwrap(), "false|true|14|red|none|PERCENT");
    assert_eq!(g.style_of("B3").unwrap(), DEFAULT_STYLE);
}

#[test]
fn should_reject_invalid_style_changes_without_partial_effects() {
    let g = sut!(create());
    g.set_style("A1", &bold(true)).unwrap();
    let with = |f: fn(&mut StyleChange)| {
        let mut c = StyleChange {
            bold: Some(false),
            ..Default::default()
        };
        f(&mut c);
        c
    };

    invalid(g.set_style("A1", &with(|c| c.font_size = Some(7))));
    invalid(g.set_style("A1", &with(|c| c.font_size = Some(73))));
    invalid(g.set_style("A1", &with(|c| c.font_size = Some(-11))));
    invalid(g.set_style("A1", &with(|c| c.text_color = Some("dark red".into()))));
    invalid(g.set_style("A1", &with(|c| c.fill_color = Some("a|b".into()))));
    invalid(g.set_style("A1", &with(|c| c.text_color = Some("".into()))));
    invalid(g.set_style("A1", &with(|c| c.number_format = Some("SCIENTIFIC".into()))));
    invalid(g.set_style("A0", &bold(true)));
    invalid(g.set_style_range("A1", "B2", &with(|c| c.font_size = Some(99))));

    assert_eq!(
        g.style_of("A1").unwrap(),
        "true|false|11|black|none|GENERAL"
    );
    assert_eq!(g.style_of("B2").unwrap(), DEFAULT_STYLE);
    for size in [8, 72] {
        g.set_style(
            "A1",
            &StyleChange {
                font_size: Some(size),
                ..Default::default()
            },
        )
        .unwrap();
    }
    assert_eq!(
        g.style_of("A1").unwrap(),
        "true|false|72|black|none|GENERAL"
    );
}

#[test]
fn should_count_a_styled_empty_cell_as_used_until_it_becomes_default_again() {
    let g = sut!(create());
    g.set_style("C3", &bold(true)).unwrap();

    assert_eq!(v(&g, "C3"), "");
    assert_eq!(g.value_type("C3"), Ok(ValueType::Empty));
    assert_eq!(g.used_range(), "C3:C3");

    g.set_style("C3", &bold(false)).unwrap();
    assert_eq!(g.used_range(), "");
    assert_eq!(g.distinct_style_count(), 0);
}

#[test]
fn should_clear_value_and_style_but_empty_text_keeps_the_style() {
    let g = sut!(create());
    g.set_number("A1", 7.0).unwrap();
    g.set_style("A1", &bold(true)).unwrap();
    g.set_number("A2", 8.0).unwrap();
    g.set_style("A2", &bold(true)).unwrap();

    g.set_text("A1", "").unwrap();
    assert_eq!(g.value_type("A1"), Ok(ValueType::Empty));
    assert!(g.style_of("A1").unwrap().starts_with("true|"));

    g.clear("A2").unwrap();
    assert_eq!(g.value_type("A2"), Ok(ValueType::Empty));
    assert_eq!(g.style_of("A2").unwrap(), DEFAULT_STYLE);
    assert_eq!(g.used_range(), "A1:A1");
}

#[test]
fn should_count_distinct_non_default_styles_still_in_use() {
    let g = sut!(create());
    g.set_style_range("A1", "A100", &bold(true)).unwrap();
    assert_eq!(g.distinct_style_count(), 1);

    g.set_style(
        "B1",
        &StyleChange {
            bold: Some(true),
            italic: Some(true),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(g.distinct_style_count(), 2);

    g.set_style("C1", &bold(true)).unwrap();
    assert_eq!(g.distinct_style_count(), 2);

    g.clear("B1").unwrap();
    assert_eq!(g.distinct_style_count(), 1);

    g.set_style_range("A1", "A100", &bold(false)).unwrap();
    assert_eq!(g.distinct_style_count(), 1); // C1 is still bold

    g.set_style("C1", &bold(false)).unwrap();
    assert_eq!(g.distinct_style_count(), 0);
}

#[test]
fn should_style_every_cell_of_a_rectangle_and_keep_values() {
    let g = sut!(create());
    g.set_number("B2", 1.5).unwrap();
    g.set_style_range(
        "B2",
        "D4",
        &StyleChange {
            italic: Some(true),
            font_size: Some(20),
            fill_color: Some("yellow".into()),
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(
        g.style_of("B2").unwrap(),
        "false|true|20|black|yellow|GENERAL"
    );
    assert_eq!(
        g.style_of("D4").unwrap(),
        "false|true|20|black|yellow|GENERAL"
    );
    assert_eq!(g.style_of("E4").unwrap(), DEFAULT_STYLE);
    assert_eq!(g.style_of("A1").unwrap(), DEFAULT_STYLE);
    assert_eq!(v(&g, "B2"), "1.5");
    assert_eq!(g.used_range(), "B2:D4");
    assert_eq!(g.distinct_style_count(), 1);
}

#[test]
fn should_reject_reversed_and_oversized_ranges() {
    let g = sut!(create());

    invalid(g.set_style_range("B2", "A2", &bold(true)));
    invalid(g.set_style_range("B2", "B1", &bold(true)));
    invalid(g.set_style_range("A1", "B500001", &bold(true))); // 1,000,002 cells: over the limit
    invalid(g.fill_number("A1", "B500001", 1.0));
    invalid(g.fill_text("A1", "ZZ1000000", "x"));
    invalid(g.clear_range("B2", "A1"));
    assert_eq!(g.used_range(), "");
}

#[test]
fn should_fill_values_and_keep_styles() {
    let g = sut!(create());
    g.set_style("B2", &bold(true)).unwrap();
    g.fill_number("A1", "C3", 2.5).unwrap();

    assert_eq!(v(&g, "A1"), "2.5");
    assert_eq!(v(&g, "C3"), "2.5");
    assert_eq!(v(&g, "D3"), "");
    assert!(g.style_of("B2").unwrap().starts_with("true|"));

    g.fill_text("B2", "C3", "x").unwrap();
    assert_eq!(v(&g, "B2"), "x");
    assert_eq!(g.value_type("C2"), Ok(ValueType::Text));
    assert_eq!(v(&g, "A1"), "2.5");

    g.fill_text("A1", "C3", "").unwrap();
    assert_eq!(g.used_range(), "B2:B2"); // only the styled cell remains
    invalid(g.fill_number("A1", "A2", f64::NAN));
}

#[test]
fn should_clear_only_the_range_including_styles() {
    let g = sut!(create());
    g.fill_number("A1", "D4", 1.0).unwrap();
    g.set_style_range("A1", "D4", &bold(true)).unwrap();

    g.clear_range("B2", "C3").unwrap();

    assert_eq!(g.value_type("B2"), Ok(ValueType::Empty));
    assert_eq!(g.style_of("C3").unwrap(), DEFAULT_STYLE);
    assert_eq!(v(&g, "A1"), "1");
    assert_eq!(v(&g, "D4"), "1");
    assert_eq!(v(&g, "B1"), "1");
    assert!(g.style_of("B1").unwrap().starts_with("true|"));
    assert_eq!(g.used_range(), "A1:D4");
}

#[test]
fn should_copy_values_and_styles_and_overwrite_the_destination() {
    let g = sut!(create());
    g.set_number("A1", 1.0).unwrap();
    g.set_style(
        "A1",
        &StyleChange {
            bold: Some(true),
            text_color: Some("red".into()),
            number_format: Some("TWO_DECIMALS".into()),
            ..Default::default()
        },
    )
    .unwrap();
    g.set_text("B1", "x").unwrap();
    // A2 stays empty; the destination cells below hold data that must be wiped by the empty source cells
    g.fill_number("E5", "F6", 9.0).unwrap();
    g.set_style_range(
        "E5",
        "F6",
        &StyleChange {
            italic: Some(true),
            ..Default::default()
        },
    )
    .unwrap();

    g.copy("A1", "B2", "E5").unwrap();

    assert_eq!(v(&g, "E5"), "1.00");
    assert_eq!(
        g.style_of("E5").unwrap(),
        "true|false|11|red|none|TWO_DECIMALS"
    );
    assert_eq!(v(&g, "F5"), "x");
    assert_eq!(g.value_type("E6"), Ok(ValueType::Empty));
    assert_eq!(g.style_of("F6").unwrap(), DEFAULT_STYLE);
    assert_eq!(v(&g, "A1"), "1.00");
    assert_eq!(v(&g, "B1"), "x");
}

#[test]
fn should_copy_overlapping_ranges_as_if_through_a_temporary_buffer() {
    let g = sut!(create());
    g.set_number("A1", 1.0).unwrap();
    g.set_number("A2", 2.0).unwrap();
    g.set_number("A3", 3.0).unwrap();
    g.copy("A1", "A3", "A2").unwrap();
    assert_eq!(column(&g, "A", 1, 4), "1,1,2,3");

    g.copy("A2", "A4", "A1").unwrap();
    assert_eq!(column(&g, "A", 1, 4), "1,2,3,3");

    let h = sut!(create());
    h.set_text("A1", "a").unwrap();
    h.set_text("B1", "b").unwrap();
    h.set_text("A2", "c").unwrap();
    h.set_text("B2", "d").unwrap();
    h.copy("A1", "B2", "B2").unwrap();
    assert_eq!(
        [v(&h, "B2"), v(&h, "C2"), v(&h, "B3"), v(&h, "C3")],
        ["a", "b", "c", "d"]
    );
    assert_eq!(v(&h, "A2"), "c");
}

#[test]
fn should_reject_a_copy_that_leaves_the_grid_and_change_nothing() {
    let g = sut!(create());
    g.set_number("A1", 1.0).unwrap();
    g.set_number("B1", 2.0).unwrap();

    invalid(g.copy("A1", "A2", "A1000000"));
    invalid(g.copy("A1", "B1", "ZZ1"));
    invalid(g.move_range("A1", "B1", "ZZ1"));
    invalid(g.copy("B1", "A1", "C1"));
    invalid(g.copy("A1", "A1", "bad"));

    assert_eq!(g.used_range(), "A1:B1");
    assert_eq!(v(&g, "A1"), "1");
    g.copy("A1", "A2", "A999999").unwrap();
}

#[test]
fn should_move_cells_and_clear_the_source_outside_the_destination() {
    let g = sut!(create());
    g.set_number("A1", 1.0).unwrap();
    g.set_number("B1", 2.0).unwrap();
    g.set_number("A2", 3.0).unwrap();
    g.set_number("B2", 4.0).unwrap();
    g.set_style("A1", &bold(true)).unwrap();

    g.move_range("A1", "B2", "B2").unwrap(); // destination B2:C3 overlaps the source in B2

    assert_eq!(v(&g, "B2"), "1");
    assert!(g.style_of("B2").unwrap().starts_with("true|"));
    assert_eq!(v(&g, "C2"), "2");
    assert_eq!(v(&g, "B3"), "3");
    assert_eq!(v(&g, "C3"), "4");
    assert_eq!(g.value_type("A1"), Ok(ValueType::Empty));
    assert_eq!(g.style_of("A1").unwrap(), DEFAULT_STYLE);
    assert_eq!(g.value_type("B1"), Ok(ValueType::Empty));
    assert_eq!(g.value_type("A2"), Ok(ValueType::Empty));
    assert_eq!(g.used_range(), "B2:C3");
}

#[test]
fn should_insert_rows_shifting_values_and_styles() {
    let g = sut!(create());
    g.set_number("A1", 1.0).unwrap();
    g.set_number("A2", 2.0).unwrap();
    g.set_number("B3", 3.0).unwrap();
    g.set_style("B3", &bold(true)).unwrap();

    g.insert_row(2).unwrap();

    assert_eq!(v(&g, "A1"), "1");
    assert_eq!(g.value_type("A2"), Ok(ValueType::Empty));
    assert_eq!(v(&g, "A3"), "2");
    assert_eq!(v(&g, "B4"), "3");
    assert!(g.style_of("B4").unwrap().starts_with("true|"));
    assert_eq!(g.style_of("B3").unwrap(), DEFAULT_STYLE);
    assert_eq!(g.row_count(), 4);
    invalid(g.insert_row(0));
    invalid(g.insert_row(1_000_001));
}

#[test]
fn should_drop_cells_pushed_past_the_last_row_or_column() {
    let g = sut!(create());
    g.set_number("A1000000", 1.0).unwrap();
    g.set_number("A999999", 2.0).unwrap();
    g.set_number("ZZ1", 3.0).unwrap();
    g.set_number("ZY2", 4.0).unwrap();

    g.insert_row(1).unwrap();
    assert_eq!(v(&g, "A1000000"), "2");
    assert_eq!(v(&g, "ZZ2"), "3");
    assert_eq!(g.row_count(), 1_000_000);

    g.insert_column("A").unwrap();
    assert_eq!(v(&g, "ZZ2"), "");
    assert_eq!(v(&g, "ZZ3"), "4");
    assert_eq!(v(&g, "B1000000"), "2");
    assert_eq!(g.column_count(), 702);
}

#[test]
fn should_delete_rows_and_make_their_cells_vanish() {
    let g = sut!(create());
    g.set_number("A1", 1.0).unwrap();
    g.set_number("A2", 2.0).unwrap();
    g.set_number("A3", 3.0).unwrap();
    g.set_number("C3", 33.0).unwrap();

    g.delete_row(2).unwrap();

    assert_eq!(v(&g, "A1"), "1");
    assert_eq!(v(&g, "A2"), "3");
    assert_eq!(v(&g, "C2"), "33");
    assert_eq!(v(&g, "A3"), "");
    assert_eq!(g.used_range(), "A1:C2");

    g.delete_row(500).unwrap(); // deleting an empty row still closes the gap and changes nothing here
    assert_eq!(v(&g, "A2"), "3");
    invalid(g.delete_row(0));
    invalid(g.delete_row(1_000_001));
}

#[test]
fn should_insert_and_delete_columns() {
    let g = sut!(create());
    g.set_number("A1", 1.0).unwrap();
    g.set_number("B1", 2.0).unwrap();
    g.set_number("C1", 3.0).unwrap();
    g.set_style("C1", &bold(true)).unwrap();

    g.insert_column("b").unwrap();
    assert_eq!(v(&g, "A1"), "1");
    assert_eq!(g.value_type("B1"), Ok(ValueType::Empty));
    assert_eq!(v(&g, "C1"), "2");
    assert_eq!(v(&g, "D1"), "3");
    assert!(g.style_of("D1").unwrap().starts_with("true|"));
    assert_eq!(g.column_count(), 4);

    g.delete_column("A").unwrap();
    assert_eq!(g.value_type("A1"), Ok(ValueType::Empty));
    assert_eq!(v(&g, "B1"), "2");
    assert_eq!(v(&g, "C1"), "3");
    assert_eq!(g.used_range(), "B1:C1");

    for bad in ["", "A1", "AAA", "1", "A$"] {
        invalid(g.insert_column(bad));
        invalid(g.delete_column(bad));
    }
}

#[test]
fn should_report_the_used_range_and_counts() {
    let g = sut!(create());
    g.set_number("B2", 1.0).unwrap();
    g.set_text("D7", "x").unwrap();

    assert_eq!(g.used_range(), "B2:D7");
    assert_eq!(g.row_count(), 7);
    assert_eq!(g.column_count(), 4);

    g.set_style("AA9", &bold(true)).unwrap();
    assert_eq!(g.used_range(), "B2:AA9");
    assert_eq!(g.column_count(), 27);
    assert_eq!(g.row_count(), 9);

    g.clear_range("A1", "ZZ1000000").unwrap();
    assert_eq!(g.used_range(), "");
    assert_eq!(g.row_count(), 0);
    assert_eq!(g.column_count(), 0);
}

#[test]
fn should_sort_numbers_before_text_before_empty_and_keep_rows_together() {
    let g = sut!(create());
    // key column A, payload column B
    g.set_text("A1", "banana").unwrap();
    g.set_text("B1", "r1").unwrap();
    g.set_number("A2", 10.0).unwrap();
    g.set_text("B2", "r2").unwrap();
    g.set_text("B3", "r3").unwrap(); // empty key
    g.set_text("A4", "Apple").unwrap();
    g.set_text("B4", "r4").unwrap();
    g.set_number("A5", -3.5).unwrap();
    g.set_text("B5", "r5").unwrap();
    g.set_style("B5", &bold(true)).unwrap();
    g.set_text("A6", "cherry").unwrap();
    g.set_text("B6", "r6").unwrap();

    g.sort_range("A1", "B6", "A", true).unwrap();

    assert_eq!(column(&g, "A", 1, 6), "-3.5,10,Apple,banana,cherry,");
    assert_eq!(column(&g, "B", 1, 6), "r5,r2,r4,r1,r6,r3");
    assert!(g.style_of("B1").unwrap().starts_with("true|"));
    assert_eq!(g.style_of("B5").unwrap(), DEFAULT_STYLE);
}

#[test]
fn should_sort_stably_and_descend_within_groups_only() {
    let g = sut!(create());
    let keys = ["b", "A", "a", "2", "1", "", "B", "x"];
    for (i, k) in keys.iter().enumerate() {
        let row = i + 1;
        if *k == "2" || *k == "1" {
            g.set_number(&format!("A{row}"), k.parse().unwrap())
                .unwrap();
        } else if !k.is_empty() {
            g.set_text(&format!("A{row}"), k).unwrap();
        }
        g.set_number(&format!("B{row}"), row as f64).unwrap();
    }

    g.sort_range("A1", "B8", "A", true).unwrap();
    // numbers 1,2 ; text a/A (stable: A before a), b/B (stable: b before B), x ; empty
    assert_eq!(column(&g, "A", 1, 8), "1,2,A,a,b,B,x,");
    assert_eq!(column(&g, "B", 1, 8), "5,4,2,3,1,7,8,6");

    g.sort_range("A1", "B8", "A", false).unwrap();
    // numbers 2,1 ; text x, then b/B tie in current order (b,B), then A/a tie (A,a) ; empty last
    assert_eq!(column(&g, "A", 1, 8), "2,1,x,b,B,A,a,");
    assert_eq!(column(&g, "B", 1, 8), "4,5,8,1,7,2,3,6");
}

#[test]
fn should_sort_only_the_given_rows_and_columns_and_reject_bad_keys() {
    let g = sut!(create());
    for r in 1..=4u32 {
        g.set_number(&format!("B{r}"), (5 - r) as f64).unwrap();
        g.set_number(&format!("A{r}"), (100 + r) as f64).unwrap();
        g.set_number(&format!("C{r}"), (200 + r) as f64).unwrap();
    }

    g.sort_range("B2", "C4", "B", true).unwrap();

    assert_eq!(column(&g, "A", 1, 4), "101,102,103,104");
    assert_eq!(column(&g, "B", 1, 4), "4,1,2,3");
    assert_eq!(column(&g, "C", 1, 4), "201,204,203,202");

    invalid(g.sort_range("B2", "C4", "A", true));
    invalid(g.sort_range("B2", "C4", "D", true));
    invalid(g.sort_range("B2", "C4", "B1", true));
    invalid(g.sort_range("C4", "B2", "B", true));
    assert_eq!(column(&g, "B", 1, 4), "4,1,2,3");
}

#[test]
fn should_keep_every_concurrent_write_to_distinct_cells() {
    let g = sut!(create());
    let (threads, per) = (8u32, 1_000u32);

    run_concurrently(threads as usize, |t| {
        let t = t as u32;
        for i in 1..=per {
            g.set_number(
                &format!("{}{}", col_name(t + 1), i),
                (t * 10_000 + i) as f64,
            )
            .unwrap();
        }
    });

    for t in 0..threads {
        for i in (1..=per).step_by(97) {
            assert_eq!(
                v(&g, &format!("{}{}", col_name(t + 1), i)),
                (t * 10_000 + i).to_string()
            );
        }
    }
    assert_eq!(g.row_count(), per);
    assert_eq!(g.column_count(), threads);
    assert_eq!(g.used_range(), "A1:H1000");
}

#[test]
fn should_apply_concurrent_row_insertions_without_losing_the_cell() {
    let g = sut!(create());
    g.set_text("A1", "anchor").unwrap();

    run_concurrently(8, |_| {
        for _ in 0..50 {
            g.insert_row(1).unwrap();
        }
    });

    assert_eq!(v(&g, "A401"), "anchor");
    assert_eq!(g.used_range(), "A401:A401");
}

#[test]
fn should_handle_two_hundred_thousand_cells_and_a_thousand_structure_operations() {
    let g = sut!(create());
    for r in 1..=2_000u32 {
        for c in 1..=100u32 {
            g.set_number(&format!("{}{}", col_name(c), r), (r * 1000 + c) as f64)
                .unwrap();
        }
    }
    let start = Instant::now();
    for i in 0..1_000u32 {
        match i % 4 {
            0 => g.insert_row(1_500 + i % 50).unwrap(),
            1 => g.insert_column(&col_name(80 + i % 10)).unwrap(),
            2 => g.delete_row(1_800 + i % 50).unwrap(),
            _ => g.delete_column(&col_name(85 + i % 10)).unwrap(),
        }
    }
    let ms = start.elapsed().as_millis();

    assert!(
        ms < 60_000,
        "1,000 structure operations on 200k cells took {ms} ms"
    );
    assert_eq!(v(&g, "A1"), "1001");
    assert_eq!(v(&g, "B1000"), "1000002");
    assert!(g.used_range().starts_with("A1:"));
}
