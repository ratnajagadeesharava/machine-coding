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
fn should_start_with_a_single_cell_snake_at_the_top_left() {
    let g = game!(3, 2, [(1, 2), (0, 1)]);

    assert_eq!(g.score(), 0);
    assert!(!g.is_over());
    assert!(!g.is_won());
    assert_eq!(g.length(), 1);
    assert_eq!(g.head(), "0,0");
    assert_eq!(g.body(), strs(&["0,0"]));
    assert_eq!(g.food_position(), "1,2");
}

#[test]
fn should_reject_invalid_boards_and_food_lists() {
    let _valid = game!(3, 3, []);
    for (w, h) in [(0, 3), (3, 0), (-1, 2), (2, -5)] {
        assert!(matches!(
            create(w, h, &[]),
            Some(Err(GameError::InvalidArgument))
        ));
    }
    for food in [(2, 0), (0, 3), (-1, 0), (0, -1), (5, 5)] {
        assert!(matches!(
            create(3, 2, &[food]),
            Some(Err(GameError::InvalidArgument))
        ));
    }
}

#[test]
fn should_reject_anything_but_the_four_upper_case_directions() {
    let mut g = game!(3, 3, []);

    for bad in ["", "u", "r", "UP", "RIGHT", " U", "U ", "X", "UD"] {
        assert_eq!(g.make_move(bad), Err(GameError::InvalidArgument));
    }
    assert_eq!(g.head(), "0,0");
    assert!(!g.is_over());
    assert_eq!(g.score(), 0);
}

#[test]
fn should_move_in_all_four_directions() {
    let mut g = game!(3, 3, []);

    assert_eq!(g.make_move("R"), Ok(0));
    assert_eq!(g.head(), "0,1");
    assert_eq!(g.make_move("D"), Ok(0));
    assert_eq!(g.head(), "1,1");
    assert_eq!(g.make_move("L"), Ok(0));
    assert_eq!(g.head(), "1,0");
    assert_eq!(g.make_move("U"), Ok(0));
    assert_eq!(g.head(), "0,0");
    assert!(!g.is_over());
    assert_eq!(g.length(), 1);
    assert_eq!(g.body(), strs(&["0,0"]));
}

#[test]
fn should_end_the_game_on_every_wall() {
    for dir in ["U", "L"] {
        let mut g = game!(3, 3, []);
        assert_eq!(g.make_move(dir), Ok(-1));
        assert!(g.is_over());
        assert!(!g.is_won());
    }
    let mut right = game!(3, 3, []);
    play(&mut right, "RR");
    assert!(!right.is_over());
    assert_eq!(right.make_move("R"), Ok(-1));
    assert!(right.is_over());

    let mut down = game!(3, 3, []);
    play(&mut down, "DD");
    assert!(!down.is_over());
    assert_eq!(down.make_move("D"), Ok(-1));
    assert!(down.is_over());
}

#[test]
fn should_leave_the_snake_where_it_was_on_a_fatal_move() {
    let mut g = game!(3, 1, [(0, 2)]);
    play(&mut g, "RR");
    assert_eq!(g.score(), 1);

    assert_eq!(g.make_move("R"), Ok(-1));

    assert!(g.is_over());
    assert_eq!(g.head(), "0,2");
    assert_eq!(g.body(), strs(&["0,2", "0,1"]));
    assert_eq!(g.length(), 2);
    assert_eq!(g.score(), 1);
}

#[test]
fn should_ignore_every_move_after_game_over() {
    let mut g = game!(3, 3, [(0, 1)]);
    g.make_move("R").unwrap();
    g.make_move("U").unwrap(); // wall
    assert!(g.is_over());
    let body = g.body();

    for d in ["D", "L", "R", "U", "D"] {
        assert_eq!(g.make_move(d), Ok(-1));
    }
    assert_eq!(g.body(), body);
    assert_eq!(g.head(), "0,1");
    assert_eq!(g.score(), 1);
    assert!(g.is_over());
    assert_eq!(g.make_move("x"), Err(GameError::InvalidArgument));
}

#[test]
fn should_grow_and_score_when_the_head_reaches_food() {
    let mut g = game!(3, 2, [(1, 2), (0, 1)]);

    assert_eq!(g.make_move("R"), Ok(0));
    assert_eq!(g.make_move("D"), Ok(0));
    assert_eq!(g.make_move("R"), Ok(1));
    assert_eq!(g.length(), 2);
    assert_eq!(g.body(), strs(&["1,2", "1,1"]));
    assert_eq!(g.food_position(), "0,1");
    assert_eq!(g.make_move("U"), Ok(1));
    assert_eq!(g.body(), strs(&["0,2", "1,2"]));
    assert_eq!(g.make_move("L"), Ok(2));
    assert_eq!(g.body(), strs(&["0,1", "0,2", "1,2"]));
    assert_eq!(g.score(), 2);
    assert_eq!(g.food_position(), "none");
    assert_eq!(g.make_move("U"), Ok(-1));
    assert!(g.is_over());
}

