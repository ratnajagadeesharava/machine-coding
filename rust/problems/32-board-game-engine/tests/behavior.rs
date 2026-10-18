#![allow(unused_imports, clippy::needless_range_loop)]
use p32_board_game_engine::{contract::*, create};
use playground_support::sut;

/// Plays the moves alternately starting with P1; every move but the last must be OK. Returns the last result.
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

const C4_DRAW: [u8; 42] = [
    1, 3, 1, 1, 1, 1, 1, 3, 5, 2, 3, 0, 4, 3, 2, 5, 5, 2, 6, 0, 6, 4, 0, 0, 0, 4, 2, 6, 3, 0, 5, 4,
    2, 2, 4, 4, 6, 5, 5, 6, 6, 3,
];

#[test]
fn should_host_known_games_and_refuse_unknown_ones() {
    let _known = sut!(create("tictactoe"));
    assert!(create("tictactoe").is_some());
    assert!(create("connect4").is_some());
    assert!(create("gomoku").is_some());
    assert!(create("chess").is_none());
    assert!(create("TicTacToe").is_none());
    assert!(create("").is_none());
}

#[test]
fn should_let_p1_start_and_alternate_turns() {
    let mut g = sut!(create("tictactoe"));
    assert_eq!(g.next_player().as_deref(), Some("P1"));

    assert_eq!(g.play("P1", "1,1"), "OK");
    assert_eq!(g.next_player().as_deref(), Some("P2"));
    assert_eq!(g.play("P2", "0,0"), "OK");
    assert_eq!(g.next_player().as_deref(), Some("P1"));

    assert_eq!(g.cell_at(1, 1), "X");
    assert_eq!(g.cell_at(0, 0), "O");
    assert_eq!(g.cell_at(2, 2), ".");
}

#[test]
fn should_refuse_out_of_turn_moves_without_changing_anything() {
    let mut g = sut!(create("tictactoe"));

    assert_eq!(g.play("P2", "0,0"), "NOT_YOUR_TURN");
    assert_eq!(g.cell_at(0, 0), ".");
    assert_eq!(g.play("P1", "0,0"), "OK");
    assert_eq!(g.play("P1", "0,1"), "NOT_YOUR_TURN");
    assert_eq!(g.cell_at(0, 1), ".");
    assert_eq!(g.next_player().as_deref(), Some("P2"));
}

#[test]
fn should_reject_invalid_moves_without_consuming_the_turn() {
    let mut g = sut!(create("tictactoe"));
    g.play("P1", "1,1");

    assert_eq!(g.play("P2", "1,1"), "INVALID_MOVE");
    assert_eq!(g.play("P2", "3,0"), "INVALID_MOVE");
    assert_eq!(g.play("P2", "0,3"), "INVALID_MOVE");
    assert_eq!(g.play("P2", "-1,0"), "INVALID_MOVE");
    assert_eq!(g.play("P2", "x"), "INVALID_MOVE");
    assert_eq!(g.play("P2", ""), "INVALID_MOVE");
    assert_eq!(g.play("P2", "1, 2"), "INVALID_MOVE");
    assert_eq!(g.play("P2", "1,2,0"), "INVALID_MOVE");

    assert_eq!(g.next_player().as_deref(), Some("P2"));
    assert_eq!(g.play("P2", "0,0"), "OK");
}

#[test]
fn should_return_blank_for_coordinates_outside_the_board() {
    let g = sut!(create("tictactoe"));

    assert_eq!(g.cell_at(-1, 0), ".");
    assert_eq!(g.cell_at(0, 3), ".");
    assert_eq!(g.cell_at(99, 99), ".");
}

#[test]
fn should_win_tic_tac_toe_on_a_row() {
    let mut g = sut!(create("tictactoe"));

    assert_eq!(
        play_all(g.as_mut(), &["0,0", "1,0", "0,1", "1,1", "0,2"]),
        "WIN"
    );
}

#[test]
fn should_win_tic_tac_toe_on_a_column() {
    let mut g = sut!(create("tictactoe"));

    assert_eq!(
        play_all(g.as_mut(), &["0,0", "0,1", "1,0", "1,1", "2,0"]),
        "WIN"
    );
}

