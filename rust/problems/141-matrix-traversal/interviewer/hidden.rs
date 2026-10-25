#![allow(unused_imports, clippy::needless_range_loop)]
use p141_matrix_traversal::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut, watchdog};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

const BUILT_INS: [&str; 9] = [
    "ROW_MAJOR",
    "COLUMN_MAJOR",
    "SNAKE",
    "SPIRAL",
    "SPIRAL_COUNTER",
    "DIAGONAL",
    "ZIGZAG",
    "BOUSTROPHEDON_COLUMNS",
    "REVERSE_ROW_MAJOR",
];

/// value = 3 * r + c, counts get() calls.
struct Grid {
    rows: usize,
    cols: usize,
    gets: AtomicU64,
}

impl Grid {
    fn new(rows: usize, cols: usize) -> Arc<Grid> {
        Arc::new(Grid {
            rows,
            cols,
            gets: AtomicU64::new(0),
        })
    }
    fn gets(&self) -> u64 {
        self.gets.load(Ordering::SeqCst)
    }
}

fn value(r: usize, c: usize) -> i32 {
    (3 * r + c) as i32
}

impl MatrixSource for Grid {
    fn rows(&self) -> usize {
        self.rows
    }
    fn cols(&self) -> usize {
        self.cols
    }
    fn get(&self, r: usize, c: usize) -> i32 {
        assert!(
            r < self.rows && c < self.cols,
            "get({r},{c}) is outside the matrix"
        );
        self.gets.fetch_add(1, Ordering::SeqCst);
        value(r, c)
    }
}

macro_rules! over {
    ($src:expr) => {
        sut!(create($src)).expect("valid matrix")
    };
}

fn drain(t: &Arc<dyn MatrixTraversal>, cursor: u64) -> Vec<i32> {
    let mut out = vec![];
    while let Some(v) = t.next(cursor).unwrap() {
        out.push(v);
    }
    out
}

// ---- independent oracles: plain simulations, no index maths ------------------------------------------------------

fn spiral_oracle(rows: usize, cols: usize) -> Vec<(usize, usize)> {
    let mut out = vec![];
    let (mut top, mut bottom, mut left, mut right) = (0i64, rows as i64 - 1, 0i64, cols as i64 - 1);
    while top <= bottom && left <= right {
        for c in left..=right {
            out.push((top as usize, c as usize));
        }
        top += 1;
        for r in top..=bottom {
            out.push((r as usize, right as usize));
        }
        right -= 1;
        if top <= bottom {
            for c in (left..=right).rev() {
                out.push((bottom as usize, c as usize));
            }
            bottom -= 1;
        }
        if left <= right {
            for r in (top..=bottom).rev() {
                out.push((r as usize, left as usize));
            }
            left += 1;
        }
    }
    out
}

fn oracle(name: &str, rows: usize, cols: usize) -> Vec<(usize, usize)> {
    let mut out = vec![];
    match name {
        "ROW_MAJOR" => {
            for r in 0..rows {
                for c in 0..cols {
                    out.push((r, c));
                }
            }
        }
        "COLUMN_MAJOR" => {
            for c in 0..cols {
                for r in 0..rows {
                    out.push((r, c));
                }
            }
        }
        "SNAKE" => {
            for r in 0..rows {
                for i in 0..cols {
                    out.push((r, if r % 2 == 0 { i } else { cols - 1 - i }));
                }
            }
        }
        "BOUSTROPHEDON_COLUMNS" => {
            for c in 0..cols {
                for i in 0..rows {
                    out.push((if c % 2 == 0 { i } else { rows - 1 - i }, c));
                }
            }
        }
        "REVERSE_ROW_MAJOR" => {
            out = oracle("ROW_MAJOR", rows, cols);
            out.reverse();
        }
        "SPIRAL" => out = spiral_oracle(rows, cols),
        "SPIRAL_COUNTER" => {
            out = spiral_oracle(cols, rows)
                .into_iter()
                .map(|(r, c)| (c, r))
                .collect();
        }
        "DIAGONAL" | "ZIGZAG" => {
            for s in 0..=(rows + cols - 2) {
                let mut diag = vec![];
                for r in 0..rows {
                    if s >= r && s - r < cols {
                        diag.push((r, s - r));
                    }
                }
                if name == "ZIGZAG" && s % 2 == 0 {
                    diag.reverse();
                }
                out.extend(diag);
            }
        }
        _ => panic!("{name}"),
    }
    out
}