#[test]
fn should_keep_the_food_where_it_is_until_it_is_eaten() {
    let mut g = game!(4, 3, [(2, 3), (0, 1)]);

    play(&mut g, "RRD");
    assert_eq!(g.food_position(), "2,3");
    assert_eq!(g.score(), 0);
    play(&mut g, "LL");
    assert_eq!(g.food_position(), "2,3");
}

#[test]
fn should_move_the_tail_forward_when_nothing_is_eaten() {
    let mut g = game!(5, 1, [(0, 1), (0, 2), (0, 4)]);
    play(&mut g, "RR");
    assert_eq!(g.body(), strs(&["0,2", "0,1", "0,0"]));

    assert_eq!(g.make_move("R"), Ok(2));

    assert_eq!(g.length(), 3);
    assert_eq!(g.body(), strs(&["0,3", "0,2", "0,1"]));
}

#[test]
fn should_skip_a_food_listed_under_the_snake_at_creation() {
    let mut g = game!(3, 1, [(0, 0), (0, 2)]);

    assert_eq!(g.food_position(), "0,2");
    assert_eq!(g.make_move("R"), Ok(0));
    assert_eq!(g.make_move("R"), Ok(1));
    assert_eq!(g.food_position(), "none");
}

#[test]
fn should_skip_foods_listed_under_the_snake_when_the_next_one_spawns() {
    let mut g = game!(3, 3, [(0, 1), (0, 1), (0, 0), (2, 2)]);

    assert_eq!(g.food_position(), "0,1");
    assert_eq!(g.make_move("R"), Ok(1));

    assert_eq!(g.food_position(), "2,2");
}

#[test]
fn should_never_come_back_to_a_skipped_food() {
    let mut g = game!(4, 2, [(0, 1), (0, 0), (1, 3)]);

    g.make_move("R").unwrap();
    assert_eq!(g.food_position(), "1,3");
    play(&mut g, "RRD"); // walks away from (0,0), which stays skipped
    assert_eq!(g.food_position(), "none");
    assert_eq!(g.score(), 2);
}

#[test]
fn should_have_no_food_when_the_list_is_empty_or_used_up() {
    let mut g = game!(3, 3, []);
    assert_eq!(g.food_position(), "none");
    play(&mut g, "RRDD");
    assert_eq!(g.score(), 0);
    assert_eq!(g.length(), 1);

    let mut one = game!(3, 1, [(0, 1)]);
    one.make_move("R").unwrap();
    assert_eq!(one.food_position(), "none");
    one.make_move("R").unwrap();
    assert_eq!(one.length(), 2);
}

#[test]
fn should_allow_a_reversal_while_the_snake_is_one_cell_long() {
    let mut g = game!(3, 3, []);

    assert_eq!(g.make_move("R"), Ok(0));
    assert_eq!(g.make_move("L"), Ok(0));
    assert!(!g.is_over());
    assert_eq!(g.head(), "0,0");
    assert_eq!(g.make_move("D"), Ok(0));
    assert_eq!(g.make_move("U"), Ok(0));
    assert!(!g.is_over());
}

fn assert_reversal_ends_the_game(
    foods: &[(i32, i32)],
    setup: &str,
    reversal: &str,
    expected_length: usize,
) {
    let mut g = sut!(create(4, 4, foods)).unwrap();
    play(&mut g, setup);
    assert!(!g.is_over());
    assert_eq!(g.length(), expected_length);
    let body = g.body();
    let score = g.score();

    assert_eq!(g.make_move(reversal), Ok(-1));

    assert!(g.is_over());
    assert!(!g.is_won());
    assert_eq!(g.body(), body);
    assert_eq!(g.score(), score);
}

#[test]
fn should_end_the_game_on_a_reversal_of_a_longer_snake() {
    assert_reversal_ends_the_game(&[(0, 1)], "R", "L", 2);
    assert_reversal_ends_the_game(&[(1, 0)], "D", "U", 2);
    assert_reversal_ends_the_game(&[(0, 1), (1, 1)], "RDL", "R", 3);
    assert_reversal_ends_the_game(&[(0, 1), (1, 1)], "RDLU", "D", 3);
}

