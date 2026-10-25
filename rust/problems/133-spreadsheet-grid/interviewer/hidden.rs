#![allow(unused_imports, clippy::needless_range_loop)]
use p133_spreadsheet_grid::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

fn col(c: u32) -> String {
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

fn addr(r: u32, c: u32) -> String {
    format!("{}{}", col(c), r)
}

fn bold(b: bool) -> StyleChange {
    StyleChange {
        bold: Some(b),
        ..Default::default()
    }
}

fn pct(f: &str) -> StyleChange {
    StyleChange {
        number_format: Some(f.to_string()),
        ..Default::default()
    }
}

/// Tiny reference model: sparse map of (row, col) -> cell, plain loops.
#[derive(Clone)]
struct MCell {
    text: Option<String>,
    number: Option<i64>,
    bold: bool,
}

#[derive(Default)]
struct Model {
    cells: HashMap<(u32, u32), MCell>,
}

impl Model {
    fn put(&mut self, r: u32, c: u32, cell: Option<MCell>) {
        match cell {
            Some(x) if x.text.is_some() || x.number.is_some() || x.bold => {
                self.cells.insert((r, c), x);
            }
            _ => {
                self.cells.remove(&(r, c));
            }
        }
    }

    fn get(&self, r: u32, c: u32) -> Option<MCell> {
        self.cells.get(&(r, c)).cloned()
    }

    fn shift(&mut self, rows: bool, at: u32, delta: i32) {
        let old = std::mem::take(&mut self.cells);
        for ((r, c), cell) in old {
            let mut x = if rows { r } else { c };
            if delta < 0 && x == at {
                continue;
            }
            if x >= at {
                x = (x as i32 + delta) as u32;
            }
            self.cells.insert(if rows { (x, c) } else { (r, x) }, cell);
        }
    }

    fn copy(&mut self, (r1, c1, r2, c2): (u32, u32, u32, u32), (dr, dc): (u32, u32), mv: bool) {
        let mut buf = Vec::new();
        for r in r1..=r2 {
            for c in c1..=c2 {
                buf.push(self.get(r, c));
            }
        }
        if mv {
            for r in r1..=r2 {
                for c in c1..=c2 {
                    self.put(r, c, None);
                }
            }
        }
        let w = c2 - c1 + 1;
        for r in r1..=r2 {
            for c in c1..=c2 {
                let cell = buf[((r - r1) * w + (c - c1)) as usize].clone();
                self.put(dr + r - r1, dc + c - c1, cell);
            }
        }
    }
}

struct Lcg(u64);
impl Lcg {
    fn next(&mut self, bound: u32) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) % bound as u64) as u32
    }
}

fn compare(g: &Arc<dyn SpreadsheetGrid>, m: &Model, step: i64) {
    let (mut max_r, mut max_c, mut min_r, mut min_c) = (0, 0, u32::MAX, u32::MAX);
    for r in 1..=300 {
        for c in 1..=60 {
            let e = m.get(r, c);
            let a = addr(r, c);
            let want = match &e {
                Some(MCell { text: Some(t), .. }) => t.clone(),
                Some(MCell {
                    number: Some(n), ..
                }) => n.to_string(),
                _ => String::new(),
            };
            assert_eq!(g.get_value(&a).unwrap(), want, "step {step} cell {a}");
            assert_eq!(
                g.style_of(&a).unwrap().starts_with("true|"),
                e.as_ref().map(|x| x.bold).unwrap_or(false),
                "step {step} bold {a}"
            );
            if e.is_some() {
                max_r = max_r.max(r);
                max_c = max_c.max(c);
                min_r = min_r.min(r);
                min_c = min_c.min(c);
            }
        }
    }
    let range = if max_r == 0 {
        String::new()
    } else {
        format!("{}:{}", addr(min_r, min_c), addr(max_r, max_c))
    };
    assert_eq!(g.used_range(), range, "step {step}");
    assert_eq!(g.row_count(), max_r);
    assert_eq!(g.column_count(), max_c);
}