#[test]
fn should_agree_with_plain_simulations_on_every_shape_up_to_9x9() {
    for rows in 1..=9 {
        for cols in 1..=9 {
            let t = over!(Grid::new(rows, cols));
            for name in BUILT_INS {
                let expected: Vec<i32> = oracle(name, rows, cols)
                    .into_iter()
                    .map(|(r, c)| value(r, c))
                    .collect();
                let label = format!("{name} {rows}x{cols}");
                assert_eq!(t.traverse(name).unwrap(), expected, "{label}");

                let cursor = t.start(name).unwrap();
                assert_eq!(drain(&t, cursor), expected, "{label} cursor");

                let mut running = 0i64;
                for k in 0..expected.len() {
                    assert_eq!(
                        t.kth_value(name, k as i64),
                        Ok(expected[k]),
                        "{label} kth {k}"
                    );
                    assert_eq!(t.path_sum(name, k as i64), Ok(running), "{label} sum {k}");
                    running += expected[k] as i64;
                }
                assert_eq!(
                    t.path_sum(name, expected.len() as i64 + 5),
                    Ok(running),
                    "{label}"
                );
            }
        }
    }
}

#[test]
fn should_answer_kth_value_on_a_20000x20000_matrix_with_one_read_and_no_scan() {
    let _w = watchdog(20_000);
    let g = Grid::new(20_000, 20_000);
    let t = over!(g.clone());
    let n: i64 = 400_000_000;

    for name in BUILT_INS {
        for k in [0, 1, 19_999, 20_000, 123_456_789, n / 2, n - 2, n - 1] {
            let before = g.gets();
            t.kth_value(name, k).unwrap();
            assert!(g.gets() - before <= 1, "{name} k={k}");
        }
    }
    // last visits, derived by hand
    assert_eq!(t.kth_value("ROW_MAJOR", n - 1), Ok(value(19_999, 19_999)));
    assert_eq!(
        t.kth_value("REVERSE_ROW_MAJOR", 0),
        Ok(value(19_999, 19_999))
    );
    assert_eq!(
        t.kth_value("COLUMN_MAJOR", n - 1),
        Ok(value(19_999, 19_999))
    );
    assert_eq!(t.kth_value("SNAKE", n - 1), Ok(value(19_999, 0)));
    assert_eq!(
        t.kth_value("BOUSTROPHEDON_COLUMNS", n - 1),
        Ok(value(0, 19_999))
    );
    assert_eq!(t.kth_value("SPIRAL", n - 1), Ok(value(10_000, 9_999)));
    assert_eq!(
        t.kth_value("SPIRAL_COUNTER", n - 1),
        Ok(value(9_999, 10_000))
    );
    assert_eq!(t.kth_value("DIAGONAL", n - 1), Ok(value(19_999, 19_999)));
    assert_eq!(t.kth_value("ZIGZAG", n - 1), Ok(value(19_999, 19_999)));
    assert_eq!(t.kth_value("SPIRAL", 19_999), Ok(value(0, 19_999)));
    assert_eq!(t.kth_value("SPIRAL", 20_000), Ok(value(1, 19_999)));
    assert_eq!(t.kth_value("ZIGZAG", 3), Ok(value(2, 0)));
}

#[test]
fn should_compute_path_sum_of_a_huge_matrix_reading_at_most_k_cells() {
    let _w = watchdog(20_000);
    let g = Grid::new(20_000, 20_000);
    let t = over!(g.clone());

    for name in BUILT_INS {
        let before = g.gets();
        let sum = t.path_sum(name, 5_000).unwrap();
        assert!(g.gets() - before <= 5_000, "{name}");

        let cursor = t.start(name).unwrap();
        let via_cursor: i64 = (0..5_000)
            .map(|_| t.next(cursor).unwrap().unwrap() as i64)
            .sum();
        assert_eq!(sum, via_cursor, "{name}");
    }
    assert_eq!(t.path_sum("ROW_MAJOR", 20_000), Ok(20_000i64 * 19_999 / 2));
}

#[test]
fn should_count_more_cells_than_a_32_bit_int_can_hold() {
    let _w = watchdog(20_000);
    let t = over!(Grid::new(100_000, 100_000));
    let n: i64 = 10_000_000_000;

    let cursor = t.start("COLUMN_MAJOR").unwrap();
    assert_eq!(t.remaining(cursor), Ok(n as u64));
    assert_eq!(t.next(cursor), Ok(Some(value(0, 0))));
    assert_eq!(t.next(cursor), Ok(Some(value(1, 0))));
    assert_eq!(t.remaining(cursor), Ok(n as u64 - 2));
    assert_eq!(t.kth_value("ROW_MAJOR", n - 1), Ok(value(99_999, 99_999)));
    assert_eq!(
        t.kth_value("ROW_MAJOR", 4_294_967_296),
        Ok(value(42_949, 67_296))
    );
    assert_eq!(
        t.kth_value("COLUMN_MAJOR", 4_294_967_296),
        Ok(value(67_296, 42_949))
    );
    assert_eq!(t.kth_value("SPIRAL", n - 1), Ok(value(50_000, 49_999)));
    assert_eq!(
        t.kth_value("SPIRAL_COUNTER", n - 1),
        Ok(value(49_999, 50_000))
    );
    assert_eq!(t.kth_value("DIAGONAL", n - 1), Ok(value(99_999, 99_999)));
    assert_eq!(t.kth_value("ZIGZAG", n - 1), Ok(value(99_999, 99_999)));
    assert_eq!(t.kth_value("REVERSE_ROW_MAJOR", n - 1), Ok(value(0, 0)));
    assert_eq!(
        t.kth_value("ROW_MAJOR", n),
        Err(TraversalError::InvalidArgument)
    );
    assert_eq!(t.path_sum("SPIRAL", 3), Ok(3));
}

