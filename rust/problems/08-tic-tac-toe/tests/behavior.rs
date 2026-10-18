#![allow(unused_imports, clippy::needless_range_loop)]
use p08_tic_tac_toe::{contract::*, create};
use playground_support::sut;
use std::time::{Duration, Instant};

/// Plays alternating X, O moves from (row, col) pairs; returns the result of the last move.
fn play(g: &mut dyn TicTacToeApi, moves: &[(i32, i32)]) -> String {
    let mut last = String::new();
    for (i, &(r, c)) in moves.iter().enumerate() {
        let player = if i % 2 == 0 { "X" } else { "O" };
        last = g.play(player, r, c);
    }
    last
}

#[test]
fn should_let_x_start_and_alternate() {
    let mut g = sut!(create(3, 3));

    assert_eq!(g.next_player().as_deref(), Some("X"));
    assert_eq!(g.play("X", 0, 0), "OK");
    assert_eq!(g.next_player().as_deref(), Some("O"));
    assert_eq!(g.cell_at(0, 0).as_deref(), Some("X"));
    assert_eq!(g.cell_at(1, 1), None);
}

#[test]
fn should_reject_move_out_of_turn_without_consuming_the_turn() {
    let mut g = sut!(create(3, 3));

    assert_eq!(g.play("O", 0, 0), "NOT_YOUR_TURN");
    assert_eq!(g.cell_at(0, 0), None);
    assert_eq!(g.play("X", 0, 0), "OK");
    assert_eq!(g.play("X", 1, 1), "NOT_YOUR_TURN");
    assert_eq!(g.next_player().as_deref(), Some("O"));
}

#[test]
fn should_reject_unknown_player() {
    let mut g = sut!(create(3, 3));

    assert_eq!(g.play("Z", 0, 0), "INVALID_PLAYER");
    assert_eq!(g.next_player().as_deref(), Some("X"));
}

#[test]
fn should_reject_occupied_cell_without_consuming_the_turn() {
    let mut g = sut!(create(3, 3));
    g.play("X", 1, 1);

    assert_eq!(g.play("O", 1, 1), "OCCUPIED");

    assert_eq!(g.next_player().as_deref(), Some("O"));
    assert_eq!(g.cell_at(1, 1).as_deref(), Some("X"));
    assert_eq!(g.play("O", 0, 0), "OK");
}

#[test]
fn should_reject_out_of_bounds_cells() {
    let mut g = sut!(create(3, 3));

    assert_eq!(g.play("X", -1, 0), "OUT_OF_BOUNDS");
    assert_eq!(g.play("X", 0, 3), "OUT_OF_BOUNDS");
    assert_eq!(g.play("X", 3, 3), "OUT_OF_BOUNDS");
    assert_eq!(g.next_player().as_deref(), Some("X"));
}

#[test]
fn should_detect_row_win() {
    let mut g = sut!(create(3, 3));

    assert_eq!(
        play(g.as_mut(), &[(0, 0), (1, 0), (0, 1), (1, 1), (0, 2)]),
        "WIN"
    );

    assert_eq!(g.winner().as_deref(), Some("X"));
    assert_eq!(g.next_player(), None);
}

#[test]
fn should_detect_column_win() {
    let mut g = sut!(create(3, 3));

    assert_eq!(
        play(g.as_mut(), &[(0, 0), (0, 1), (1, 0), (1, 1), (2, 0)]),
        "WIN"
    );
    assert_eq!(g.winner().as_deref(), Some("X"));
}

#[test]
fn should_detect_diagonal_win() {
    let mut g = sut!(create(3, 3));

    assert_eq!(
        play(g.as_mut(), &[(0, 0), (0, 1), (1, 1), (0, 2), (2, 2)]),
        "WIN"
    );
    assert_eq!(g.winner().as_deref(), Some("X"));
}

#[test]
fn should_detect_anti_diagonal_win_for_second_player() {
    let mut g = sut!(create(3, 3));

    assert_eq!(
        play(
            g.as_mut(),
            &[(0, 0), (0, 2), (0, 1), (1, 1), (2, 2), (2, 0)]
        ),
        "WIN"
    );
    assert_eq!(g.winner().as_deref(), Some("O"));
}

#[test]
fn should_detect_draw() {
    let mut g = sut!(create(3, 3));

    let result = play(
        g.as_mut(),
        &[
            (0, 0),
            (0, 1),
            (0, 2),
            (1, 1),
            (1, 0),
            (1, 2),
            (2, 1),
            (2, 0),
            (2, 2),
        ],
    );

    assert_eq!(result, "DRAW");
    assert_eq!(g.winner(), None);
    assert_eq!(g.next_player(), None);
}

#[test]
fn should_reject_moves_after_the_game_ends() {
    let mut g = sut!(create(3, 3));
    play(g.as_mut(), &[(0, 0), (1, 0), (0, 1), (1, 1), (0, 2)]);

    assert_eq!(g.play("O", 2, 2), "GAME_OVER");
    assert_eq!(g.play("X", 2, 2), "GAME_OVER");
    assert_eq!(g.cell_at(2, 2), None);
}

#[test]
fn should_support_larger_boards_with_shorter_winning_line() {
    let mut g = sut!(create(5, 4));

    // X builds (0,0)..(3,3) diagonal; O plays elsewhere in row 4.
    assert_eq!(
        play(
            g.as_mut(),
            &[(0, 0), (4, 0), (1, 1), (4, 1), (2, 2), (4, 2), (3, 3)]
        ),
        "WIN"
    );
    assert_eq!(g.winner().as_deref(), Some("X"));
}

#[test]
fn should_not_treat_a_shorter_line_as_a_win_on_a_larger_board() {
    let mut g = sut!(create(5, 4));

    assert_eq!(
        play(g.as_mut(), &[(0, 0), (4, 0), (0, 1), (4, 1), (0, 2)]),
        "OK"
    );
    assert_eq!(g.winner(), None);
    assert_eq!(g.next_player().as_deref(), Some("O"));
}

#[test]
fn should_handle_large_boards_quickly() {
    let mut g = sut!(create(1000, 5));
    let (mut xs, mut os) = (Vec::new(), Vec::new());
    for r in 0..20 {
        for c in 0..500 {
            if (c / 2 + r) % 2 == 0 {
                xs.push((r, c));
            } else {
                os.push((r, c));
            }
        }
    }
    let start = Instant::now();

    for i in 0..xs.len() {
        assert_eq!(g.play("X", xs[i].0, xs[i].1), "OK");
        assert_eq!(g.play("O", os[i].0, os[i].1), "OK");
    }

    assert!(start.elapsed() < Duration::from_secs(10));
}
