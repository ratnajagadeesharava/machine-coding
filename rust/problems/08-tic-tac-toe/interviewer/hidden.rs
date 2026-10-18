#![allow(unused_imports, clippy::needless_range_loop)]
use p08_tic_tac_toe::{contract::*, create};
use playground_support::sut;

fn play(g: &mut dyn TicTacToeApi, moves: &[(i32, i32)]) -> String {
    let mut last = String::new();
    for (i, &(r, c)) in moves.iter().enumerate() {
        let player = if i % 2 == 0 { "X" } else { "O" };
        last = g.play(player, r, c);
    }
    last
}

#[test]
fn should_win_by_filling_the_middle_of_a_line() {
    let mut g = sut!(create(3, 3));
    assert_eq!(
        play(g.as_mut(), &[(0, 0), (1, 0), (0, 2), (1, 1), (0, 1)]),
        "WIN"
    );
}

#[test]
fn should_win_with_a_line_longer_than_k() {
    let mut g = sut!(create(6, 3));
    play(
        g.as_mut(),
        &[
            (0, 0),
            (5, 0),
            (0, 1),
            (5, 2),
            (0, 3),
            (5, 4),
            (0, 4),
            (4, 0),
        ],
    );
    assert_eq!(g.play("X", 0, 2), "WIN");
}

#[test]
fn last_cell_that_wins_should_be_win_not_draw() {
    let mut g = sut!(create(3, 3));
    let r = play(
        g.as_mut(),
        &[
            (0, 0),
            (0, 1),
            (0, 2),
            (1, 0),
            (1, 1),
            (1, 2),
            (2, 1),
            (2, 0),
            (2, 2),
        ],
    );
    assert_eq!(r, "WIN");
    assert_eq!(g.winner().as_deref(), Some("X"));
}

#[test]
fn should_detect_anti_diagonal_win_in_the_corner_of_a_large_board() {
    let mut g = sut!(create(4, 3));
    assert_eq!(
        play(g.as_mut(), &[(0, 3), (3, 0), (1, 2), (3, 1), (2, 1)]),
        "WIN"
    );
}

#[test]
fn should_not_win_on_a_non_consecutive_line() {
    let mut g = sut!(create(5, 3));
    assert_eq!(
        play(g.as_mut(), &[(0, 0), (4, 4), (0, 1), (4, 3), (0, 3)]),
        "OK"
    );
    assert_eq!(g.winner(), None);
}