#[test]
fn should_agree_with_a_simple_model_over_thousands_of_random_operations() {
    let g = sut!(create());
    let mut m = Model::default();
    let mut rnd = Lcg(12345);
    for step in 0..4_000i64 {
        let r = 1 + rnd.next(15);
        let c = 1 + rnd.next(8);
        match rnd.next(10) {
            0 | 1 => {
                let v = rnd.next(100) as i64 - 50;
                g.set_number(&addr(r, c), v as f64).unwrap();
                let b = m.get(r, c).map(|x| x.bold).unwrap_or(false);
                m.put(
                    r,
                    c,
                    Some(MCell {
                        text: None,
                        number: Some(v),
                        bold: b,
                    }),
                );
            }
            2 => {
                let t = format!("t{}", rnd.next(50));
                g.set_text(&addr(r, c), &t).unwrap();
                let b = m.get(r, c).map(|x| x.bold).unwrap_or(false);
                m.put(
                    r,
                    c,
                    Some(MCell {
                        text: Some(t),
                        number: None,
                        bold: b,
                    }),
                );
            }
            3 => {
                g.clear(&addr(r, c)).unwrap();
                m.put(r, c, None);
            }
            4 => {
                let b = rnd.next(2) == 0;
                g.set_style(&addr(r, c), &bold(b)).unwrap();
                let old = m.get(r, c);
                m.put(
                    r,
                    c,
                    Some(MCell {
                        text: old.as_ref().and_then(|x| x.text.clone()),
                        number: old.as_ref().and_then(|x| x.number),
                        bold: b,
                    }),
                );
            }
            5 => {
                g.insert_row(r).unwrap();
                m.shift(true, r, 1);
            }
            6 => {
                g.delete_row(r).unwrap();
                m.shift(true, r, -1);
            }
            7 => {
                if rnd.next(2) == 0 {
                    g.insert_column(&col(c)).unwrap();
                    m.shift(false, c, 1);
                } else {
                    g.delete_column(&col(c)).unwrap();
                    m.shift(false, c, -1);
                }
            }
            _ => {
                let r2 = r + rnd.next(4);
                let c2 = c + rnd.next(3);
                let dr = 1 + rnd.next(15);
                let dc = 1 + rnd.next(8);
                let mv = rnd.next(2) == 0;
                if mv {
                    g.move_range(&addr(r, c), &addr(r2, c2), &addr(dr, dc))
                        .unwrap();
                } else {
                    g.copy(&addr(r, c), &addr(r2, c2), &addr(dr, dc)).unwrap();
                }
                m.copy((r, c, r2, c2), (dr, dc), mv);
            }
        }
        if step % 25 == 0 {
            compare(&g, &m, step);
        }
    }
    compare(&g, &m, -1);
}

#[test]
fn should_clear_a_huge_range_in_time_proportional_to_the_cells_present() {
    let g = sut!(create());
    for r in 1..=2_000 {
        for c in 1..=100 {
            g.set_number(&addr(r, c), 1.0).unwrap();
        }
    }
    let start = Instant::now();
    for _ in 0..200 {
        g.clear_range("ZZ999999", "ZZ1000000").unwrap();
        g.clear_range("DA1", "ZZ1000000").unwrap();
    }
    g.clear_range("A1", "ZZ1000000").unwrap();

    assert!(start.elapsed().as_millis() < 10_000);
    assert_eq!(g.used_range(), "");
}

#[test]
fn should_sort_fifty_thousand_rows_stably_and_keep_styles_with_their_rows() {
    let g = sut!(create());
    let n = 50_000u32;
    for r in 1..=n {
        let key = ((r as u64 * 7919) % 1000) as u32;
        if r % 10 == 0 {
            g.set_text(&format!("A{r}"), &format!("k{}", key % 100))
                .unwrap();
        } else if r % 10 != 1 {
            g.set_number(&format!("A{r}"), key as f64).unwrap();
        }
        g.set_number(&format!("B{r}"), r as f64).unwrap();
        if r % 3 == 0 {
            g.set_style(&format!("B{r}"), &bold(true)).unwrap();
        }
    }

    g.sort_range("A1", &format!("B{n}"), "A", true).unwrap();

    let (mut prev_kind, mut prev_number, mut prev_payload) = (0, f64::NEG_INFINITY, 0u32);
    for r in 1..=n {
        let kind = match g.value_type(&format!("A{r}")).unwrap() {
            ValueType::Number => 0,
            ValueType::Text => 1,
            ValueType::Empty => 2,
        };
        assert!(kind >= prev_kind);
        let payload: u32 = g.get_value(&format!("B{r}")).unwrap().parse().unwrap();
        assert_eq!(
            g.style_of(&format!("B{r}")).unwrap().starts_with("true|"),
            payload % 3 == 0
        );
        if kind == 0 {
            let v: f64 = g.get_value(&format!("A{r}")).unwrap().parse().unwrap();
            assert!(v >= prev_number);
            if v == prev_number {
                assert!(payload > prev_payload);
            }
            prev_number = v;
        } else if kind == 2 && prev_kind == 2 {
            assert!(payload > prev_payload);
        }
        prev_kind = kind;
        prev_payload = payload;
    }
}