#[test]
fn should_allow_the_same_axis_when_it_is_not_a_reversal() {
    let mut g = game!(5, 2, [(0, 1)]);
    g.make_move("R").unwrap();

    assert_eq!(g.make_move("R"), Ok(1));
    assert_eq!(g.make_move("D"), Ok(1));
    assert_eq!(g.make_move("D"), Ok(-1)); // wall: row 2 does not exist
}

#[test]
fn should_allow_moving_into_the_cell_the_tail_is_leaving() {
    let mut g = game!(3, 3, [(0, 1), (1, 1), (1, 0)]);
    play(&mut g, "RDL");
    assert_eq!(g.body(), strs(&["1,0", "1,1", "0,1", "0,0"]));
    assert_eq!(g.score(), 3);

    assert_eq!(g.make_move("U"), Ok(3));

    assert!(!g.is_over());
    assert_eq!(g.body(), strs(&["0,0", "1,0", "1,1", "0,1"]));
    assert_eq!(g.make_move("R"), Ok(3)); // chases its own tail again
    assert_eq!(g.body(), strs(&["0,1", "0,0", "1,0", "1,1"]));
}

#[test]
fn should_end_the_game_when_the_head_hits_any_other_part_of_the_body() {
    let mut g = game!(3, 3, [(0, 1), (0, 2), (1, 2), (1, 1)]);
    play(&mut g, "RRDL");
    assert_eq!(g.body(), strs(&["1,1", "1,2", "0,2", "0,1", "0,0"]));
    assert_eq!(g.score(), 4);

    assert_eq!(g.make_move("U"), Ok(-1)); // (0,1) is a body cell that is not the tail

    assert!(g.is_over());
    assert!(!g.is_won());
    assert_eq!(g.head(), "1,1");
    assert_eq!(g.length(), 5);
}

#[test]
fn should_win_and_stop_when_the_snake_fills_the_board() {
    let mut g = game!(2, 2, [(0, 1), (1, 1), (1, 0)]);

    assert_eq!(g.make_move("R"), Ok(1));
    assert_eq!(g.make_move("D"), Ok(2));
    assert!(!g.is_over());
    assert_eq!(g.make_move("L"), Ok(-1));

    assert!(g.is_over());
    assert!(g.is_won());
    assert_eq!(g.score(), 3);
    assert_eq!(g.length(), 4);
    assert_eq!(g.body(), strs(&["1,0", "1,1", "0,1", "0,0"]));
    assert_eq!(g.food_position(), "none");
    assert_eq!(g.make_move("U"), Ok(-1));
    assert_eq!(g.body(), strs(&["1,0", "1,1", "0,1", "0,0"]));
    assert!(g.is_won());
}

#[test]
fn should_count_a_one_cell_board_as_already_won() {
    let mut g = game!(1, 1, [(0, 0)]);

    assert!(g.is_over());
    assert!(g.is_won());
    assert_eq!(g.food_position(), "none");
    assert_eq!(g.make_move("R"), Ok(-1));
    assert_eq!(g.score(), 0);
    assert_eq!(g.length(), 1);
}

#[test]
fn should_report_a_loss_as_not_won() {
    let mut g = game!(2, 1, [(0, 1)]);
    assert_eq!(g.make_move("R"), Ok(-1)); // eats the last cell: board full
    assert!(g.is_won());

    let mut lost = game!(2, 2, []);
    lost.make_move("L").unwrap();
    assert!(lost.is_over());
    assert!(!lost.is_won());
}

#[test]
fn should_keep_score_equal_to_eaten_foods_and_length_one_higher() {
    let mut g = game!(6, 1, [(0, 1), (0, 2), (0, 3), (0, 4)]);

    for i in 1..=4 {
        assert_eq!(g.make_move("R"), Ok(i));
        assert_eq!(g.score(), i);
        assert_eq!(g.length(), i as usize + 1);
        assert_eq!(g.body().len(), i as usize + 1);
    }
    assert_eq!(g.body(), strs(&["0,4", "0,3", "0,2", "0,1", "0,0"]));
}

#[test]
fn should_keep_the_food_shown_after_a_loss() {
    let mut g = game!(3, 3, [(2, 2)]);

    g.make_move("U").unwrap();

    assert!(g.is_over());
    assert_eq!(g.food_position(), "2,2");
}

#[test]
fn should_handle_food_in_every_corner_of_a_larger_board() {
    let mut g = game!(4, 3, [(0, 3), (2, 3), (2, 0)]);

    play(&mut g, "RRR");
    assert_eq!(g.score(), 1);
    assert_eq!(g.food_position(), "2,3");
    play(&mut g, "DD");
    assert_eq!(g.score(), 2);
    assert_eq!(g.food_position(), "2,0");
    assert!(!g.is_over());
    assert_eq!(g.head(), "2,3");
}