#[test]
fn should_win_tic_tac_toe_on_the_main_diagonal() {
    let mut g = sut!(create("tictactoe"));

    assert_eq!(
        play_all(g.as_mut(), &["0,0", "0,1", "1,1", "0,2", "2,2"]),
        "WIN"
    );
}

#[test]
fn should_let_p2_win_too() {
    let mut g = sut!(create("tictactoe"));

    assert_eq!(
        play_all(g.as_mut(), &["0,0", "1,0", "0,1", "1,1", "2,2", "1,2"]),
        "WIN"
    );
}

#[test]
fn should_refuse_everything_after_the_game_is_won() {
    let mut g = sut!(create("tictactoe"));
    play_all(g.as_mut(), &["0,0", "1,0", "0,1", "1,1", "0,2"]);

    assert_eq!(g.play("P2", "2,2"), "GAME_OVER");
    assert_eq!(g.play("P1", "2,2"), "GAME_OVER");
    assert_eq!(g.play("P1", "garbage"), "GAME_OVER");
    assert_eq!(g.cell_at(2, 2), ".");
    assert_eq!(g.next_player(), None);
}

#[test]
fn should_declare_a_draw_when_the_board_fills_up_without_a_winner() {
    let mut g = sut!(create("tictactoe"));

    let last = play_all(
        g.as_mut(),
        &[
            "0,0", "0,1", "0,2", "1,1", "1,0", "1,2", "2,1", "2,0", "2,2",
        ],
    );

    assert_eq!(last, "DRAW");
    assert_eq!(g.play("P2", "0,0"), "GAME_OVER");
    assert_eq!(g.next_player(), None);
}

#[test]
fn should_prefer_win_over_draw_when_the_last_cell_completes_a_line() {
    let mut g = sut!(create("tictactoe"));

    let last = play_all(
        g.as_mut(),
        &[
            "0,0", "0,1", "0,2", "1,0", "1,1", "1,2", "2,1", "2,0", "2,2",
        ],
    );

    assert_eq!(last, "WIN");
}

#[test]
fn should_render_the_tic_tac_toe_board_top_row_first() {
    let mut g = sut!(create("tictactoe"));
    g.play("P1", "0,0");
    g.play("P2", "1,1");
    g.play("P1", "2,1");

    assert_eq!(g.render(), "X..\n.O.\n.X.");
}

#[test]
fn should_drop_connect4_pieces_to_the_lowest_free_row() {
    let mut g = sut!(create("connect4"));

    assert_eq!(g.play("P1", "3"), "OK");
    assert_eq!(g.play("P2", "3"), "OK");
    assert_eq!(g.play("P1", "0"), "OK");

    assert_eq!(g.cell_at(0, 3), "X");
    assert_eq!(g.cell_at(1, 3), "O");
    assert_eq!(g.cell_at(2, 3), ".");
    assert_eq!(g.cell_at(0, 0), "X");
    assert_eq!(g.cell_at(5, 6), ".");
    assert_eq!(g.cell_at(6, 0), ".");
}

#[test]
fn should_render_connect4_with_the_bottom_row_last() {
    let mut g = sut!(create("connect4"));
    g.play("P1", "3");
    g.play("P2", "3");
    g.play("P1", "0");

    assert_eq!(
        g.render(),
        [".......", ".......", ".......", ".......", "...O...", "X..X..."].join("\n")
    );
}

#[test]
fn should_reject_connect4_columns_out_of_range_or_malformed() {
    let mut g = sut!(create("connect4"));

    assert_eq!(g.play("P1", "7"), "INVALID_MOVE");
    assert_eq!(g.play("P1", "-1"), "INVALID_MOVE");
    assert_eq!(g.play("P1", "x"), "INVALID_MOVE");
    assert_eq!(g.play("P1", ""), "INVALID_MOVE");
    assert_eq!(g.play("P1", "3,3"), "INVALID_MOVE");
    assert_eq!(g.play("P1", " 3"), "INVALID_MOVE");
    assert_eq!(g.next_player().as_deref(), Some("P1"));
}

#[test]
fn should_reject_a_move_in_a_full_connect4_column_without_consuming_the_turn() {
    let mut g = sut!(create("connect4"));
    assert_eq!(play_all(g.as_mut(), &["0", "0", "0", "0", "0", "0"]), "OK");

    assert_eq!(g.play("P1", "0"), "INVALID_MOVE");
    assert_eq!(g.next_player().as_deref(), Some("P1"));
    assert_eq!(g.play("P1", "1"), "OK");
}