#[test]
fn should_handle_extremely_thin_matrices() {
    let _w = watchdog(20_000);
    let r = over!(Grid::new(1, 1_000_000));
    let c = over!(Grid::new(1_000_000, 1));

    for name in BUILT_INS {
        let rev = name == "REVERSE_ROW_MAJOR";
        assert_eq!(
            r.kth_value(name, 0),
            Ok(if rev { 999_999 } else { 0 }),
            "{name}"
        );
        assert_eq!(
            r.kth_value(name, 999_999),
            Ok(if rev { 0 } else { 999_999 }),
            "{name}"
        );
        assert_eq!(
            r.kth_value(name, 1234),
            Ok(if rev { 999_999 - 1234 } else { 1234 }),
            "{name}"
        );
        assert_eq!(
            c.kth_value(name, 0),
            Ok(if rev { 3 * 999_999 } else { 0 }),
            "{name}"
        );
        assert_eq!(
            c.kth_value(name, 999_999),
            Ok(if rev { 0 } else { 3 * 999_999 }),
            "{name}"
        );
        assert_eq!(
            c.kth_value(name, 1234),
            Ok(if rev { 3 * (999_999 - 1234) } else { 3 * 1234 }),
            "{name}"
        );
    }
}

#[test]
fn should_return_each_value_exactly_once_when_threads_share_one_cursor() {
    let t = over!(Grid::new(200, 200));
    let cursor = t.start("SPIRAL").unwrap();

    let parts = collect_concurrently(8, |_| drain(&t, cursor));

    let mut all: Vec<i32> = parts.into_iter().flatten().collect();
    let mut expected = t.traverse("SPIRAL").unwrap();
    assert_eq!(all.len(), 40_000);
    all.sort_unstable();
    expected.sort_unstable();
    assert_eq!(all, expected);
    assert_eq!(t.has_next(cursor), Ok(false));
}

#[test]
fn should_let_exactly_one_thread_register_a_contested_name() {
    struct RowMajor;
    impl TraversalStrategy for RowMajor {
        fn position(&self, _rows: usize, cols: usize, index: u64) -> (usize, usize) {
            (
                (index / cols as u64) as usize,
                (index % cols as u64) as usize,
            )
        }
    }
    let t = over!(Grid::new(4, 4));
    let wins = AtomicUsize::new(0);

    run_concurrently(16, |_| {
        if t.register_strategy("CONTESTED", Box::new(RowMajor)).is_ok() {
            wins.fetch_add(1, Ordering::SeqCst);
        }
    });

    assert_eq!(wins.load(Ordering::SeqCst), 1);
    assert_eq!(t.traverse("CONTESTED").unwrap().len(), 16);
}

#[test]
fn should_ask_a_registered_strategy_only_for_what_it_needs_on_a_huge_matrix() {
    struct Counted(Arc<AtomicU64>);
    impl TraversalStrategy for Counted {
        fn position(&self, _rows: usize, cols: usize, index: u64) -> (usize, usize) {
            self.0.fetch_add(1, Ordering::SeqCst);
            (
                (index / cols as u64) as usize,
                (index % cols as u64) as usize,
            )
        }
    }
    let g = Grid::new(20_000, 20_000);
    let t = over!(g.clone());
    let calls = Arc::new(AtomicU64::new(0));
    t.register_strategy("COUNTED", Box::new(Counted(calls.clone())))
        .unwrap();

    let cursor = t.start("COUNTED").unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(g.gets(), 0);
    for _ in 0..10 {
        t.next(cursor).unwrap();
    }
    assert!(calls.load(Ordering::SeqCst) <= 10);
    assert!(g.gets() <= 100);

    calls.store(0, Ordering::SeqCst);
    assert_eq!(
        t.kth_value("COUNTED", 399_999_999),
        Ok(value(19_999, 19_999))
    );
    assert!(calls.load(Ordering::SeqCst) <= 1);
    calls.store(0, Ordering::SeqCst);
    assert_eq!(t.path_sum("COUNTED", 100), Ok((0..100).sum()));
    assert!(calls.load(Ordering::SeqCst) <= 100);
}

