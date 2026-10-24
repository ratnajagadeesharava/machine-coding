#![allow(unused_imports, clippy::needless_range_loop)]
use p141_matrix_traversal::{contract::*, create};
use playground_support::{collect_concurrently, sut};
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
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

fn g3() -> Vec<Vec<i32>> {
    vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]]
}
fn g23() -> Vec<Vec<i32>> {
    vec![vec![1, 2, 3], vec![4, 5, 6]]
}
fn g32() -> Vec<Vec<i32>> {
    vec![vec![1, 2], vec![3, 4], vec![5, 6]]
}

/// value = r * cols + c, counts every get(); never held in memory.
struct Counting {
    rows: usize,
    cols: usize,
    gets: AtomicU64,
}

impl Counting {
    fn new(rows: usize, cols: usize) -> Arc<Counting> {
        Arc::new(Counting {
            rows,
            cols,
            gets: AtomicU64::new(0),
        })
    }
    fn gets(&self) -> u64 {
        self.gets.load(Ordering::SeqCst)
    }
}

impl MatrixSource for Counting {
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
        (r * self.cols + c) as i32
    }
}

fn grid(g: Vec<Vec<i32>>) -> Arc<dyn MatrixSource> {
    Arc::new(GridSource::new(&g).unwrap())
}

