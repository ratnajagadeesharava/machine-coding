#![allow(unused_imports, clippy::needless_range_loop)]
use p123_snake_game::{contract::*, create};
use playground_support::sut;

macro_rules! game {
    ($w:expr, $h:expr, $foods:expr) => {
        sut!(create($w, $h, &$foods)).unwrap()
    };
}

fn play(g: &mut Box<dyn SnakeGame>, moves: &str) {
    for c in moves.chars() {
        g.make_move(&c.to_string()).unwrap();
    }
}

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn should_create_a_huge_board_without_depending_on_its_area() {
    let mut g = game!(100_000, 100_000, [(0, 3), (99_999, 99_999)]);

    assert_eq!(g.food_position(), "0,3");
    play(&mut g, "RRR");
    assert_eq!(g.score(), 1);
    assert_eq!(g.food_position(), "99999,99999");
    assert_eq!(g.make_move("D"), Ok(1));
    assert!(!g.is_over());
    assert_eq!(g.head(), "1,3");
}

#[test]
fn should_eat_along_a_huge_corridor_and_win_without_slowing_down() {
    let n: i32 = 200_000;
    let foods: Vec<(i32, i32)> = (1..n).map(|c| (0, c)).collect();
    let mut g = game!(n, 1, foods);

    for i in 1..n - 1 {
        assert_eq!(g.make_move("R"), Ok(i));
    }
    assert!(!g.is_over());
    assert_eq!(g.make_move("R"), Ok(-1));
    assert!(g.is_won());
    assert_eq!(g.length(), n as usize);
    assert_eq!(g.score(), n - 1);
    assert_eq!(g.head(), format!("0,{}", n - 1));
    assert_eq!(g.body().len(), n as usize);
    assert_eq!(g.body()[n as usize - 1], "0,0");
}

#[test]
fn should_fill_a_whole_board_in_a_serpentine() {
    let mut g = game!(
        4,
        3,
        [
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 3),
            (1, 2),
            (1, 1),
            (1, 0),
            (2, 0),
            (2, 1),
            (2, 2),
            (2, 3)
        ]
    );
    let path = "RRRDLLLDRRR";

    for (i, c) in path.chars().take(path.len() - 1).enumerate() {
        assert_eq!(g.make_move(&c.to_string()), Ok(i as i32 + 1));
    }
    assert!(!g.is_won());
    assert_eq!(g.food_position(), "2,3");

    assert_eq!(g.make_move("R"), Ok(-1));

    assert!(g.is_won());
    assert!(g.is_over());
    assert_eq!(g.length(), 12);
    assert_eq!(g.score(), 11);
    assert_eq!(g.food_position(), "none");
    assert_eq!(g.make_move("L"), Ok(-1));
    assert_eq!(g.length(), 12);
}

#[test]
fn should_chase_its_own_tail_for_hundreds_of_thousands_of_moves() {
    let mut g = game!(3, 3, [(0, 1), (1, 1), (1, 0)]);
    play(&mut g, "RDL");
    assert_eq!(g.length(), 4);

    for _ in 0..100_000 {
        assert_eq!(g.make_move("U"), Ok(3));
        assert_eq!(g.make_move("R"), Ok(3));
        assert_eq!(g.make_move("D"), Ok(3));
        assert_eq!(g.make_move("L"), Ok(3));
    }

    assert!(!g.is_over());
    assert_eq!(g.body(), strs(&["1,0", "1,1", "0,1", "0,0"]));
}

#[test]
fn should_not_free_the_tail_cell_on_the_move_that_eats() {
    let mut g = game!(4, 1, [(0, 1), (0, 2), (0, 0), (0, 1)]);

    assert_eq!(g.make_move("R"), Ok(1));
    assert_eq!(g.food_position(), "0,2");

    assert_eq!(g.make_move("R"), Ok(2));

    // (0,0) and (0,1) are both still under the snake: both are skipped, nothing is left
    assert_eq!(g.food_position(), "none");
}

#[test]
fn should_skip_a_long_run_of_foods_under_the_snake_and_keep_the_rest() {
    let mut g = game!(4, 1, [(0, 1), (0, 0), (0, 1), (0, 0), (0, 3), (0, 1)]);

    g.make_move("R").unwrap();

    assert_eq!(g.food_position(), "0,3");
    g.make_move("R").unwrap();
    assert_eq!(g.make_move("R"), Ok(2));
    assert_eq!(g.food_position(), "none");
}

#[test]
fn should_use_the_last_applied_direction_for_reversal_even_after_an_invalid_input() {
    let mut g = game!(4, 4, [(0, 1)]);
    g.make_move("R").unwrap();

    assert_eq!(g.make_move("l"), Err(GameError::InvalidArgument));
    assert!(!g.is_over());
    assert_eq!(g.body(), strs(&["0,1", "0,0"]));

    assert_eq!(g.make_move("L"), Ok(-1));
    assert!(g.is_over());
    assert_eq!(g.head(), "0,1");
}

#[test]
fn should_allow_turning_back_on_the_other_axis_after_a_reversal_was_avoided() {
    let mut g = game!(4, 4, [(0, 1)]);
    g.make_move("R").unwrap();

    assert_eq!(g.make_move("D"), Ok(1));
    assert_eq!(g.make_move("L"), Ok(1));
    assert_eq!(g.make_move("U"), Ok(1));
    assert!(!g.is_over());
    assert_eq!(g.body(), strs(&["0,0", "1,0"]));
}

#[test]
fn should_accept_the_same_cell_more_than_once_in_the_food_list() {
    let mut g = game!(3, 3, [(2, 2), (2, 2), (2, 2)]);

    assert_eq!(g.food_position(), "2,2");
    play(&mut g, "RRDD");
    assert_eq!(g.score(), 1);
    assert_eq!(g.food_position(), "none");
}

#[test]
fn should_return_a_body_snapshot_that_later_moves_do_not_change() {
    let mut g = game!(5, 1, [(0, 1)]);
    g.make_move("R").unwrap();
    let before = g.body();

    g.make_move("R").unwrap();

    assert_eq!(before, strs(&["0,1", "0,0"]));
    assert_eq!(g.body(), strs(&["0,2", "0,1"]));
}
