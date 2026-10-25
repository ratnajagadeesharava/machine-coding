#![allow(unused_imports, clippy::needless_range_loop)]
use p143_excel_formulas::{contract::*, create};
use playground_support::{collect_concurrently, sut};
use std::time::Instant;

fn cell_name(i: usize) -> String {
    format!("{}{}", (b'A' + (i % 26) as u8) as char, i / 26 + 1)
}

fn on_default_stack(body: fn()) {
    std::thread::spawn(body)
        .join()
        .expect("default-stack thread failed");
}

#[test]
fn should_build_the_chain_backwards_and_evaluate_every_cell_once_when_the_head_arrives() {
    on_default_stack(|| {
        let s = sut!(create());
        for i in (1..1000).rev() {
            s.set(&cell_name(i), &format!("={}+1", cell_name(i - 1)))
                .unwrap();
        }
        assert_eq!(s.get(&cell_name(999)).unwrap(), "999");
        let before = s.recompute_count();
        s.set("A1", "1").unwrap();
        assert_eq!(s.recompute_count() - before, 999);
        assert_eq!(s.get(&cell_name(999)).unwrap(), "1000");
        assert_eq!(s.get("B1").unwrap(), "2");
        assert_eq!(s.dependents("A1").unwrap(), vec!["B1".to_string()]);
    });
}

#[test]
fn should_detect_a_reference_cycle_through_all_twenty_six_hundred_cells_and_recover_from_it() {
    on_default_stack(|| {
        let s = sut!(create());
        for i in 0..2600 {
            s.set(
                &cell_name(i),
                &format!("={}+1", cell_name((i + 2599) % 2600)),
            )
            .unwrap();
        }
        for i in (0..2600).step_by(137) {
            assert_eq!(s.get(&cell_name(i)).unwrap(), "#CYCLE!");
        }
        assert_eq!(s.get("Z100").unwrap(), "#CYCLE!");
        assert_eq!(s.get("A1").unwrap(), "#CYCLE!");

        let before = s.recompute_count();
        s.set("A1", "1").unwrap();
        assert_eq!(s.recompute_count() - before, 2599);
        assert_eq!(s.get("Z100").unwrap(), "2600");
        assert_eq!(s.get("B1").unwrap(), "2");
    });
}

#[test]
fn should_not_evaluate_a_cell_once_per_path_through_a_layered_lattice() {
    let s = sut!(create());
    s.set("A1", "1").unwrap();
    for c in 1..26 {
        s.set(&cell_name(c), "=A1").unwrap();
    }
    for r in 2..=8 {
        for c in 0..26 {
            s.set(
                &cell_name((r - 1) * 26 + c),
                &format!("=SUM(A{}:Z{})", r - 1, r - 1),
            )
            .unwrap();
        }
    }
    assert_eq!(s.get("Z1").unwrap(), "1");
    assert_eq!(s.get("A2").unwrap(), "26");
    assert_eq!(s.get("Z3").unwrap(), "676");
    assert_eq!(s.get("M8").unwrap(), "8031810176");

    let before = s.recompute_count();
    s.set("A1", "2").unwrap();
    assert_eq!(s.recompute_count() - before, 25 + 7 * 26);
    assert_eq!(s.get("M8").unwrap(), "16063620352");
    assert_eq!(s.get("A2").unwrap(), "52");
}

#[test]
fn should_handle_twenty_six_large_ranges_over_the_whole_grid() {
    let s = sut!(create());
    for i in 0..99 * 26 {
        s.set(&cell_name(i), &(i % 7).to_string()).unwrap();
    }
    for c in 0..26 {
        s.set(&cell_name(99 * 26 + c), "=SUM(A1:Z99)").unwrap();
    }
    let mut expected: i64 = (0..99 * 26).map(|i| (i % 7) as i64).sum();
    assert_eq!(s.get("A100").unwrap(), expected.to_string());
    assert_eq!(s.dependencies("Z100").unwrap().len(), 99 * 26);
    assert_eq!(s.dependents("A1").unwrap().len(), 26);
    assert_eq!(s.dependents("Z99").unwrap().len(), 26);

    let before = s.recompute_count();
    let start = Instant::now();
    for k in 0..100usize {
        let i = (k * 37) % (99 * 26);
        s.set(&cell_name(i), "100").unwrap();
        expected += 100 - (i % 7) as i64;
        if k % 25 == 0 {
            assert_eq!(s.get("Q100").unwrap(), expected.to_string());
        }
        s.set(&cell_name(i), &(i % 7).to_string()).unwrap();
        expected -= 100 - (i % 7) as i64;
    }
    assert_eq!(s.recompute_count() - before, 100 * 2 * 26);
    assert_eq!(s.get("Z100").unwrap(), expected.to_string());
    assert!(start.elapsed().as_millis() < 30_000);
}