macro_rules! of {
    ($g:expr) => {
        sut!(create(grid($g))).expect("valid matrix")
    };
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

/// A strategy defined by the test: the reverse of COLUMN_MAJOR.
struct ReverseColumnMajor;
impl TraversalStrategy for ReverseColumnMajor {
    fn position(&self, rows: usize, cols: usize, index: u64) -> (usize, usize) {
        let j = rows as u64 * cols as u64 - 1 - index;
        ((j % rows as u64) as usize, (j / rows as u64) as usize)
    }
}

/// Row-major, except that visit `bad_at` returns whatever `bad` says.
struct Faulty {
    bad_at: u64,
    bad: Box<dyn Fn(usize, usize) -> (usize, usize) + Send + Sync>,
}
impl TraversalStrategy for Faulty {
    fn position(&self, rows: usize, cols: usize, index: u64) -> (usize, usize) {
        if index == self.bad_at {
            return (self.bad)(rows, cols);
        }
        (
            (index / cols as u64) as usize,
            (index % cols as u64) as usize,
        )
    }
}

fn faulty(
    bad_at: u64,
    bad: impl Fn(usize, usize) -> (usize, usize) + Send + Sync + 'static,
) -> Box<Faulty> {
    Box::new(Faulty {
        bad_at,
        bad: Box::new(bad),
    })
}

#[test]
fn should_traverse_row_major_and_its_reverse() {
    let t = of!(g3());

    assert_eq!(
        t.traverse("ROW_MAJOR").unwrap(),
        [1, 2, 3, 4, 5, 6, 7, 8, 9]
    );
    assert_eq!(
        t.traverse("REVERSE_ROW_MAJOR").unwrap(),
        [9, 8, 7, 6, 5, 4, 3, 2, 1]
    );
}

#[test]
fn should_traverse_column_major() {
    let t = of!(g3());
    assert_eq!(
        t.traverse("COLUMN_MAJOR").unwrap(),
        [1, 4, 7, 2, 5, 8, 3, 6, 9]
    );
}

#[test]
fn should_traverse_snake_rows_and_snake_columns() {
    let t = of!(g3());

    assert_eq!(t.traverse("SNAKE").unwrap(), [1, 2, 3, 6, 5, 4, 7, 8, 9]);
    assert_eq!(
        t.traverse("BOUSTROPHEDON_COLUMNS").unwrap(),
        [1, 4, 7, 8, 5, 2, 3, 6, 9]
    );
}

#[test]
fn should_traverse_spiral_clockwise_and_counter_clockwise() {
    let t = of!(g3());

    assert_eq!(t.traverse("SPIRAL").unwrap(), [1, 2, 3, 6, 9, 8, 7, 4, 5]);
    assert_eq!(
        t.traverse("SPIRAL_COUNTER").unwrap(),
        [1, 4, 7, 8, 9, 6, 3, 2, 5]
    );
}

#[test]
fn should_traverse_anti_diagonals_top_right_to_bottom_left() {
    let t = of!(g3());
    assert_eq!(t.traverse("DIAGONAL").unwrap(), [1, 2, 4, 3, 5, 7, 6, 8, 9]);
}

#[test]
fn should_traverse_zigzag_like_leetcode_498() {
    let t = of!(g3());
    assert_eq!(t.traverse("ZIGZAG").unwrap(), [1, 2, 4, 7, 5, 3, 6, 8, 9]);
}

#[test]
fn should_handle_wide_matrices() {
    let t = of!(g23());

    assert_eq!(t.traverse("ROW_MAJOR").unwrap(), [1, 2, 3, 4, 5, 6]);
    assert_eq!(t.traverse("COLUMN_MAJOR").unwrap(), [1, 4, 2, 5, 3, 6]);
    assert_eq!(t.traverse("SNAKE").unwrap(), [1, 2, 3, 6, 5, 4]);
    assert_eq!(t.traverse("SPIRAL").unwrap(), [1, 2, 3, 6, 5, 4]);
    assert_eq!(t.traverse("SPIRAL_COUNTER").unwrap(), [1, 4, 5, 6, 3, 2]);
    assert_eq!(t.traverse("DIAGONAL").unwrap(), [1, 2, 4, 3, 5, 6]);
    assert_eq!(t.traverse("ZIGZAG").unwrap(), [1, 2, 4, 5, 3, 6]);
    assert_eq!(
        t.traverse("BOUSTROPHEDON_COLUMNS").unwrap(),
        [1, 4, 5, 2, 3, 6]
    );
    assert_eq!(t.traverse("REVERSE_ROW_MAJOR").unwrap(), [6, 5, 4, 3, 2, 1]);
}

#[test]
fn should_handle_tall_matrices() {
    let t = of!(g32());

    assert_eq!(t.traverse("ROW_MAJOR").unwrap(), [1, 2, 3, 4, 5, 6]);
    assert_eq!(t.traverse("COLUMN_MAJOR").unwrap(), [1, 3, 5, 2, 4, 6]);
    assert_eq!(t.traverse("SNAKE").unwrap(), [1, 2, 4, 3, 5, 6]);
    assert_eq!(t.traverse("SPIRAL").unwrap(), [1, 2, 4, 6, 5, 3]);
    assert_eq!(t.traverse("SPIRAL_COUNTER").unwrap(), [1, 3, 5, 6, 4, 2]);
    assert_eq!(t.traverse("DIAGONAL").unwrap(), [1, 2, 3, 4, 5, 6]);
    assert_eq!(t.traverse("ZIGZAG").unwrap(), [1, 2, 3, 5, 4, 6]);
    assert_eq!(
        t.traverse("BOUSTROPHEDON_COLUMNS").unwrap(),
        [1, 3, 5, 6, 4, 2]
    );
    assert_eq!(t.traverse("REVERSE_ROW_MAJOR").unwrap(), [6, 5, 4, 3, 2, 1]);
}

#[test]
fn should_handle_single_row_single_column_and_single_cell() {
    let row = of!(vec![vec![7, 8, 9]]);
    let col = of!(vec![vec![7], vec![8], vec![9]]);
    let one = of!(vec![vec![42]]);

    assert_eq!(row.traverse("SPIRAL").unwrap(), [7, 8, 9]);
    assert_eq!(row.traverse("SPIRAL_COUNTER").unwrap(), [7, 8, 9]);
    assert_eq!(row.traverse("COLUMN_MAJOR").unwrap(), [7, 8, 9]);
    assert_eq!(row.traverse("ZIGZAG").unwrap(), [7, 8, 9]);
    assert_eq!(col.traverse("SPIRAL").unwrap(), [7, 8, 9]);
    assert_eq!(col.traverse("SPIRAL_COUNTER").unwrap(), [7, 8, 9]);
    assert_eq!(col.traverse("SNAKE").unwrap(), [7, 8, 9]);
    assert_eq!(col.traverse("BOUSTROPHEDON_COLUMNS").unwrap(), [7, 8, 9]);
    assert_eq!(col.traverse("DIAGONAL").unwrap(), [7, 8, 9]);
    assert_eq!(col.traverse("ZIGZAG").unwrap(), [7, 8, 9]);
    for name in BUILT_INS {
        assert_eq!(one.traverse(name).unwrap(), [42], "{name}");
    }
}

#[test]
fn should_visit_every_cell_exactly_once_for_every_strategy_and_shape() {
    for rows in 1..=6 {
        for cols in 1..=6 {
            let t = over!(Counting::new(rows, cols));
            for name in BUILT_INS {
                let seen = t.traverse(name).unwrap();
                let distinct: HashSet<i32> = seen.iter().copied().collect();
                assert_eq!(seen.len(), rows * cols, "{name} {rows}x{cols}");
                assert_eq!(distinct.len(), rows * cols, "{name} {rows}x{cols}");
            }
        }
    }
}

#[test]
fn should_reject_unknown_and_misspelled_strategy_names() {
    let t = of!(g3());

    assert_eq!(t.traverse("NOPE"), Err(TraversalError::UnknownStrategy));
    assert_eq!(
        t.traverse("row_major"),
        Err(TraversalError::UnknownStrategy)
    );
    assert_eq!(t.traverse(""), Err(TraversalError::UnknownStrategy));
    assert_eq!(t.start("Spiral"), Err(TraversalError::UnknownStrategy));
    assert_eq!(t.path_sum("NOPE", 1), Err(TraversalError::UnknownStrategy));
    assert_eq!(t.kth_value("NOPE", 0), Err(TraversalError::UnknownStrategy));
}

#[test]
fn should_reject_invalid_matrices() {
    let _valid = of!(g3()); // the factory works for a valid matrix; now the invalid ones
    assert!(matches!(
        GridSource::new(&[]),
        Err(TraversalError::InvalidMatrix)
    ));
    assert!(matches!(
        GridSource::new(&[vec![]]),
        Err(TraversalError::InvalidMatrix)
    ));
    assert!(matches!(
        GridSource::new(&[vec![1, 2], vec![3]]),
        Err(TraversalError::InvalidMatrix)
    ));
    assert!(matches!(
        create(Counting::new(0, 5)),
        Some(Err(TraversalError::InvalidMatrix))
    ));
    assert!(matches!(
        create(Counting::new(5, 0)),
        Some(Err(TraversalError::InvalidMatrix))
    ));
}

#[test]
fn should_not_see_later_changes_to_the_vec_it_was_built_from() {
    let mut g = vec![vec![1, 2], vec![3, 4]];
    let t = of!(g.clone());
    g[0][0] = 99;
    g[1] = vec![7, 7];

    assert_eq!(t.traverse("ROW_MAJOR").unwrap(), [1, 2, 3, 4]);
}

#[test]
fn should_iterate_with_a_cursor() {
    let t = of!(g3());
    let c = t.start("SPIRAL").unwrap();

    assert_eq!(c, 1);
    assert_eq!(t.has_next(c), Ok(true));
    assert_eq!(t.remaining(c), Ok(9));
    assert_eq!(t.next(c), Ok(Some(1)));
    assert_eq!(t.next(c), Ok(Some(2)));
    assert_eq!(t.remaining(c), Ok(7));
    assert_eq!(drain(&t, c), [3, 6, 9, 8, 7, 4, 5]);
    assert_eq!(t.has_next(c), Ok(false));
    assert_eq!(t.remaining(c), Ok(0));
}

#[test]
fn should_return_none_once_exhausted_and_keep_doing_so() {
    let t = of!(vec![vec![5, 6]]);
    let c = t.start("ROW_MAJOR").unwrap();

    assert_eq!(drain(&t, c), [5, 6]);
    assert_eq!(t.next(c), Ok(None));
    assert_eq!(t.next(c), Ok(None));
    assert_eq!(t.has_next(c), Ok(false));
    assert_eq!(t.remaining(c), Ok(0));
}

#[test]
fn should_reset_a_cursor_to_the_start_even_when_exhausted() {
    let t = of!(g3());
    let c = t.start("SNAKE").unwrap();
    t.next(c).unwrap();
    t.next(c).unwrap();
    t.reset(c).unwrap();

    assert_eq!(t.remaining(c), Ok(9));
    assert_eq!(drain(&t, c), [1, 2, 3, 6, 5, 4, 7, 8, 9]);
    t.reset(c).unwrap();
    assert_eq!(t.has_next(c), Ok(true));
    assert_eq!(t.next(c), Ok(Some(1)));
}

#[test]
fn should_keep_cursors_independent_of_each_other() {
    let t = of!(g3());
    let a = t.start("ROW_MAJOR").unwrap();
    let b = t.start("COLUMN_MAJOR").unwrap();
    let c = t.start("ROW_MAJOR").unwrap();

    assert_eq!([a, b, c], [1, 2, 3]);
    assert_eq!(t.next(a), Ok(Some(1)));
    assert_eq!(t.next(a), Ok(Some(2)));
    assert_eq!(t.next(b), Ok(Some(1)));
    assert_eq!(t.next(b), Ok(Some(4)));
    assert_eq!(t.remaining(c), Ok(9));
    t.reset(a).unwrap();
    assert_eq!(t.remaining(b), Ok(7));
    assert_eq!(t.next(c), Ok(Some(1)));
}

#[test]
fn should_reject_unknown_and_closed_cursors_and_never_reuse_ids() {
    let t = of!(g3());
    let a = t.start("ROW_MAJOR").unwrap();
    t.close(a).unwrap();
    let b = t.start("ROW_MAJOR").unwrap();

    assert_eq!(b, 2);
    let unknown = Err(TraversalError::UnknownCursor);
    assert_eq!(t.next(a), unknown);
    assert_eq!(t.has_next(a), Err(TraversalError::UnknownCursor));
    assert_eq!(t.remaining(a), Err(TraversalError::UnknownCursor));
    assert_eq!(t.reset(a), Err(TraversalError::UnknownCursor));
    assert_eq!(t.close(a), Err(TraversalError::UnknownCursor));
    assert_eq!(t.next(0), Err(TraversalError::UnknownCursor));
    assert_eq!(t.next(77), Err(TraversalError::UnknownCursor));
    assert_eq!(t.next(b), Ok(Some(1)));
}

#[test]
fn should_compute_path_sum_of_the_first_k_visits() {
    let t = of!(g3());

    assert_eq!(t.path_sum("SPIRAL", 0), Ok(0));
    assert_eq!(t.path_sum("SPIRAL", 1), Ok(1));
    assert_eq!(t.path_sum("SPIRAL", 4), Ok(1 + 2 + 3 + 6));
    assert_eq!(t.path_sum("ZIGZAG", 5), Ok(1 + 2 + 4 + 7 + 5));
    assert_eq!(t.path_sum("REVERSE_ROW_MAJOR", 3), Ok(9 + 8 + 7));
}

#[test]
fn should_clamp_path_sum_to_the_whole_matrix_and_reject_negative_k() {
    let t = of!(g3());

    assert_eq!(t.path_sum("ROW_MAJOR", 9), Ok(45));
    assert_eq!(t.path_sum("ROW_MAJOR", 10), Ok(45));
    assert_eq!(t.path_sum("ROW_MAJOR", i64::MAX), Ok(45));
    assert_eq!(
        t.path_sum("ROW_MAJOR", -1),
        Err(TraversalError::InvalidArgument)
    );
}

#[test]
fn should_sum_in_64_bits_without_overflowing_i32() {
    let m = i32::MAX;
    let t = of!(vec![vec![m, m], vec![m, -5]]);

    assert_eq!(t.path_sum("ROW_MAJOR", 3), Ok(3 * m as i64));
    assert_eq!(t.path_sum("ROW_MAJOR", 4), Ok(3 * m as i64 - 5));
}

#[test]
fn should_return_the_kth_value_and_reject_out_of_range() {
    let t = of!(g3());

    assert_eq!(t.kth_value("SPIRAL", 0), Ok(1));
    assert_eq!(t.kth_value("SPIRAL", 4), Ok(9));
    assert_eq!(t.kth_value("SPIRAL", 8), Ok(5));
    assert_eq!(t.kth_value("DIAGONAL", 3), Ok(3));
    assert_eq!(t.kth_value("BOUSTROPHEDON_COLUMNS", 4), Ok(5));
    assert_eq!(
        t.kth_value("SPIRAL", 9),
        Err(TraversalError::InvalidArgument)
    );
    assert_eq!(
        t.kth_value("SPIRAL", -1),
        Err(TraversalError::InvalidArgument)
    );
}

#[test]
fn should_use_a_registered_strategy_through_traverse_cursors_path_sum_and_kth() {
    let t = of!(g3());
    t.register_strategy("REVERSE_COLUMN_MAJOR", Box::new(ReverseColumnMajor))
        .unwrap();

    assert_eq!(
        t.traverse("REVERSE_COLUMN_MAJOR").unwrap(),
        [9, 6, 3, 8, 5, 2, 7, 4, 1]
    );
    let c = t.start("REVERSE_COLUMN_MAJOR").unwrap();
    assert_eq!(t.next(c), Ok(Some(9)));
    assert_eq!(t.next(c), Ok(Some(6)));
    assert_eq!(t.remaining(c), Ok(7));
    assert_eq!(t.path_sum("REVERSE_COLUMN_MAJOR", 3), Ok(9 + 6 + 3));
    assert_eq!(t.kth_value("REVERSE_COLUMN_MAJOR", 4), Ok(5));
}

#[test]
fn should_hand_out_the_matrix_shape_to_a_registered_strategy() {
    struct Spy(Arc<Mutex<Vec<(usize, usize)>>>);
    impl TraversalStrategy for Spy {
        fn position(&self, rows: usize, cols: usize, index: u64) -> (usize, usize) {
            self.0.lock().unwrap().push((rows, cols));
            (
                (index / cols as u64) as usize,
                (index % cols as u64) as usize,
            )
        }
    }
    let t = of!(g23());
    let shapes = Arc::new(Mutex::new(vec![]));
    t.register_strategy("SPY", Box::new(Spy(shapes.clone())))
        .unwrap();

    assert_eq!(t.traverse("SPY").unwrap(), [1, 2, 3, 4, 5, 6]);
    let seen = shapes.lock().unwrap();
    assert!(!seen.is_empty() && seen.iter().all(|s| *s == (2, 3)));
}

#[test]
fn should_reject_duplicate_and_built_in_registrations() {
    let t = of!(g3());
    t.register_strategy("MINE", Box::new(ReverseColumnMajor))
        .unwrap();

    assert_eq!(
        t.register_strategy("MINE", Box::new(ReverseColumnMajor)),
        Err(TraversalError::DuplicateStrategy)
    );
    assert_eq!(
        t.register_strategy("SPIRAL", Box::new(ReverseColumnMajor)),
        Err(TraversalError::DuplicateStrategy)
    );
    assert_eq!(
        t.register_strategy(" ", Box::new(ReverseColumnMajor)),
        Err(TraversalError::InvalidArgument)
    );
    assert_eq!(t.traverse("SPIRAL").unwrap(), [1, 2, 3, 6, 9, 8, 7, 4, 5]);
    // names are case-sensitive: "spiral" is a new name
    t.register_strategy("spiral", Box::new(ReverseColumnMajor))
        .unwrap();
    assert_eq!(t.kth_value("spiral", 0), Ok(9));
}

#[test]
fn should_keep_registered_strategies_per_service_instance() {
    let a = of!(g3());
    let b = of!(g3());
    a.register_strategy("MINE", Box::new(ReverseColumnMajor))
        .unwrap();

    assert_eq!(b.traverse("MINE"), Err(TraversalError::UnknownStrategy));
    b.register_strategy("MINE", Box::new(ReverseColumnMajor))
        .unwrap();
}

#[test]
fn should_fail_traverse_naming_the_index_of_an_out_of_range_position() {
    let t = of!(g3());
    t.register_strategy("TOO_WIDE_AT_7", faulty(7, |_, cols| (0, cols)))
        .unwrap();
    t.register_strategy("TOO_TALL_AT_4", faulty(4, |rows, _| (rows, 0)))
        .unwrap();

    assert_eq!(
        t.traverse("TOO_WIDE_AT_7"),
        Err(TraversalError::InvalidPosition(7))
    );
    assert_eq!(
        t.traverse("TOO_TALL_AT_4"),
        Err(TraversalError::InvalidPosition(4))
    );
}

#[test]
fn should_fail_traverse_naming_the_index_of_a_repeated_position() {
    let t = of!(g3());
    t.register_strategy("REPEATS", faulty(5, |_, _| (0, 2)))
        .unwrap();

    assert_eq!(
        t.traverse("REPEATS"),
        Err(TraversalError::InvalidPosition(5))
    );
}

#[test]
fn should_fail_cursor_kth_and_path_sum_on_an_out_of_range_position_without_corrupting_the_cursor() {
    let t = of!(g3());
    t.register_strategy("BAD_AT_2", faulty(2, |_, _| (9, 9)))
        .unwrap();
    let c = t.start("BAD_AT_2").unwrap();

    assert_eq!(t.next(c), Ok(Some(1)));
    assert_eq!(t.next(c), Ok(Some(2)));
    assert_eq!(t.next(c), Err(TraversalError::InvalidPosition(2)));
    assert_eq!(t.remaining(c), Ok(7));
    assert_eq!(
        t.kth_value("BAD_AT_2", 2),
        Err(TraversalError::InvalidPosition(2))
    );
    assert_eq!(t.kth_value("BAD_AT_2", 1), Ok(2));
    assert_eq!(
        t.path_sum("BAD_AT_2", 3),
        Err(TraversalError::InvalidPosition(2))
    );
    assert_eq!(t.path_sum("BAD_AT_2", 2), Ok(3));
}

#[test]
fn should_start_and_take_ten_elements_of_a_huge_matrix_with_at_most_hundred_reads() {
    for name in BUILT_INS {
        let huge = Counting::new(20_000, 20_000);
        let t = over!(huge.clone());
        let c = t.start(name).unwrap();
        for _ in 0..10 {
            assert!(t.next(c).unwrap().is_some(), "{name}");
        }
        assert!(huge.gets() <= 100, "{name}: {} reads", huge.gets());
        assert_eq!(t.remaining(c), Ok(400_000_000 - 10), "{name}");
    }
}

#[test]
fn should_read_the_first_values_of_a_huge_matrix_in_the_right_order() {
    let t = over!(Counting::new(20_000, 20_000));
    let spiral = t.start("SPIRAL").unwrap();
    let counter = t.start("SPIRAL_COUNTER").unwrap();
    let zig = t.start("ZIGZAG").unwrap();

    let first: Vec<i32> = (0..4).map(|_| t.next(spiral).unwrap().unwrap()).collect();
    assert_eq!(first, [0, 1, 2, 3]);
    assert_eq!(t.next(counter), Ok(Some(0)));
    assert_eq!(t.next(counter), Ok(Some(20_000)));
    assert_eq!(t.next(zig), Ok(Some(0)));
    assert_eq!(t.next(zig), Ok(Some(1)));
    assert_eq!(t.next(zig), Ok(Some(20_000)));
    assert_eq!(t.next(zig), Ok(Some(40_000)));
}

#[test]
fn should_give_every_thread_its_own_cursor_with_the_full_sequence() {
    let t = over!(Counting::new(40, 25));
    t.register_strategy("REVERSE_COLUMN_MAJOR", Box::new(ReverseColumnMajor))
        .unwrap();
    let mut names: Vec<&str> = BUILT_INS.to_vec();
    names.push("REVERSE_COLUMN_MAJOR");

    let got = collect_concurrently(16, |i| {
        let cursor = t.start(names[i % names.len()]).unwrap();
        drain(&t, cursor)
    });

    for i in 0..16 {
        assert_eq!(
            got[i],
            t.traverse(names[i % names.len()]).unwrap(),
            "thread {i}"
        );
    }
}

#[test]
fn should_hand_out_distinct_cursor_ids_to_concurrent_starters() {
    let t = of!(g3());

    let ids = collect_concurrently(16, |_| {
        (0..100)
            .map(|_| t.start("ROW_MAJOR").unwrap())
            .collect::<Vec<u64>>()
    });

    let all: HashSet<u64> = ids.into_iter().flatten().collect();
    assert_eq!(all.len(), 1_600);
    assert!(all.iter().all(|id| (1..=1_600).contains(id)));
}