#[test]
fn should_accept_a_registered_permutation_and_reject_one_that_is_only_mostly_a_permutation() {
    struct Permutation {
        order: Vec<usize>,
        last_repeats_first: bool,
    }
    impl TraversalStrategy for Permutation {
        fn position(&self, _rows: usize, cols: usize, index: u64) -> (usize, usize) {
            let i = if self.last_repeats_first && index as usize == self.order.len() - 1 {
                0
            } else {
                index as usize
            };
            (self.order[i] / cols, self.order[i] % cols)
        }
    }
    let (rows, cols) = (30usize, 17usize);
    let n = rows * cols;
    // deterministic shuffle: multiplication by a number coprime with n
    let order: Vec<usize> = (0..n).map(|i| (i * 7 + 3) % n).collect();
    let t = over!(Grid::new(rows, cols));
    t.register_strategy(
        "SHUFFLED",
        Box::new(Permutation {
            order: order.clone(),
            last_repeats_first: false,
        }),
    )
    .unwrap();
    t.register_strategy(
        "ALMOST",
        Box::new(Permutation {
            order: order.clone(),
            last_repeats_first: true,
        }),
    )
    .unwrap();

    let expected: Vec<i32> = order
        .iter()
        .map(|&cell| value(cell / cols, cell % cols))
        .collect();
    assert_eq!(t.traverse("SHUFFLED").unwrap(), expected);
    assert_eq!(
        t.traverse("ALMOST"),
        Err(TraversalError::InvalidPosition(n as u64 - 1))
    );
    // the very same strategy is fine for the prefix it gets right
    let prefix: i64 = expected.iter().take(n - 1).map(|&v| v as i64).sum();
    assert_eq!(t.path_sum("ALMOST", n as i64 - 1), Ok(prefix));
}

#[test]
fn should_fail_a_mid_stream_bad_position_only_when_reached_and_keep_other_cursors_working() {
    struct BadAt5;
    impl TraversalStrategy for BadAt5 {
        fn position(&self, rows: usize, cols: usize, index: u64) -> (usize, usize) {
            if index == 5 {
                (rows, cols)
            } else {
                (
                    (index / cols as u64) as usize,
                    (index % cols as u64) as usize,
                )
            }
        }
    }
    let t = over!(Grid::new(3, 3));
    t.register_strategy("BAD_AT_5", Box::new(BadAt5)).unwrap();
    let bad = t.start("BAD_AT_5").unwrap();
    let good = t.start("ROW_MAJOR").unwrap();

    for _ in 0..5 {
        assert!(t.next(bad).unwrap().is_some());
    }
    assert_eq!(t.next(bad), Err(TraversalError::InvalidPosition(5)));
    assert_eq!(t.next(bad), Err(TraversalError::InvalidPosition(5)));
    assert_eq!(t.remaining(bad), Ok(4));
    t.reset(bad).unwrap();
    assert_eq!(t.next(bad), Ok(Some(0)));
    assert_eq!(t.next(good), Ok(Some(0)));
    assert_eq!(t.has_next(good), Ok(true));
}

#[test]
fn should_open_and_close_a_hundred_thousand_cursors() {
    let t = over!(Grid::new(20_000, 20_000));
    let ids: Vec<u64> = (0..100_000)
        .map(|i| t.start(BUILT_INS[i % BUILT_INS.len()]).unwrap())
        .collect();
    for (i, id) in ids.iter().enumerate() {
        assert_eq!(*id, i as u64 + 1);
    }
    for id in &ids {
        t.close(*id).unwrap();
    }
    assert_eq!(t.start("SPIRAL"), Ok(100_001));
    assert_eq!(t.next(ids[0]), Err(TraversalError::UnknownCursor));
}

#[test]
fn should_survive_cursors_closing_while_others_keep_reading() {
    let t = over!(Grid::new(60, 60));
    let expected = t.traverse("ZIGZAG").unwrap();

    let got = collect_concurrently(16, |_| {
        for _ in 0..20 {
            let scratch = t.start("ROW_MAJOR").unwrap();
            t.next(scratch).unwrap();
            t.close(scratch).unwrap();
        }
        let cursor = t.start("ZIGZAG").unwrap();
        let mine = drain(&t, cursor);
        t.close(cursor).unwrap();
        mine
    });

    for mine in got {
        assert_eq!(mine, expected);
    }
}

#[test]
fn should_match_the_oracle_sums_on_an_odd_sized_thousand_square_grid() {
    let t = over!(Grid::new(1_000, 1_001));

    for name in BUILT_INS {
        let expected: i64 = oracle(name, 1_000, 1_001)
            .into_iter()
            .map(|(r, c)| value(r, c) as i64)
            .sum();
        assert_eq!(t.path_sum(name, i64::MAX), Ok(expected), "{name}");
    }
}