#[test]
fn should_propagate_errors_along_a_long_chain_and_recover_when_the_head_is_fixed() {
    on_default_stack(|| {
        let s = sut!(create());
        s.set("A1", "=1/0").unwrap();
        for i in 1..2600 {
            s.set(&cell_name(i), &format!("={}+1", cell_name(i - 1)))
                .unwrap();
        }
        assert_eq!(s.get("Z100").unwrap(), "#DIV/0!");
        s.set("A1", "text").unwrap();
        assert_eq!(s.get("Z100").unwrap(), "#VALUE!");
        s.set("A1", "=Z101").unwrap();
        assert_eq!(s.get("M50").unwrap(), "#REF!");
        s.set("A1", "0").unwrap();
        assert_eq!(s.get("Z100").unwrap(), "2599");
    });
}

#[test]
fn should_count_syntax_errors_cycle_tails_and_identical_rewrites() {
    let s = sut!(create());
    s.set("A1", "=1+").unwrap();
    assert_eq!(s.recompute_count(), 1);
    assert_eq!(s.get("A1").unwrap(), "#ERR!");
    s.set("A1", "=1+").unwrap();
    assert_eq!(s.recompute_count(), 2);

    s.set("B1", "5").unwrap();
    s.set("B2", "=B1+1").unwrap();
    s.set("B3", "=B2+1").unwrap();
    let before = s.recompute_count();
    s.set("B1", "5").unwrap();
    assert_eq!(s.recompute_count() - before, 2);

    s.set("C1", "=C2").unwrap();
    s.set("C2", "=C1").unwrap();
    s.set("C3", "=C1+1").unwrap();
    s.set("C4", "=C3+1").unwrap();
    let before = s.recompute_count();
    s.set("C2", "=C1").unwrap();
    assert_eq!(s.recompute_count() - before, 2);
    assert_eq!(s.get("C4").unwrap(), "#CYCLE!");
    s.set("C2", "").unwrap();
    assert_eq!(s.get("C1").unwrap(), "0");
    assert_eq!(s.get("C4").unwrap(), "2");
    assert_eq!(s.dependents("C2").unwrap(), vec!["C1".to_string()]);
}

#[test]
fn should_follow_values_through_text_number_and_formula_transitions() {
    let s = sut!(create());
    s.set("A1", "10").unwrap();
    s.set("B1", "=A1").unwrap();
    s.set("C1", "=B1*2").unwrap();
    s.set("D1", "=SUM(A1:C1)").unwrap();
    assert_eq!(s.get("D1").unwrap(), "40");
    s.set("A1", "ten").unwrap();
    assert_eq!(s.get("B1").unwrap(), "ten");
    assert_eq!(s.get("C1").unwrap(), "#VALUE!");
    assert_eq!(s.get("D1").unwrap(), "#VALUE!");
    s.set("A1", "=B1").unwrap();
    assert_eq!(s.get("D1").unwrap(), "#CYCLE!");
    s.set("A1", "").unwrap();
    assert_eq!(s.get("B1").unwrap(), "0");
    assert_eq!(s.get("D1").unwrap(), "0");
    s.set("B1", "0.1").unwrap();
    s.set("E1", "=SUM(B1,B1,B1,B1,B1,B1,B1,B1,B1,B1)").unwrap();
    assert_eq!(s.get("E1").unwrap(), "1");
    let nested = format!("={}B1+1{}", "(".repeat(200), ")".repeat(200));
    s.set("F1", &nested).unwrap();
    assert_eq!(s.get("F1").unwrap(), "1.1");
}

#[test]
fn should_keep_derived_cells_mutually_consistent_while_writers_race_readers() {
    let s = sut!(create());
    s.set("A1", "1").unwrap();
    s.set("B1", "=A1").unwrap();
    s.set("C1", "=A1*2").unwrap();
    s.set("D1", "=C1-B1-B1").unwrap();
    s.set("E1", "=SUM(B1:C1)").unwrap();
    s.set("F1", "=E1-A1*3").unwrap();

    let results = collect_concurrently(8, |t| {
        if t < 3 {
            for i in 0..2_000 {
                s.set("A1", &(t * 10_000 + i).to_string()).unwrap();
            }
            return true;
        }
        let mut ok = true;
        for _ in 0..5_000 {
            ok &= s.get("D1").unwrap() == "0";
            ok &= s.get("F1").unwrap() == "0";
            ok &= s.dependencies("D1").unwrap() == vec!["B1".to_string(), "C1".to_string()];
        }
        ok
    });
    assert!(results.iter().all(|&b| b));
    assert_eq!(s.recompute_count(), 5 + 3 * 2_000 * 5);
}