#[test]
fn should_win_connect4_vertically() {
    let mut g = sut!(create("connect4"));

    assert_eq!(
        play_all(g.as_mut(), &["0", "1", "0", "1", "0", "1", "0"]),
        "WIN"
    );
}

#[test]
fn should_win_connect4_horizontally() {
    let mut g = sut!(create("connect4"));

    assert_eq!(
        play_all(g.as_mut(), &["0", "0", "1", "1", "2", "2", "3"]),
        "WIN"
    );
    assert_eq!(g.play("P2", "4"), "GAME_OVER");
}

#[test]
fn should_win_connect4_on_an_ascending_diagonal() {
    let mut g = sut!(create("connect4"));

    assert_eq!(
        play_all(
            g.as_mut(),
            &["0", "1", "1", "2", "3", "2", "2", "3", "3", "6", "3"]
        ),
        "WIN"
    );
}

#[test]
fn should_not_win_connect4_with_only_three_in_a_row() {
    let mut g = sut!(create("connect4"));

    assert_eq!(play_all(g.as_mut(), &["0", "0", "1", "1", "2"]), "OK");
    assert_eq!(g.play("P2", "5"), "OK");
}

#[test]
fn should_declare_connect4_a_draw_when_the_board_is_full() {
    let mut g = sut!(create("connect4"));
    let moves: Vec<String> = C4_DRAW.iter().map(|c| c.to_string()).collect();
    let refs: Vec<&str> = moves.iter().map(String::as_str).collect();

    assert_eq!(play_all(g.as_mut(), &refs), "DRAW");
    assert_eq!(g.play("P1", "0"), "GAME_OVER");
    assert_eq!(g.next_player(), None);
}

#[test]
fn should_play_gomoku_on_a_15_by_15_board() {
    let mut g = sut!(create("gomoku"));

    assert_eq!(g.play("P1", "7,7"), "OK");
    assert_eq!(g.play("P2", "14,14"), "OK");
    assert_eq!(g.play("P1", "0,0"), "OK");

    assert_eq!(g.cell_at(7, 7), "X");
    assert_eq!(g.cell_at(14, 14), "O");
    let render = g.render();
    let lines: Vec<&str> = render.split('\n').collect();
    assert_eq!(lines.len(), 15);
    assert!(lines.iter().all(|l| l.len() == 15));
    assert_eq!(lines[7], ".......X.......");
    assert!(lines[0].starts_with('X'));
}

#[test]
fn should_reject_gomoku_moves_outside_the_board_or_on_occupied_cells() {
    let mut g = sut!(create("gomoku"));
    g.play("P1", "7,7");

    assert_eq!(g.play("P2", "15,0"), "INVALID_MOVE");
    assert_eq!(g.play("P2", "0,15"), "INVALID_MOVE");
    assert_eq!(g.play("P2", "7,7"), "INVALID_MOVE");
    assert_eq!(g.play("P2", "-1,3"), "INVALID_MOVE");
    assert_eq!(g.play("P2", "14,14"), "OK");
}

#[test]
fn should_need_five_in_a_row_to_win_gomoku() {
    let mut g = sut!(create("gomoku"));

    assert_eq!(
        play_all(
            g.as_mut(),
            &["7,3", "0,0", "7,4", "2,0", "7,5", "4,0", "7,6"]
        ),
        "OK"
    );
    assert_eq!(g.play("P2", "6,0"), "OK");
    assert_eq!(g.play("P1", "7,7"), "WIN");
    assert_eq!(g.play("P2", "8,0"), "GAME_OVER");
}

#[test]
fn should_win_gomoku_on_a_diagonal() {
    let mut g = sut!(create("gomoku"));

    assert_eq!(
        play_all(
            g.as_mut(),
            &["3,3", "0,0", "4,4", "0,2", "5,5", "0,4", "6,6", "0,6", "7,7"]
        ),
        "WIN"
    );
}

#[test]
fn should_keep_games_independent() {
    let mut a = sut!(create("tictactoe"));
    let b = sut!(create("tictactoe"));

    a.play("P1", "0,0");

    assert_eq!(b.cell_at(0, 0), ".");
    assert_eq!(b.next_player().as_deref(), Some("P1"));
}
