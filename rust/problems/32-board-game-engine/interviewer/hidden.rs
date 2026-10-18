#![allow(unused_imports, clippy::needless_range_loop)]
use p32_board_game_engine::{contract::*, create};
use playground_support::sut;

fn play_all(g: &mut dyn BoardGameApi, moves: &[&str]) -> String {
    let mut last = String::new();
    for (i, mv) in moves.iter().enumerate() {
        last = g.play(if i % 2 == 0 { "P1" } else { "P2" }, mv);
        if i + 1 < moves.len() {
            assert_eq!(last, "OK", "move {i} ({mv})");
        }
    }
    last
}

#[test]
fn should_win_tic_tac_toe_on_the_anti_diagonal() {
    let mut g = sut!(create("tictactoe"));

    assert_eq!(
        play_all(g.as_mut(), &["0,2", "0,0", "1,1", "0,1", "2,0"]),
        "WIN"
    );
}

#[test]
fn should_win_connect4_on_a_descending_diagonal() {
    let mut g = sut!(create("connect4"));

    assert_eq!(
        play_all(
            g.as_mut(),
            &["6", "5", "5", "4", "3", "4", "4", "3", "3", "0", "3"]
        ),
        "WIN"
    );
}

#[test]
fn should_win_connect4_when_the_placed_piece_fills_the_middle_of_the_line() {
    let mut g = sut!(create("connect4"));

    assert_eq!(
        play_all(g.as_mut(), &["0", "0", "1", "1", "3", "3", "2"]),
        "WIN"
    );
}

#[test]
fn should_count_six_in_a_row_as_a_win_in_gomoku() {
    let mut g = sut!(create("gomoku"));

    assert_eq!(
        play_all(
            g.as_mut(),
            &["7,3", "0,0", "7,4", "2,0", "7,5", "4,0", "7,7", "6,0", "7,8", "8,0", "7,6"]
        ),
        "WIN"
    );
}

#[test]
fn should_win_gomoku_on_a_column_and_on_the_anti_diagonal() {
    let mut vertical = sut!(create("gomoku"));
    assert_eq!(
        play_all(
            vertical.as_mut(),
            &["3,7", "0,0", "4,7", "0,2", "5,7", "0,4", "6,7", "0,6", "7,7"]
        ),
        "WIN"
    );

    let mut anti = sut!(create("gomoku"));
    assert_eq!(
        play_all(
            anti.as_mut(),
            &["3,7", "0,0", "4,6", "0,2", "5,5", "0,4", "6,4", "0,6", "7,3"]
        ),
        "WIN"
    );
}

#[test]
fn should_treat_unknown_players_as_not_their_turn_and_huge_numbers_as_invalid() {
    let mut g = sut!(create("tictactoe"));

    assert_eq!(g.play("P3", "0,0"), "NOT_YOUR_TURN");
    assert_eq!(g.play("p1", "0,0"), "NOT_YOUR_TURN");
    assert_eq!(g.play("", "0,0"), "NOT_YOUR_TURN");
    assert_eq!(g.play("P2", "x"), "NOT_YOUR_TURN");
    assert_eq!(g.play("P1", "99999999999,0"), "INVALID_MOVE");
    assert_eq!(g.play("P1", "0,99999999999999999999"), "INVALID_MOVE");
    assert_eq!(g.render(), "...\n...\n...");
}

#[test]
fn should_answer_game_over_before_turn_and_move_checks_on_connect4() {
    let mut g = sut!(create("connect4"));
    play_all(g.as_mut(), &["0", "1", "0", "1", "0", "1", "0"]);

    assert_eq!(g.play("P1", "9"), "GAME_OVER");
    assert_eq!(g.play("nobody", "x"), "GAME_OVER");
    assert_eq!(g.cell_at(4, 0), ".");
}