#[test]
fn should_keep_a_consistent_grid_when_readers_run_during_structure_changes() {
    let g = sut!(create());
    g.set_text("A1", "anchor").unwrap();
    g.set_number("B2", 5.0).unwrap();

    run_concurrently(8, |t| {
        if t < 4 {
            for _ in 0..40 {
                g.insert_row(1).unwrap();
                g.insert_column("A").unwrap();
            }
        } else {
            for _ in 0..2_000 {
                assert!(!g.used_range().is_empty());
                assert!(g.row_count() >= 2);
                g.get_value("C3").unwrap();
                g.distinct_style_count();
            }
        }
    });

    assert_eq!(g.get_value(&addr(161, 161)).unwrap(), "anchor");
    assert_eq!(g.get_value(&addr(162, 162)).unwrap(), "5");
    assert_eq!(
        g.used_range(),
        format!("{}:{}", addr(161, 161), addr(162, 162))
    );
}

#[test]
fn should_handle_the_grid_corners_when_structure_changes() {
    let g = sut!(create());
    g.set_text("ZZ1000000", "corner").unwrap();

    g.insert_row(1).unwrap();
    assert_eq!(g.used_range(), ""); // pushed past the last row

    g.set_text("ZZ1000000", "corner").unwrap();
    g.delete_row(1).unwrap();
    assert_eq!(g.get_value("ZZ999999").unwrap(), "corner");
    g.insert_row(1).unwrap();
    assert_eq!(g.get_value("ZZ1000000").unwrap(), "corner");

    g.delete_column("A").unwrap();
    assert_eq!(g.get_value("ZY1000000").unwrap(), "corner");
    g.insert_column("ZY").unwrap();
    assert_eq!(g.get_value("ZZ1000000").unwrap(), "corner");
    g.insert_column("ZZ").unwrap();
    assert_eq!(g.used_range(), "");

    g.set_style("ZZ1", &bold(true)).unwrap();
    g.delete_column("zz").unwrap();
    assert_eq!(g.distinct_style_count(), 0);
    assert_eq!(g.used_range(), "");
}

#[test]
fn should_format_large_and_tiny_numbers_without_exponents() {
    let g = sut!(create());
    g.set_number("A1", 1e20).unwrap();
    g.set_number("A2", -123456789012345678.0).unwrap();
    g.set_number("A3", 1.0e-10 / 3.0).unwrap();
    g.set_number("A4", 12345.678901234567).unwrap();
    g.set_number("A5", 0.99999999995).unwrap();
    g.set_number("A6", 999.9999999999).unwrap();

    assert_eq!(g.get_value("A1").unwrap(), "100000000000000000000");
    assert_eq!(g.get_value("A2").unwrap(), "-123456789012345680");
    assert_eq!(g.get_value("A3").unwrap(), "0");
    assert_eq!(g.get_value("A4").unwrap(), "12345.6789012346");
    assert_eq!(g.get_value("A5").unwrap(), "1");
    assert_eq!(g.get_value("A6").unwrap(), "999.9999999999");

    g.set_style_range("A1", "A6", &pct("PERCENT")).unwrap();
    assert_eq!(g.get_value("A5").unwrap(), "100%");
    assert_eq!(g.get_value("A1").unwrap(), "10000000000000000000000%");
    g.set_style_range("A1", "A6", &pct("TWO_DECIMALS")).unwrap();
    assert_eq!(g.get_value("A6").unwrap(), "1000.00");
    assert_eq!(g.get_value("A3").unwrap(), "0.00");
}

#[test]
fn should_copy_across_structure_changes_without_sharing_state() {
    let g = sut!(create());
    g.set_number("A1", 1.0).unwrap();
    g.set_style(
        "A1",
        &StyleChange {
            bold: Some(true),
            text_color: Some("blue".into()),
            ..Default::default()
        },
    )
    .unwrap();
    g.copy("A1", "A1", "B1").unwrap();
    g.set_style("A1", &bold(false)).unwrap();
    g.set_number("B1", 2.0).unwrap();

    assert_eq!(
        g.style_of("A1").unwrap(),
        "false|false|11|blue|none|GENERAL"
    );
    assert_eq!(g.style_of("B1").unwrap(), "true|false|11|blue|none|GENERAL");
    assert_eq!(g.distinct_style_count(), 2);

    g.insert_column("A").unwrap();
    g.copy("B1", "C1", "A5").unwrap();
    assert_eq!(g.get_value("A5").unwrap(), "1");
    assert_eq!(g.get_value("B5").unwrap(), "2");
    g.delete_column("B").unwrap();
    assert_eq!(g.get_value("A5").unwrap(), "1");
    assert_eq!(g.get_value("B5").unwrap(), "");
    assert_eq!(g.used_range(), "A1:B5");
}
